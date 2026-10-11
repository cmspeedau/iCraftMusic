//! The playback engine: loading tracks and driving the transport.
//!
//! The engine is the only part of the audio module that holds a device. It
//! stays usable when no device could be opened — it simply reports
//! [`AudioError::OutputUnavailable`] from the operations that would make sound,
//! so the queue, the library and the transport buttons keep working.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::decoder::DecoderError;
use rodio::{Decoder, Source};

use super::{AudioError, MAX_VOLUME, Output};

/// The track currently handed to the output device.
#[derive(Debug, Clone)]
struct LoadedTrack {
    /// Where the track was read from; kept so a finished track can be replayed.
    path: PathBuf,
    /// The track's length, when the decoder could determine it.
    duration: Option<Duration>,
    /// Set once the device has consumed the track to the end.
    finished: bool,
}

/// Owns the output device and the transport state of the current track.
pub struct AudioEngine {
    output: Option<Output>,
    volume: f32,
    loaded: Option<LoadedTrack>,
}

impl AudioEngine {
    /// Opens the default output device and returns a ready engine.
    ///
    /// A failure to open a device is not fatal: the engine keeps working with
    /// no sound so the rest of the application stays usable.
    pub fn open() -> Result<Self, AudioError> {
        let output = Output::open()?;
        let engine = Self {
            output: Some(output),
            volume: 1.0,
            loaded: None,
        };
        engine.apply_volume();
        Ok(engine)
    }

