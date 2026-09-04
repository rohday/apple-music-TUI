use anyhow::{Context, Result};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Fetches artwork bytes, using an on-disk cache keyed by URL so repeated
/// sessions never re-download the same cover.
pub async fn fetch_artwork_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let cache_path = disk_cache_path(url);
    if let Ok(bytes) = std::fs::read(&cache_path) {
        return Ok(bytes);
    }

    let resp = client
        .get(url)
        .send()
        .await
        .context("Artwork request failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("Artwork HTTP {}", resp.status());
    }
    let bytes = resp
        .bytes()
        .await
        .context("Artwork body read failed")?
        .to_vec();

    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cache_path, &bytes);
    Ok(bytes)
}

fn disk_cache_path(url: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    let dir = crate::config::Config::get_config_dir()
        .map(|d| d.join("art_cache"))
        .unwrap_or_else(|_| std::env::temp_dir().join("appletui_art"));
    dir.join(format!("{hash}.img"))
}

/// One terminal cell rendered as a half block: the upper pixel colors the
/// foreground of `▀`, the lower pixel the background. This works in any
/// terminal that supports truecolor, with no graphics protocol needed.
pub type ArtCells = Vec<Vec<(Color, Color)>>;

/// Decodes an encoded image (png/jpeg) into half-block cells that fit within
/// `max_width` columns and `max_height` rows, preserving aspect ratio.
pub fn to_half_block_cells(bytes: &[u8], max_width: usize, max_height: usize) -> Option<ArtCells> {
    let img = image::load_from_memory(bytes).ok()?.to_rgb8();
    to_half_block_cells_from_image(&img, max_width, max_height)
}

/// Same as [`to_half_block_cells`] but for an already-decoded image, so
/// callers can resize the cached cover repeatedly without re-decoding.
pub fn to_half_block_cells_from_image(
    img: &image::RgbImage,
    max_width: usize,
    max_height: usize,
) -> Option<ArtCells> {
    if max_width == 0 || max_height == 0 {
        return None;
    }
    let (iw, ih) = (img.width() as usize, img.height() as usize);
    if iw == 0 || ih == 0 {
        return None;
    }

    // Each cell covers 1 pixel horizontally and 2 vertically.
    let target_w = max_width;
    let target_h = max_height * 2;
    let scale = (target_w as f64 / iw as f64).min(target_h as f64 / ih as f64);
    let w = ((iw as f64 * scale).round() as usize).clamp(1, target_w);
    let h = ((ih as f64 * scale).round() as usize).clamp(1, target_h);
    let h = h - (h % 2); // half blocks need an even pixel height

    let thumb = image::imageops::thumbnail(img, w.max(1) as u32, h.max(1) as u32);

    let mut cells: ArtCells = Vec::with_capacity(h / 2);
    for row in 0..(h / 2) {
        let mut line = Vec::with_capacity(w);
        for col in 0..w {
            let top = thumb.get_pixel(col as u32, (row * 2) as u32);
            let bottom = thumb.get_pixel(col as u32, ((row * 2 + 1) as u32).min(h as u32 - 1));
            line.push((
                Color::Rgb(top[0], top[1], top[2]),
                Color::Rgb(bottom[0], bottom[1], bottom[2]),
            ));
        }
        cells.push(line);
    }
    Some(cells)
}

/// Converts cached cells into renderable lines of half-block spans.
pub fn art_lines(cells: &ArtCells) -> Vec<Line<'static>> {
    cells
        .iter()
        .map(|row| {
            Line::from(
                row.iter()
                    .map(|(fg, bg)| Span::styled("▀", Style::default().fg(*fg).bg(*bg)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// Centered ♪ glyph inside a muted block for missing artwork.
pub fn glyph_thumbnail_lines(
    width: usize,
    height: usize,
    accent: Color,
    muted: Color,
) -> Vec<Line<'static>> {
    let mid_row = height / 2;
    let mid_col = width / 2;
    (0..height)
        .map(|row| {
            let spans: Vec<Span> = (0..width)
                .map(|col| {
                    if row == mid_row && col == mid_col {
                        Span::styled(
                            "♪",
                            Style::default()
                                .fg(accent)
                                .bg(muted)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled(" ", Style::default().bg(muted))
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn converts_solid_image_to_uniform_cells() {
        let mut img = RgbImage::new(10, 10);
        for p in img.pixels_mut() {
            *p = Rgb([200, 10, 10]);
        }
        // Square image, width-limited: 4 cells wide x 4 px tall = 2 rows.
        let cells = to_half_block_cells_from_image(&img, 4, 3).unwrap();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].len(), 4);
        for row in &cells {
            for (fg, bg) in row {
                assert_eq!(*fg, Color::Rgb(200, 10, 10));
                assert_eq!(*bg, Color::Rgb(200, 10, 10));
            }
        }
    }

    #[test]
    fn aspect_ratio_preserved() {
        let img = RgbImage::new(100, 50);
        // 2:1 image in a square cell budget: 10 wide x 4 px tall = 2 rows.
        let cells = to_half_block_cells_from_image(&img, 10, 10).unwrap();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].len(), 10);
    }

    #[test]
    fn rejects_zero_budget_and_bad_bytes() {
        let img = RgbImage::new(4, 4);
        assert!(to_half_block_cells_from_image(&img, 0, 3).is_none());
        assert!(to_half_block_cells(b"not an image", 4, 3).is_none());
    }

    #[test]
    fn glyph_thumbnail_dimensions() {
        let lines = glyph_thumbnail_lines(6, 3, Color::Rgb(1, 2, 3), Color::Rgb(0, 0, 0));
        assert_eq!(lines.len(), 3);
    }
}
