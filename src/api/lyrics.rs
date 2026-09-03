use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub time_secs: f64,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LyricsData {
    pub synced: bool,
    pub lines: Vec<LyricLine>,
}

#[derive(Debug, Deserialize)]
struct LrcLibResponse {
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

impl LyricsData {
    pub fn parse_lrc(lrc_text: &str) -> Self {
        let mut lines = Vec::new();
        for raw_line in lrc_text.lines() {
            let line = raw_line.trim();
            if !line.starts_with('[') {
                continue;
            }
            if let Some(close_bracket) = line.find(']') {
                let time_part = &line[1..close_bracket];
                let text = line[close_bracket + 1..].trim().to_string();

                if let Some(colon) = time_part.find(':') {
                    let min_str = &time_part[..colon];
                    let sec_str = &time_part[colon + 1..];

                    if let (Ok(min), Ok(sec)) = (min_str.parse::<f64>(), sec_str.parse::<f64>()) {
                        let time_secs = min * 60.0 + sec;
                        lines.push(LyricLine { time_secs, text });
                    }
                }
            }
        }

        lines.sort_by(|a, b| {
            a.time_secs
                .partial_cmp(&b.time_secs)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let synced = !lines.is_empty();
        Self { synced, lines }
    }

    pub fn from_plain(plain_text: &str) -> Self {
        let lines = plain_text
            .lines()
            .map(|l| LyricLine {
                time_secs: 0.0,
                text: l.trim().to_string(),
            })
            .collect();
        Self {
            synced: false,
            lines,
        }
    }

    pub fn current_line_idx(&self, current_time_secs: f64) -> Option<usize> {
        if !self.synced || self.lines.is_empty() {
            return None;
        }

        let mut matched = None;
        for (i, line) in self.lines.iter().enumerate() {
            if line.time_secs <= current_time_secs {
                matched = Some(i);
            } else {
                break;
            }
        }
        matched
    }
}

pub async fn fetch_lyrics(
    client: &reqwest::Client,
    track_name: &str,
    artist_name: &str,
    duration_secs: Option<u32>,
    mock_mode: bool,
) -> Result<LyricsData> {
    if mock_mode {
        let sample = format!(
            "[00:02.00] ♫ (Intro) ♫\n[00:08.00] {}\n[00:15.00] Performed by {}\n[00:22.50] Dancing in the shadows of the neon light\n[00:30.00] Music pulsing through the terminal night\n[00:42.00] Beats flowing smoothly line by line\n[00:55.00] AppleTUI running in prime time\n[01:10.00] ♫ (Instrumental Solo) ♫\n[01:30.00] Yeah, everything feels right tonight\n[01:45.00] ♫ (Outro) ♫",
            track_name, artist_name
        );
        return Ok(LyricsData::parse_lrc(&sample));
    }

    let mut req = client
        .get("https://lrclib.net/api/get")
        .query(&[("track_name", track_name), ("artist_name", artist_name)]);

    if let Some(dur) = duration_secs {
        req = req.query(&[("duration", dur.to_string())]);
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(LyricsData::default());
    }

    let parsed: LrcLibResponse = resp.json().await?;
    if let Some(synced) = parsed.synced_lyrics {
        if !synced.is_empty() {
            return Ok(LyricsData::parse_lrc(&synced));
        }
    }

    if let Some(plain) = parsed.plain_lyrics {
        if !plain.is_empty() {
            return Ok(LyricsData::from_plain(&plain));
        }
    }

    Ok(LyricsData::default())
}
