//! The playback engine: loading tracks and driving the transport.
//!
//! The engine is the only part of the audio module that holds a device. It
//! stays usable when no device could be opened — it simply reports
//! [`AudioError::OutputUnavailable`] from the operations that would make sound,
//! so the queue, the library and the transport buttons keep working.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rodio::decoder::DecoderError;
use rodio::source::SeekError;
use rodio::{ChannelCount, Decoder, Sample, SampleRate, Source};

use super::equalizer::Equalized;
use super::{
    AudioError, EqualizerSettings, EqualizerShared, MAX_CROSSFADE_SECONDS, MAX_VOLUME, Output,
};

/// The track currently handed to the output device.
#[derive(Debug, Clone)]
struct LoadedTrack {
    /// Where the track was read from; kept so a finished track can be replayed.
    path: PathBuf,
    /// The track's length, when the decoder could determine it.
    duration: Option<Duration>,
    /// Set once the device has consumed the track to the end.
    finished: bool,
    /// Set by the audio thread the instant the track's last sample is pulled.
    ///
    /// This is how the engine learns that a track has ended without asking the
    /// clock or trusting a length the decoder may not have known.
    done: Arc<AtomicBool>,
}

/// A decoded track held ready to start on the spare deck when a crossfade
/// begins.
///
/// It is deliberately *not* handed to a player yet: during a crossfade the next
/// track starts while the current one is still playing, so it cannot be queued
/// behind it the way a gapless hand-over is.
struct Prepared {
    /// Where the track was read from, so it can be named when it takes over.
    path: PathBuf,
    /// The track's length, when the decoder could determine it.
    duration: Option<Duration>,
    /// The samples, waiting to be appended to the spare deck.
    source: Box<dyn Source + Send>,
    /// Set when the samples are drained; unused until the track is playing.
    done: Arc<AtomicBool>,
}

/// A crossfade in progress: the outgoing track fading out while the incoming
/// one fades in.
struct FadeState {
    /// The active player's position when the fade began.
    ///
    /// The fade is measured from here rather than from a count of frames, so it
    /// keeps time with the audio whatever the interface's frame rate.
    started_at: Duration,
    /// How long the whole fade lasts.
    total: Duration,
    /// The track fading in, promoted to the current track when the fade ends.
    next: LoadedTrack,
}

/// What the output queue did since the last poll.
///
/// The difference between the two is the whole point of gapless playback: a
/// track with another already queued behind it is gone without a pause, and the
/// interface has only to catch up with what is already playing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    /// The track that was waiting took over with no gap between the two.
    Advanced {
        /// The track that is now playing.
        path: PathBuf,
    },
    /// The track ended and nothing was queued behind it.
    Finished {
        /// The track that ended.
        path: PathBuf,
    },
}

/// A decoded track wrapped so the engine learns the instant it is drained.
///
/// rodio plays the sources appended to one [`rodio::Player`] back to back,
/// which is what makes a gapless hand-over possible; this flag is how the
/// engine notices the hand-over has happened — exactly when the last sample is
/// read rather than a frame later, and without depending on a length the
/// decoder may not have reported.
struct Signalled<S> {
    inner: S,
    done: Arc<AtomicBool>,
}

impl<S> Signalled<S> {
    /// Wraps `inner`, returning the source together with the flag it sets when
    /// its samples run out.
    fn new(inner: S) -> (Self, Arc<AtomicBool>) {
        let done = Arc::new(AtomicBool::new(false));
        (
            Self {
                inner,
                done: Arc::clone(&done),
            },
            done,
        )
    }
}

impl<S: Iterator<Item = Sample>> Iterator for Signalled<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next();
        if sample.is_none() {
            // The last sample has just been handed out, so nothing more will
            // play from this source; any hand-over is now.
            self.done.store(true, Ordering::SeqCst);
        }
        sample
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Source> Source for Signalled<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)
    }
}

