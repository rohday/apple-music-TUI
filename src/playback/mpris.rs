use crate::playback::types::{PlaybackCommand, PlaybackState, PlaybackStatus, RepeatMode};
use mpris_server::{
    zbus::{fdo, Result},
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus as MprisPlaybackStatus, PlayerInterface,
    Property, RootInterface, Server, Time, TrackId, Volume,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

pub struct MprisPlayer {
    pub cmd_tx: mpsc::Sender<PlaybackCommand>,
    pub status: Arc<Mutex<PlaybackStatus>>,
}

impl RootInterface for MprisPlayer {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Stop).await;
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("AppleTUI".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("appletui".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl PlayerInterface for MprisPlayer {
    async fn next(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Next).await;
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Previous).await;
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Pause).await;
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::TogglePlayPause).await;
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Stop).await;
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::Resume).await;
        Ok(())
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let secs = (offset.as_micros() as f64) / 1_000_000.0;
        let _ = self.cmd_tx.send(PlaybackCommand::SeekRelative(secs)).await;
        Ok(())
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        let secs = (position.as_micros() as f64) / 1_000_000.0;
        let _ = self.cmd_tx.send(PlaybackCommand::Seek(secs)).await;
        Ok(())
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<MprisPlaybackStatus> {
        let st = self.status.lock().await;
        match st.state {
            PlaybackState::Playing => Ok(MprisPlaybackStatus::Playing),
            PlaybackState::Paused => Ok(MprisPlaybackStatus::Paused),
            PlaybackState::Stopped | PlaybackState::Loading => Ok(MprisPlaybackStatus::Stopped),
        }
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        let st = self.status.lock().await;
        match st.repeat {
            RepeatMode::Off => Ok(LoopStatus::None),
            RepeatMode::All => Ok(LoopStatus::Playlist),
            RepeatMode::One => Ok(LoopStatus::Track),
        }
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::CycleRepeat).await;
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> Result<()> {
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        let st = self.status.lock().await;
        Ok(st.shuffle)
    }

    async fn set_shuffle(&self, _shuffle: bool) -> Result<()> {
        let _ = self.cmd_tx.send(PlaybackCommand::ToggleShuffle).await;
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let st = self.status.lock().await;
        if let Some(song) = &st.current_song {
            let mut b = Metadata::builder()
                .trackid(TrackId::NO_TRACK)
                .title(&song.name)
                .artist([&song.artist_name])
                .length(Time::from_millis(song.duration_in_millis as i64));
            if let Some(album) = &song.album_name {
                b = b.album(album);
            }
            Ok(b.build())
        } else {
            Ok(Metadata::builder().trackid(TrackId::NO_TRACK).build())
        }
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        let st = self.status.lock().await;
        Ok((st.volume as f64) / 100.0)
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        let vol = (volume * 100.0).clamp(0.0, 100.0) as u8;
        let _ = self.cmd_tx.send(PlaybackCommand::SetVolume(vol)).await;
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        let st = self.status.lock().await;
        Ok(Time::from_micros((st.current_time_secs * 1_000_000.0) as i64))
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}

pub async fn start_mpris_background_service(
    cmd_tx: mpsc::Sender<PlaybackCommand>,
    status: Arc<Mutex<PlaybackStatus>>,
) {
    let player = MprisPlayer {
        cmd_tx,
        status: status.clone(),
    };

    let server = match Server::new("appletui", player).await {
        Ok(s) => s,
        Err(e) => {
            warn!("MPRIS D-Bus service could not be registered: {:#}", e);
            return;
        }
    };

    info!("MPRIS D-Bus service active as org.mpris.MediaPlayer2.appletui");

    let mut prev_state = PlaybackState::Stopped;
    let mut prev_song_id: Option<String> = None;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        let st = status.lock().await.clone();

        let state_changed = st.state != prev_state;
        let song_changed = st.current_song.as_ref().map(|s| &s.id) != prev_song_id.as_ref();

        if state_changed || song_changed {
            prev_state = st.state;
            prev_song_id = st.current_song.as_ref().map(|s| s.id.clone());

            let mpris_status = match st.state {
                PlaybackState::Playing => MprisPlaybackStatus::Playing,
                PlaybackState::Paused => MprisPlaybackStatus::Paused,
                PlaybackState::Stopped | PlaybackState::Loading => MprisPlaybackStatus::Stopped,
            };

            let metadata = if let Some(song) = &st.current_song {
                let mut b = Metadata::builder()
                    .trackid(TrackId::NO_TRACK)
                    .title(&song.name)
                    .artist([&song.artist_name])
                    .length(Time::from_millis(song.duration_in_millis as i64));
                if let Some(album) = &song.album_name {
                    b = b.album(album);
                }
                b.build()
            } else {
                Metadata::builder().trackid(TrackId::NO_TRACK).build()
            };

            let _ = server
                .properties_changed([
                    Property::PlaybackStatus(mpris_status),
                    Property::Metadata(metadata),
                ])
                .await;
        }
    }
}
