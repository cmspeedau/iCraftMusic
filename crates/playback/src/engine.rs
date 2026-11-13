use std::{fs::File, path::PathBuf, time::Duration};

use rodio::{Decoder, DeviceSinkBuilder, Player, Source};

#[derive(Debug)]
pub enum PlaybackError {
    Device(String),
    File(String),
    Decode(String),
    Seek(String),
}

impl std::fmt::Display for PlaybackError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Device(message) => write!(formatter, "Audio output unavailable: {message}"),
            Self::File(message) => write!(formatter, "Could not open audio file: {message}"),
            Self::Decode(message) => write!(formatter, "Could not decode audio file: {message}"),
            Self::Seek(message) => write!(formatter, "Could not seek in audio: {message}"),
        }
    }
}

#[derive(Default)]
pub struct Playback {
    output: Option<rodio::MixerDeviceSink>,
    player: Option<Player>,
    current_path: Option<PathBuf>,
    duration: Option<Duration>,
    error: Option<String>,
}

impl Playback {
    pub fn current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn paused(&self) -> bool {
        self.player
            .as_ref()
            .is_none_or(|player| player.is_paused() || player.empty())
    }

    pub fn finished(&self) -> bool {
        self.player.as_ref().is_some_and(Player::empty)
    }

    pub fn position(&self) -> Duration {
        self.player.as_ref().map_or(Duration::ZERO, Player::get_pos)
    }

    pub fn play_file(&mut self, path: PathBuf) -> Result<(), PlaybackError> {
        self.error = None;
        let result = self.start_file(path);
        if let Err(error) = &result {
            self.stop();
            self.error = Some(error.to_string());
        }
        result
    }

    fn start_file(&mut self, path: PathBuf) -> Result<(), PlaybackError> {
        let file = File::open(&path).map_err(|error| PlaybackError::File(error.to_string()))?;
        let decoder =
            Decoder::try_from(file).map_err(|error| PlaybackError::Decode(error.to_string()))?;
        let duration = decoder.total_duration();
        let output = if let Some(output) = self.output.take() {
            output
        } else {
            DeviceSinkBuilder::open_default_sink()
                .map_err(|error| PlaybackError::Device(error.to_string()))?
        };
        let player = Player::connect_new(output.mixer());
        player.append(decoder);
        self.output = Some(output);
        self.player = Some(player);
        self.current_path = Some(path);
        self.duration = duration;
        Ok(())
    }

    pub fn toggle(&mut self) {
        self.error = None;
        if let Some(player) = &self.player {
            if player.is_paused() {
                player.play();
            } else {
                player.pause();
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
        self.current_path = None;
        self.duration = None;
        self.error = None;
    }

    pub fn seek(&mut self, position: Duration) -> Result<(), PlaybackError> {
        self.error = None;
        let Some(player) = &self.player else {
            return Ok(());
        };

        if let Err(error) = player.try_seek(position) {
            let error = PlaybackError::Seek(error.to_string());
            self.error = Some(error.to_string());
            return Err(error);
        }
        Ok(())
    }
}