/// Owns the output device and the transport state of the current track.
pub struct AudioEngine {
    output: Option<Output>,
    volume: f32,
    /// The equalizer every track's samples pass through.
    equalizer: Arc<EqualizerShared>,
    /// The length of a crossfade, or zero for a gapless hand-over.
    crossfade: Duration,
    loaded: Option<LoadedTrack>,
    /// A track already handed to the device behind the current one, so the two
    /// run together with no gap. Never more than one deep: the application
    /// lines up exactly the track the queue says comes next.
    preloaded: Option<LoadedTrack>,
    /// A track decoded but not yet playing, held for a crossfade.
    prepared: Option<Prepared>,
    /// The crossfade currently running, if any.
    fade: Option<FadeState>,
}

impl AudioEngine {
    /// Opens the default output device and returns a ready engine.
    ///
    /// A failure to open a device is not fatal: the engine keeps working with
    /// no sound so the rest of the application stays usable.
    pub fn open() -> Result<Self, AudioError> {
        Self::open_device(None)
    }

    /// Opens `device`, or the system's default when `None` is given.
    ///
    /// The identifier is one of [`crate::audio::available_devices`]'s; the
    /// settings screen stores exactly that, so the device a user picked is the
    /// device they get.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::DeviceNotFound`] when the named device is not on
    /// this machine, and [`AudioError::OutputDevice`] when it cannot be opened.
    pub fn open_device(device: Option<&str>) -> Result<Self, AudioError> {
        let output = match device {
            Some(id) => Output::open_device(id)?,
            None => Output::open()?,
        };

        let engine = Self::new(output);
        engine.apply_volume();
        Ok(engine)
    }

    /// The state every engine starts in, around an already-open output.
    fn new(output: Output) -> Self {
        Self {
            output: Some(output),
            volume: 1.0,
            equalizer: Arc::new(EqualizerShared::new(EqualizerSettings::default())),
            crossfade: Duration::ZERO,
            loaded: None,
            preloaded: None,
            prepared: None,
            fade: None,
        }
    }

    /// The equalizer settings every track is currently filtered by.
    #[must_use]
    pub fn equalizer(&self) -> EqualizerSettings {
        self.equalizer.snapshot()
    }

    /// Replaces the equalizer settings.
    ///
    /// The change reaches tracks already playing, because each track reads the
    /// settings as it runs rather than copying them when it loads.
    pub fn set_equalizer(&self, settings: EqualizerSettings) {
        self.equalizer.set(settings);
    }

    /// The crossfade length in seconds; zero means tracks change over gaplessly.
    #[must_use]
    pub fn crossfade_seconds(&self) -> f32 {
        self.crossfade.as_secs_f32()
    }