    /// The volume the engine will apply to the next track, in `0.0..=1.0`.
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Sets the volume, clamped to the range the engine accepts.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, MAX_VOLUME);
        self.apply_volume();
    }

    /// Whether an output device is open and sound is possible.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.output.is_some()
    }

    /// A description of the open output device, if there is one.
    #[must_use]
    pub fn output_description(&self) -> Option<String> {
        self.output.as_ref().map(Output::description)
    }

    /// Loads `path` and starts playing it, replacing whatever played before.
    ///
    /// Returns the track's duration when the decoder could determine it; the
    /// caller stores that because it may be unknown for some compressed files.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::Open`] if the file cannot be read,
    /// [`AudioError::UnsupportedFormat`] or [`AudioError::Decode`] if the
    /// decoder rejects it, and [`AudioError::OutputUnavailable`] when no device
    /// is open.
    pub fn load(&mut self, path: &Path) -> Result<Option<Duration>, AudioError> {
        let Some(output) = &self.output else {
            return Err(AudioError::OutputUnavailable);
        };

        let file = File::open(path).map_err(|source| AudioError::Open {
            path: path.to_path_buf(),
            source,
        })?;
        let decoder = Decoder::new(file).map_err(|source| match source {
            DecoderError::UnrecognizedFormat => AudioError::UnsupportedFormat {
                path: path.to_path_buf(),
            },
            source => AudioError::Decode {
                path: path.to_path_buf(),
                source,
            },
        })?;

        let duration = decoder.total_duration();

        // Replace, rather than append: `clear` drops the old source, so the new
        // track starts immediately instead of after the current one ends.
        output.player().clear();
        output.player().append(decoder);
        output.player().play();

        self.loaded = Some(LoadedTrack {
            path: path.to_path_buf(),
            duration,
            finished: false,
        });
        Ok(duration)
    }

    /// Whether a track is loaded and the device is playing it.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        match (&self.output, &self.loaded) {
            (Some(output), Some(loaded)) => !loaded.finished && !output.player().is_paused(),
            _ => false,
        }
    }

    /// Whether a track is loaded but paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        match (&self.output, &self.loaded) {
            (Some(output), Some(loaded)) => !loaded.finished && output.player().is_paused(),
            _ => false,
        }
    }

    /// Whether a track is loaded at all.
    #[must_use]
    pub fn has_track(&self) -> bool {
        self.loaded.is_some()
    }

    /// The path of the loaded track, if any.
    #[must_use]
    pub fn current_path(&self) -> Option<&Path> {
        self.loaded.as_ref().map(|loaded| loaded.path.as_path())
    }

    /// The loaded track's duration, if the decoder reported one.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.loaded.as_ref().and_then(|loaded| loaded.duration)
    }

    /// How far into the loaded track playback has reached.
    ///
    /// Reports zero once the track has finished, so the progress bar starts
    /// from the beginning when the next track loads.
    #[must_use]
    pub fn position(&self) -> Duration {
        match (&self.output, &self.loaded) {
            (Some(output), Some(loaded)) if !loaded.finished => output.player().get_pos(),
            _ => Duration::ZERO,
        }
    }

    /// Playback progress as a fraction in `0.0..=1.0`.
    ///
    /// Returns zero when the duration is unknown, which keeps a seek bar from
    /// pretending to know more than it does.
    #[must_use]
    pub fn progress(&self) -> f32 {
        let Some(duration) = self.duration() else {
            return 0.0;
        };
        if duration.is_zero() {
            return 0.0;
        }
        (self.position().as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0) as f32
    }

    /// Pauses playback, keeping the position so it can be resumed.
    pub fn pause(&self) {
        if let Some(output) = &self.output {
            output.player().pause();
        }
    }

    /// Resumes playback, replaying the loaded track if it had finished.
    ///
    /// Resuming at the end of a track restarts it, which is what the play
    /// button should do in that state.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::OutputUnavailable`] when no device is open, or the
    /// same errors as [`AudioEngine::load`] if the track has to be reloaded.
    pub fn resume(&mut self) -> Result<(), AudioError> {
        if self.output.is_none() {
            return Err(AudioError::OutputUnavailable);
        }
        if let Some(path) = self.finished_track_path() {
            return self.load(&path).map(|_| ());
        }
        if let Some(output) = &self.output {
            output.player().play();
        }
        Ok(())
    }

    /// Starts or pauses playback and reports the resulting state.
    ///
    /// # Errors
    ///
    /// As [`AudioEngine::resume`].
    pub fn toggle(&mut self) -> Result<bool, AudioError> {
        if self.is_playing() {
            self.pause();
        } else {
            self.resume()?;
        }
        Ok(self.is_playing())
    }

    /// Stops playback and forgets the loaded track.
    pub fn stop(&mut self) {
        if let Some(output) = &self.output {
            output.player().clear();
        }
        self.loaded = None;
    }

    /// Seeks to `position` within the loaded track.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::Seek`] when the format cannot be seeked, and
    /// [`AudioError::OutputUnavailable`] when no device is open.
    pub fn seek(&self, position: Duration) -> Result<(), AudioError> {
        let Some(output) = &self.output else {
            return Err(AudioError::OutputUnavailable);
        };
        let Some(loaded) = &self.loaded else {
            return Ok(());
        };
        output
            .player()
            .try_seek(position)
            .map_err(|source| AudioError::Seek {
                path: loaded.path.clone(),
                source,
            })
    }

    /// Reports whether the loaded track has just reached its end.
    ///
    /// This is called once per UI frame; it returns `true` exactly once per
    /// track, when the device has drained it, so the caller advances the queue
    /// one track per natural end rather than repeatedly.
    pub fn poll_finished(&mut self) -> bool {
        let Some(loaded) = self.loaded.as_mut() else {
            return false;
        };
        if loaded.finished {
            return false;
        }
        let Some(output) = &self.output else {
            return false;
        };
        if output.player().empty() {
            loaded.finished = true;
            return true;
        }
        false
    }

    /// An engine with no output device.
    ///
    /// Used when the device could not be opened at startup, and by tests: the
    /// engine still tracks its volume and reports its state, and the operations
    /// that need sound fail with [`AudioError::OutputUnavailable`] rather than
    /// panicking.
    #[must_use]
    pub fn without_device() -> Self {
        Self {
            output: None,
            volume: 1.0,
            loaded: None,
        }
    }

    /// The path of the loaded track if it has already finished.
    fn finished_track_path(&self) -> Option<PathBuf> {
        self.loaded
            .as_ref()
            .filter(|loaded| loaded.finished)
            .map(|loaded| loaded.path.clone())
    }

    /// Pushes the current volume onto the player, if there is one.
    fn apply_volume(&self) {
        if let Some(output) = &self.output {
            output.player().set_volume(self.volume);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_engine_without_a_device_still_reports_its_state() {
        let engine = AudioEngine::without_device();

        assert!(!engine.is_available());
        assert!(!engine.has_track());
        assert!(!engine.is_playing());
        assert!(!engine.is_paused());
        assert_eq!(engine.current_path(), None);
        assert_eq!(engine.duration(), None);
        assert_eq!(engine.position(), Duration::ZERO);
        assert_eq!(engine.progress(), 0.0);
        assert_eq!(engine.output_description(), None);
    }

    #[test]
    fn operations_that_need_sound_fail_cleanly_without_a_device() {
        let mut engine = AudioEngine::without_device();

        assert!(matches!(
            engine.load(Path::new("/music/song.mp3")),
            Err(AudioError::OutputUnavailable)
        ));
        assert!(matches!(
            engine.resume(),
            Err(AudioError::OutputUnavailable)
        ));
        assert!(matches!(
            engine.seek(Duration::from_secs(5)),
            Err(AudioError::OutputUnavailable)
        ));
        // The transport controls that only change local state stay harmless.
        engine.pause();
        engine.stop();
        engine.set_volume(0.5);
        assert_eq!(engine.volume(), 0.5);
    }

    #[test]
    fn volume_is_clamped_to_the_accepted_range() {
        let mut engine = AudioEngine::without_device();

        engine.set_volume(4.0);
        assert_eq!(engine.volume(), MAX_VOLUME);
        engine.set_volume(-1.0);
        assert_eq!(engine.volume(), 0.0);
    }

    #[test]
    fn a_finished_track_is_reported_once() {
        let mut engine = AudioEngine::without_device();

        // With no track loaded there is nothing to finish.
        assert!(!engine.poll_finished());
    }
}