    /// Sets the crossfade length, clamped to the accepted range.
    ///
    /// A change applies to transitions chosen from now on; a track that is
    /// already lined up for a gapless hand-over is left to change over that way,
    /// since its audio is on the device by the time the setting moves.
    pub fn set_crossfade(&mut self, seconds: f32) {
        let seconds = if seconds.is_finite() {
            seconds.clamp(0.0, MAX_CROSSFADE_SECONDS)
        } else {
            0.0
        };
        self.crossfade = Duration::from_secs_f32(seconds);
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

    /// The identifier of the device the engine is playing through.
    #[must_use]
    pub fn output_id(&self) -> Option<&str> {
        self.output.as_ref().and_then(Output::id)
    }

    /// The sample rate the open device negotiated, in hertz.
    #[must_use]
    pub fn output_sample_rate(&self) -> Option<u32> {
        self.output.as_ref().map(Output::sample_rate)
    }

    /// The number of output channels the open device negotiated.
    #[must_use]
    pub fn output_channels(&self) -> Option<u16> {
        self.output.as_ref().map(Output::channel_count)
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
        if self.output.is_none() {
            return Err(AudioError::OutputUnavailable);
        }

        let (source, done, duration) = build_source(path, &self.equalizer)?;

        // A new track cancels whatever transition was in flight: the fade, the
        // prepared track and the gapless hand-over all belonged to the track
        // being replaced.
        self.clear_transition();

        // Replace, rather than append: `clear` drops the old source, so the new
        // track starts immediately instead of after the current one ends.
        if let Some(output) = &self.output {
            output.active().clear();
            output.active().append(source);
            output.active().play();
            output.active().set_volume(self.volume);
        }

        self.loaded = Some(LoadedTrack {
            path: path.to_path_buf(),
            duration,
            finished: false,
            done,
        });
        Ok(duration)
    }

    /// Hands the track that comes next to the engine, ready for the change-over.
    ///
    /// What "ready" means depends on the transition setting. With no crossfade
    /// the track is handed to the device now, behind the current one, so the two
    /// run together with no gap and [`AudioEngine::poll_transition`] reports the
    /// hand-over at the end. With a crossfade the track is decoded but held
    /// back, because it has to *start* while the current track is still playing;
    /// [`AudioEngine::poll_transition`] begins the fade near the end.
    ///
    /// Either way preloading again replaces whatever was waiting, so only the
    /// last track stays lined up.
    ///
    /// # Errors
    ///
    /// As [`AudioEngine::load`]: the file is opened and decoded now, so a file
    /// that will not decode fails here rather than at the end of the current
    /// track.
    pub fn preload(&mut self, path: &Path) -> Result<Option<Duration>, AudioError> {
        if self.output.is_none() {
            return Err(AudioError::OutputUnavailable);
        }

        let (source, done, duration) = build_source(path, &self.equalizer)?;

        if self.crossfade.is_zero() {
            // Gapless: append only. The current track keeps playing, and this
            // one starts on its own the moment the one before it runs out.
            if let Some(output) = &self.output {
                output.active().append(source);
            }
            self.preloaded = Some(LoadedTrack {
                path: path.to_path_buf(),
                duration,
                finished: false,
                done,
            });
        } else {
            // Crossfade: hold it back to be started on the spare deck later.
            self.prepared = Some(Prepared {
                path: path.to_path_buf(),
                duration,
                source,
                done,
            });
        }
        Ok(duration)
    }

    /// Whether a track is loaded and the device is playing it.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        match (&self.output, &self.loaded) {
            (Some(output), Some(loaded)) => !loaded.finished && !output.active().is_paused(),
            _ => false,
        }
    }

    /// Whether a track is loaded but paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        match (&self.output, &self.loaded) {
            (Some(output), Some(loaded)) => !loaded.finished && output.active().is_paused(),
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
            (Some(output), Some(loaded)) if !loaded.finished => output.active().get_pos(),
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
    ///
    /// Both decks are paused: during a crossfade one track is fading in on the
    /// spare one, and pausing only the audible deck would let the other run on
    /// underneath.
    pub fn pause(&self) {
        if let Some(output) = &self.output {
            output.active().pause();
            output.incoming().pause();
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
            output.active().play();
            output.incoming().play();
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
            output.active().clear();
            output.incoming().clear();
        }
        self.loaded = None;
        self.preloaded = None;
        self.prepared = None;
        self.fade = None;
    }

    /// Seeks to `position` within the loaded track.
    ///
    /// A seek cancels a crossfade in progress: the fade is measured from the
    /// position in the track being left, so carrying it across a jump would
    /// finish it at once or never. The track that was fading in is dropped and
    /// the queue lines it up again.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::Seek`] when the format cannot be seeked, and
    /// [`AudioError::OutputUnavailable`] when no device is open.
    pub fn seek(&mut self, position: Duration) -> Result<(), AudioError> {
        if self.output.is_none() {
            return Err(AudioError::OutputUnavailable);
        }
        self.abort_fade();

        let Some(loaded) = &self.loaded else {
            return Ok(());
        };
        let path = loaded.path.clone();
        let Some(output) = &self.output else {
            return Err(AudioError::OutputUnavailable);
        };
        output
            .active()
            .try_seek(position)
            .map_err(|source| AudioError::Seek { path, source })
    }

    /// Reports the end of the current track, handing over to a queued one when
    /// there is one.
    ///
    /// This is called once per UI frame. Three things can come of it:
    ///
    /// - A crossfade that is running has its two gains moved along, and reports
    ///   [`EngineEvent::Advanced`] once the incoming track is the one playing.
    /// - A track nearing its end with a crossfade set starts one, and nothing is
    ///   reported until it finishes.
    /// - A track with another already lined up behind it ends without a pause:
    ///   the device has moved on by itself, and this reports
    ///   [`EngineEvent::Advanced`] naming the track now playing so the queue and
    ///   the interface can catch up. With nothing queued, the track has genuinely
    ///   finished and this reports [`EngineEvent::Finished`].
    ///
    /// Either way a track is reported exactly once, so the caller advances the
    /// queue one track per natural end.
    pub fn poll_transition(&mut self) -> Option<EngineEvent> {
        if self.fade.is_some() {
            return self.poll_fade();
        }
        if self.begin_crossfade() {
            return None;
        }

        let loaded = self.loaded.as_ref()?;
        if loaded.finished || !loaded.done.load(Ordering::SeqCst) {
            return None;
        }

        if let Some(next) = self.preloaded.take() {
            let path = next.path.clone();
            self.loaded = Some(next);
            return Some(EngineEvent::Advanced { path });
        }

        let loaded = self.loaded.as_mut()?;
        loaded.finished = true;
        Some(EngineEvent::Finished {
            path: loaded.path.clone(),
        })
    }

    /// Starts a crossfade if the current track is close enough to its end and a
    /// track is waiting for one.
    ///
    /// Returns whether a fade was started. A track whose length the decoder did
    /// not report cannot be timed, so it falls back to the gapless path; the
    /// same is true of a current track that is paused.
    fn begin_crossfade(&mut self) -> bool {
        if self.crossfade.is_zero() {
            return false;
        }

        let Some(loaded) = &self.loaded else {
            return false;
        };
        let Some(duration) = loaded.duration else {
            return false;
        };
        if loaded.finished {
            return false;
        }
        let Some(output) = &self.output else {
            return false;
        };
        if output.active().is_paused() {
            return false;
        }

        let position = output.active().get_pos();
        if position + self.crossfade < duration {
            return false;
        }

        let Some(prepared) = self.prepared.take() else {
            return false;
        };

        let started_at = output.active().get_pos();
        output.incoming().clear();
        output.incoming().append(prepared.source);
        output.incoming().play();
        output.incoming().set_volume(0.0);

        self.fade = Some(FadeState {
            started_at,
            total: self.crossfade,
            next: LoadedTrack {
                path: prepared.path,
                duration: prepared.duration,
                finished: false,
                done: prepared.done,
            },
        });
        true
    }

    /// Moves a running crossfade along, finishing it when it has run its length.
    fn poll_fade(&mut self) -> Option<EngineEvent> {
        let Some(fade) = &self.fade else {
            return None;
        };

        // The outgoing track can run out before the fade does — a track shorter
        // than the crossfade, or one that ends early. When it does, the fade is
        // over, because there is nothing left to fade out.
        let drained = self
            .loaded
            .as_ref()
            .is_some_and(|loaded| loaded.done.load(Ordering::SeqCst));
        let progress = self.fade_progress(fade);

        let (out_gain, in_gain) = fade_gains(progress);
        if let Some(output) = &self.output {
            output.active().set_volume(self.volume * out_gain);
            output.incoming().set_volume(self.volume * in_gain);
        }

        if drained || progress >= 1.0 {
            return self.complete_crossfade();
        }
        None
    }

    /// How far through the running crossfade playback has reached, in `0.0..=1.0`.
    fn fade_progress(&self, fade: &FadeState) -> f32 {
        let Some(output) = &self.output else {
            return 1.0;
        };
        let total = fade.total.as_secs_f32();
        if total <= 0.0 {
            return 1.0;
        }
        let elapsed = output
            .active()
            .get_pos()
            .saturating_sub(fade.started_at)
            .as_secs_f32();
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// Ends a running crossfade by promoting the incoming track to current.
    fn complete_crossfade(&mut self) -> Option<EngineEvent> {
        let fade = self.fade.take()?;

        if let Some(output) = &mut self.output {
            output.swap();
            // The track that faded out is now the spare: it is silent and its
            // samples are spent, so let it go.
            output.incoming().clear();
        }

        let path = fade.next.path.clone();
        self.loaded = Some(fade.next);
        self.apply_volume();
        Some(EngineEvent::Advanced { path })
    }

    /// Drops a running crossfade without promoting anything.
    ///
    /// The spare deck is silenced and the track it was fading in is forgotten;
    /// the queue is expected to line it up again. Used when playback jumps
    /// somewhere the fade no longer describes.
    fn abort_fade(&mut self) {
        if self.fade.take().is_none() {
            return;
        }
        if let Some(output) = &self.output {
            output.incoming().clear();
        }
        self.prepared = None;
        self.apply_volume();
    }

    /// Forgets every track lined up behind the current one.
    fn clear_transition(&mut self) {
        self.preloaded = None;
        self.prepared = None;
        self.fade = None;
        if let Some(output) = &self.output {
            output.incoming().clear();
        }
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
            equalizer: Arc::new(EqualizerShared::new(EqualizerSettings::default())),
            crossfade: Duration::ZERO,
            loaded: None,
            preloaded: None,
            prepared: None,
            fade: None,
        }
    }

    /// The path of the loaded track if it has already finished.
    fn finished_track_path(&self) -> Option<PathBuf> {
        self.loaded
            .as_ref()
            .filter(|loaded| loaded.finished)
            .map(|loaded| loaded.path.clone())
    }

    /// Pushes the current volume onto the deck, or onto both during a fade.
    fn apply_volume(&self) {
        let Some(output) = &self.output else {
            return;
        };
        match &self.fade {
            Some(fade) => {
                let (out_gain, in_gain) = fade_gains(self.fade_progress(fade));
                output.active().set_volume(self.volume * out_gain);
                output.incoming().set_volume(self.volume * in_gain);
            }
            None => output.active().set_volume(self.volume),
        }
    }
}

/// The (outgoing, incoming) gains of an equal-power crossfade at `progress`.
///
/// The two gains are a cosine and a sine of the same angle, so their squares sum
/// to one and the perceived loudness holds steady across the change. A linear
/// fade would dip by about 3 dB in the middle, which is heard as a soft spot.
fn fade_gains(progress: f32) -> (f32, f32) {
    let angle = progress.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
    (angle.cos(), angle.sin())
}

/// A decoded track ready for the device: the samples, the flag they set when
/// drained, and the track's length when the decoder knew one.
type DecodedTrack = (Box<dyn Source + Send>, Arc<AtomicBool>, Option<Duration>);

/// Decodes `path` and wraps it so the equalizer colours it and the engine learns
/// when it is drained.
///
/// The source is boxed because a crossfade holds a decoded track before handing
/// it to a player, and that field has to name one type whatever the track is.
fn build_source(path: &Path, equalizer: &Arc<EqualizerShared>) -> Result<DecodedTrack, AudioError> {
    let decoder = decode(path)?;
    let duration = decoder.total_duration();
    let (source, done) = Signalled::new(Equalized::new(decoder, Arc::clone(equalizer)));
    Ok((Box::new(source), done, duration))
}

/// Opens and decodes `path`, translating the decoder's failures into the
/// engine's own error type.
fn decode(path: &Path) -> Result<Decoder<File>, AudioError> {
    let file = File::open(path).map_err(|source| AudioError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    Decoder::new(file).map_err(|source| match source {
        DecoderError::UnrecognizedFormat => AudioError::UnsupportedFormat {
            path: path.to_path_buf(),
        },
        source => AudioError::Decode {
            path: path.to_path_buf(),
            source,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::num::NonZero;

    use rodio::Player;
    use rodio::buffer::SamplesBuffer;

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
            engine.preload(Path::new("/music/next.mp3")),
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
    fn the_completion_signal_fires_when_the_samples_run_out() {
        let samples = SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(44_100).unwrap(),
            vec![0.0f32, 0.5, -0.5, 0.25],
        );
        let (mut source, done) = Signalled::new(samples);

        assert!(
            !done.load(Ordering::SeqCst),
            "a source with samples left has not been drained"
        );
        while source.next().is_some() {}
        assert!(
            done.load(Ordering::SeqCst),
            "the signal fires the moment the samples are gone"
        );
    }

    /// A loaded track whose completion signal is already set, as the audio
    /// thread would have left it after draining the track.
    fn drained(path: &str) -> LoadedTrack {
        LoadedTrack {
            path: PathBuf::from(path),
            duration: None,
            finished: false,
            done: Arc::new(AtomicBool::new(true)),
        }
    }

    /// A track lined up to follow, whose own completion signal is not yet set.
    fn waiting(path: &str, duration: Duration) -> LoadedTrack {
        LoadedTrack {
            path: PathBuf::from(path),
            duration: Some(duration),
            finished: false,
            done: Arc::new(AtomicBool::new(false)),
        }
    }

    #[test]
    fn nothing_is_reported_without_a_loaded_track() {
        let mut engine = AudioEngine::without_device();

        assert_eq!(engine.poll_transition(), None);
    }

    #[test]
    fn a_drained_track_with_nothing_queued_finishes_once() {
        let mut engine = AudioEngine::without_device();
        engine.loaded = Some(drained("/music/a.mp3"));

        assert_eq!(
            engine.poll_transition(),
            Some(EngineEvent::Finished {
                path: PathBuf::from("/music/a.mp3")
            })
        );
        assert_eq!(
            engine.poll_transition(),
            None,
            "a finished track is reported exactly once"
        );
    }

    #[test]
    fn a_drained_track_hands_over_to_the_one_lined_up_behind_it() {
        let mut engine = AudioEngine::without_device();
        engine.loaded = Some(drained("/music/a.mp3"));
        engine.preloaded = Some(waiting("/music/b.mp3", Duration::from_secs(120)));

        assert_eq!(
            engine.poll_transition(),
            Some(EngineEvent::Advanced {
                path: PathBuf::from("/music/b.mp3")
            })
        );
        assert_eq!(engine.current_path(), Some(Path::new("/music/b.mp3")));
        assert_eq!(
            engine.duration(),
            Some(Duration::from_secs(120)),
            "the track that took over is the one now reported"
        );
        assert_eq!(
            engine.poll_transition(),
            None,
            "the hand-over is not a finish, and it has already happened"
        );
    }

    #[test]
    fn two_sources_queued_on_one_player_run_back_to_back() {
        // This is the mechanism the engine rests on, exercised without a sound
        // card: two tracks appended to one player, the samples pulled from it
        // exactly as the audio callback would. The second must follow the first
        // with nothing — not even a silent sample — between them.
        let (player, mut output) = Player::new();
        let rate = NonZero::new(44_100).unwrap();
        let channels = NonZero::new(2).unwrap();

        let (first, first_done) =
            Signalled::new(SamplesBuffer::new(channels, rate, vec![1.0f32; 8]));
        let (second, second_done) =
            Signalled::new(SamplesBuffer::new(channels, rate, vec![2.0f32; 8]));
        player.append(first);
        player.append(second);

        let samples: Vec<Sample> = output.by_ref().take(16).collect();

        assert_eq!(&samples[..8], &[1.0f32; 8], "the first track plays first");
        assert_eq!(
            &samples[8..],
            &[2.0f32; 8],
            "the second follows with no gap"
        );
        assert!(
            first_done.load(Ordering::SeqCst),
            "the first track reports itself drained as the second begins"
        );
        assert!(
            !second_done.load(Ordering::SeqCst),
            "the second track has not ended yet"
        );
    }

    /// A decoded track held for a crossfade, as [`AudioEngine::preload`] leaves
    /// one when a crossfade is set.
    fn prepared(path: &str, duration: Duration) -> Prepared {
        let samples = SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(44_100).unwrap(),
            vec![0.0f32; 8],
        );
        Prepared {
            path: PathBuf::from(path),
            duration: Some(duration),
            source: Box::new(samples),
            done: Arc::new(AtomicBool::new(false)),
        }
    }

    #[test]
    fn the_crossfade_gains_keep_the_loudness_steady() {
        // At each end one track is silent and the other is whole.
        assert_eq!(fade_gains(0.0), (1.0, 0.0));
        let (out, into) = fade_gains(1.0);
        assert!(out.abs() < 1e-6, "the outgoing track ends silent: {out}");
        assert!(
            (into - 1.0).abs() < 1e-6,
            "the incoming one ends whole: {into}"
        );

        // Equal power across the fade: the squares of the two gains sum to one,
        // so the blend does not dip in the middle.
        for step in 0..=10 {
            let (out, into) = fade_gains(step as f32 / 10.0);
            assert!(
                (out * out + into * into - 1.0).abs() < 1e-5,
                "the gains at {step}/10 do not hold power: {out}, {into}"
            );
        }
    }

    #[test]
    fn the_crossfade_length_is_clamped_to_what_the_engine_accepts() {
        let mut engine = AudioEngine::without_device();
        assert_eq!(engine.crossfade_seconds(), 0.0, "gapless by default");

        engine.set_crossfade(99.0);
        assert_eq!(engine.crossfade_seconds(), MAX_CROSSFADE_SECONDS);

        engine.set_crossfade(-2.0);
        assert_eq!(engine.crossfade_seconds(), 0.0);

        engine.set_crossfade(f32::NAN);
        assert_eq!(engine.crossfade_seconds(), 0.0, "nonsense becomes gapless");
    }

    #[test]
    fn the_engine_hands_back_the_equalizer_settings_it_was_given() {
        let engine = AudioEngine::without_device();
        assert_eq!(engine.equalizer(), EqualizerSettings::default());

        let settings = EqualizerSettings {
            enabled: true,
            preamp_db: -3.0,
            bands: [1.0, 2.0, 3.0, 4.0, 5.0, -1.0, -2.0, -3.0, -4.0, -5.0],
        };
        engine.set_equalizer(settings.clone());

        assert_eq!(
            engine.equalizer(),
            settings,
            "the current settings are kept"
        );
    }

    #[test]
    fn without_a_device_a_prepared_track_is_not_crossfaded() {
        // With no device there is nothing to fade on, so the change-over falls
        // back to the finished-track path and the prepared audio is left alone.
        let mut engine = AudioEngine::without_device();
        engine.set_crossfade(5.0);
        engine.loaded = Some(drained("/music/a.mp3"));
        engine.prepared = Some(prepared("/music/b.mp3", Duration::from_secs(120)));

        assert_eq!(
            engine.poll_transition(),
            Some(EngineEvent::Finished {
                path: PathBuf::from("/music/a.mp3")
            })
        );
        assert!(
            engine.prepared.is_some(),
            "nothing promoted the prepared track"
        );
    }
}
