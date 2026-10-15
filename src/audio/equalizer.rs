//! The graphic equalizer.
//!
//! The equalizer is a fixed bank of peaking filters applied to every track
//! before it reaches the output device, so it colours whatever is playing
//! however the user set it. Two halves are kept apart on purpose:
//!
//! - [`EqualizerSettings`] is the value the application stores and the settings
//!   screen edits. It is plain data: ten band gains, a preamp and an on/off
//!   switch.
//! - [`Equalized`] is the audio thread's view. It wraps one track's samples,
//!   holds the filter state, and re-reads the settings when they change, so
//!   moving a slider is audible without restarting the track.
//!
//! The filters are second-order RBJ peaking biquads, the same recipe the
//! "Audio EQ Cookbook" gives. A band at zero gain is an exact identity, which is
//! what lets the flat setting cost nothing but arithmetic.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use rodio::source::SeekError;
use rodio::{ChannelCount, Sample, SampleRate, Source};

/// The number of bands in the equalizer.
pub const BANDS: usize = 10;

/// The centre frequency of each band, in hertz.
///
/// Ten bands roughly an octave apart, from the bottom of what most speakers
/// reproduce to the top of what most ears hear. They are fixed: a graphic
/// equalizer trades flexibility for a control per band, and the labels are the
/// frequencies themselves.
pub const BAND_FREQUENCIES: [u32; BANDS] =
    [31, 62, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

/// The largest gain any band or the preamp accepts, in decibels.
pub const MAX_GAIN_DB: f32 = 12.0;

/// The smallest gain any band or the preamp accepts, in decibels.
pub const MIN_GAIN_DB: f32 = -12.0;

/// The bandwidth of each band, as a filter Q.
///
/// About one octave wide, which is what makes adjacent bands overlap a little
/// rather than leaving gaps between them.
const BAND_Q: f32 = 1.4;

/// The equalizer as the user sets it and the application stores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EqualizerSettings {
    /// Whether the equalizer colours the sound at all.
    pub enabled: bool,
    /// A gain applied to every band's input, in decibels.
    ///
    /// Boosting bands can push the signal past full scale; the preamp is the
    /// control that pulls it back, exactly as a hardware equalizer's does.
    pub preamp_db: f32,
    /// One gain per band, in decibels, in [`BAND_FREQUENCIES`] order.
    pub bands: [f32; BANDS],
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            preamp_db: 0.0,
            bands: [0.0; BANDS],
        }
    }
}

impl EqualizerSettings {
    /// Clamp every value to the range the filters can render.
    ///
    /// Values outside the range are folded in rather than rejected, because a
    /// hand-edited or older settings file should load as something the player
    /// can use. A value that is not a number becomes zero — the flat setting —
    /// rather than poisoning the filter arithmetic.
    pub fn sanitize(&mut self) {
        self.preamp_db = clamp_gain(self.preamp_db);
        for gain in &mut self.bands {
            *gain = clamp_gain(*gain);
        }
    }

    /// Whether every band and the preamp are at zero.
    #[must_use]
    pub fn is_flat(&self) -> bool {
        self.preamp_db == 0.0 && self.bands.iter().all(|gain| *gain == 0.0)
    }

    /// The band gains of the named preset, if there is one.
    #[must_use]
    pub fn preset(name: &str) -> Option<Self> {
        PRESETS
            .iter()
            .find(|preset| preset.name == name)
            .map(|preset| Self {
                enabled: true,
                preamp_db: 0.0,
                bands: preset.gains,
            })
    }
}

/// A named set of band gains offered in the settings screen.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// What the button says.
    pub name: &'static str,
    /// One gain per band, in [`BAND_FREQUENCIES`] order.
    pub gains: [f32; BANDS],
}

/// The equalizer presets the settings screen offers.
///
/// They are starting points, not fixed modes: choosing one copies its gains
/// into the sliders, which can then be moved freely.
pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Flat",
        gains: [0.0; BANDS],
    },
    Preset {
        name: "Bass Booster",
        gains: [7.0, 6.0, 4.5, 2.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Preset {
        name: "Treble Booster",
        gains: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.5, 4.5, 6.0, 7.0],
    },
    Preset {
        name: "Rock",
        gains: [5.0, 4.0, 2.5, -1.0, -2.0, -1.0, 1.0, 3.0, 4.0, 5.0],
    },
    Preset {
        name: "Pop",
        gains: [-1.0, 0.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0, -1.0, -1.0],
    },
    Preset {
        name: "Jazz",
        gains: [3.0, 2.0, 1.0, 2.0, -1.0, -1.0, 0.0, 1.0, 2.0, 3.0],
    },
    Preset {
        name: "Classical",
        gains: [4.0, 3.0, 2.0, 0.0, -1.0, -1.0, 0.0, 2.0, 3.0, 4.0],
    },
    Preset {
        name: "Vocal",
        gains: [-2.0, -1.0, 0.0, 2.0, 4.0, 4.0, 3.0, 1.0, 0.0, -1.0],
    },
];

/// The settings every track's filter reads, and the version that marks them
/// changed.
///
/// The audio thread never waits on the interface: it holds one of these behind
/// an [`Arc`], reads the version cheaply, and copies the settings only when the
/// version moves. A slider that is dragged a hundred times a second therefore
/// costs a hundred copies, not a hundred filter rebuilds on every missed ring.
#[derive(Debug)]
pub struct EqualizerShared {
    settings: Mutex<EqualizerSettings>,
    version: AtomicU64,
}

impl EqualizerShared {
    /// Wrap `settings` for handing to track filters.
    #[must_use]
    pub fn new(settings: EqualizerSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
            version: AtomicU64::new(1),
        }
    }

    /// Replace the settings, marking them changed for every filter.
    ///
    /// An identical setting is ignored, so a slider that ends where it started
    /// does not make the audio thread rebuild its filters for nothing.
    pub fn set(&self, settings: EqualizerSettings) {
        let mut current = self.lock();
        if *current == settings {
            return;
        }
        *current = settings;
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    /// A copy of the current settings.
    #[must_use]
    pub fn snapshot(&self) -> EqualizerSettings {
        self.lock().clone()
    }

    /// The version a filter compares against to notice a change.
    fn version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }

    /// Lock the settings, recovering from a poisoned lock.
    ///
    /// The lock only ever guards a copy of plain data, so a panic elsewhere
    /// cannot leave it inconsistent; recovering keeps one bad frame from
    /// silencing the equalizer for the rest of the session.
    fn lock(&self) -> std::sync::MutexGuard<'_, EqualizerSettings> {
        self.settings
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

/// One track's samples with the equalizer applied.
///
/// The filter state is per track, so changing tracks does not carry a transient
/// from the last one.
pub struct Equalized<S> {
    inner: S,
    shared: Arc<EqualizerShared>,
    /// The last settings version this filter rendered; zero until the first
    /// sample forces a copy.
    seen: u64,
    /// The sample rate the filters were built for, or zero before the first
    /// sample.
    sample_rate: u32,
    enabled: bool,
    preamp: f32,
    filters: [PeakingBiquad; BANDS],
}

impl<S> Equalized<S> {
    /// Wrap `inner` so the settings `shared` holds colour its samples.
    #[must_use]
    pub fn new(inner: S, shared: Arc<EqualizerShared>) -> Self {
        Self {
            inner,
            shared,
            seen: 0,
            sample_rate: 0,
            enabled: false,
            preamp: 1.0,
            filters: [PeakingBiquad::identity(); BANDS],
        }
    }

    /// Copy the settings again when they have changed, rebuilding the filters.
    fn refresh(&mut self) {
        let version = self.shared.version();
        if version == self.seen {
            return;
        }
        self.seen = version;

        let settings = self.shared.snapshot();
        self.enabled = settings.enabled;
        self.preamp = db_to_gain(settings.preamp_db);
        self.rebuild(&settings.bands);
    }

    /// Rebuild each band's coefficients for the current rate and gains.
    fn rebuild(&mut self, gains: &[f32; BANDS]) {
        if self.sample_rate == 0 {
            return;
        }
        let rate = self.sample_rate as f32;
        for (filter, (frequency, gain)) in self
            .filters
            .iter_mut()
            .zip(BAND_FREQUENCIES.iter().zip(gains.iter()))
        {
            *filter = PeakingBiquad::peaking(rate, *frequency as f32, BAND_Q, *gain);
        }
    }
}

impl<S: Source> Iterator for Equalized<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        // The rate can only change at a span boundary, but it is cheap to ask,
        // and rebuilding here is what keeps a filter correct across a track
        // that changes rate partway through.
        let rate = self.inner.sample_rate().get();
        if rate != self.sample_rate {
            self.sample_rate = rate;
            self.seen = 0;
        }
        self.refresh();

        let sample = self.inner.next()?;
        if !self.enabled {
            return Some(sample);
        }

        let mut value = sample * self.preamp;
        for filter in &mut self.filters {
            value = filter.process(value);
        }
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Source> Source for Equalized<S> {
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

/// A second-order peaking biquad, in transposed direct form II.
///
/// The transposed form is used because it needs two state variables rather than
/// four and behaves better when the coefficients are close to degenerate — which
/// ten cascaded bands at low gain produces.
#[derive(Debug, Clone, Copy)]
struct PeakingBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    s1: f32,
    s2: f32,
}

impl PeakingBiquad {
    /// A filter that passes its input through unchanged.
    const fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Build a peaking filter centred on `frequency` with the given Q and gain.
    ///
    /// Follows the Audio EQ Cookbook's peaking-EQ recipe. At a gain of zero the
    /// coefficients reduce to an exact identity, which is what makes a flat
    /// band free of colouring.
    fn peaking(rate: f32, frequency: f32, q: f32, gain_db: f32) -> Self {
        if gain_db == 0.0 {
            return Self::identity();
        }

        let a = 10f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * frequency / rate;
        let (sin_omega, cos_omega) = omega.sin_cos();
        let alpha = sin_omega / (2.0 * q);
        let a0 = 1.0 + alpha / a;

        Self {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos_omega) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos_omega) / a0,
            a2: (1.0 - alpha / a) / a0,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Filter one sample.
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.s1;
        self.s1 = self.b1 * x - self.a1 * y + self.s2;
        self.s2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// A decibel value as a linear amplitude multiplier.
fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// Clamp a gain, turning anything not finite into silence-neutral zero.
fn clamp_gain(db: f32) -> f32 {
    if db.is_finite() {
        db.clamp(MIN_GAIN_DB, MAX_GAIN_DB)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::f32::consts::PI;

    use rodio::buffer::SamplesBuffer;

    /// A mono source of `samples` at `rate`, for feeding the filter.
    fn tone(rate: u32, samples: Vec<f32>) -> SamplesBuffer {
        SamplesBuffer::new(
            std::num::NonZero::new(1).unwrap(),
            std::num::NonZero::new(rate).unwrap(),
            samples,
        )
    }

    /// A sine at `frequency`, `rate` hertz, `count` mono samples.
    fn sine(rate: u32, frequency: f32, count: usize) -> Vec<f32> {
        (0..count)
            .map(|n| (2.0 * PI * frequency * n as f32 / rate as f32).sin())
            .collect()
    }

    /// The energy of a signal, skipping a settling prefix.
    fn energy(samples: &[f32]) -> f32 {
        let settle = samples.len() / 4;
        samples[settle..].iter().map(|s| s * s).sum()
    }

    /// Run `input` through a one-band equalizer and return what comes out.
    fn through_eq(input: Vec<f32>, gain_db: f32, preamp_db: f32) -> Vec<f32> {
        let mut settings = EqualizerSettings {
            enabled: true,
            preamp_db,
            bands: [0.0; BANDS],
        };
        // Put the chosen gain on the band nearest 1 kHz, the tone's frequency.
        settings.bands[5] = gain_db;

        let shared = Arc::new(EqualizerShared::new(settings));
        let source = Equalized::new(tone(48_000, input), shared);
        source.collect()
    }

    #[test]
    fn defaults_are_flat_and_off() {
        let settings = EqualizerSettings::default();

        assert!(!settings.enabled);
        assert_eq!(settings.preamp_db, 0.0);
        assert!(settings.bands.iter().all(|gain| *gain == 0.0));
        assert!(settings.is_flat());
    }

    #[test]
    fn sanitize_folds_out_of_range_and_nonsense_values_in() {
        let mut settings = EqualizerSettings {
            enabled: true,
            preamp_db: 40.0,
            bands: [f32::NAN; BANDS],
        };
        settings.bands[0] = -40.0;

        settings.sanitize();

        assert_eq!(settings.preamp_db, MAX_GAIN_DB);
        assert_eq!(settings.bands[0], MIN_GAIN_DB);
        assert!(settings.bands[1].is_finite(), "NaN becomes a usable gain");
        assert_eq!(settings.bands[1], 0.0);
    }

    #[test]
    fn a_disabled_equalizer_passes_samples_untouched() {
        let input = sine(48_000, 1_000.0, 4_800);
        let settings = EqualizerSettings {
            enabled: false,
            preamp_db: 12.0,
            bands: [12.0; BANDS],
        };
        let shared = Arc::new(EqualizerShared::new(settings));
        let mut source = Equalized::new(tone(48_000, input.clone()), shared);

        let output: Vec<f32> = std::iter::from_fn(|| source.next()).collect();
        assert_eq!(output, input, "an off equalizer changes nothing");
    }

    #[test]
    fn a_flat_band_is_an_identity() {
        let input = sine(48_000, 1_000.0, 4_800);
        let output = through_eq(input.clone(), 0.0, 0.0);

        for (before, after) in input.iter().zip(output.iter()) {
            assert!(
                (before - after).abs() < 1e-5,
                "a zero-gain band should not colour the sound: {before} vs {after}"
            );
        }
    }

    #[test]
    fn a_boost_raises_the_band_and_a_cut_lowers_it() {
        let input = sine(48_000, 1_000.0, 20_000);
        let flat = energy(&through_eq(input.clone(), 0.0, 0.0));
        let boosted = energy(&through_eq(input.clone(), 9.0, 0.0));
        let cut = energy(&through_eq(input, -9.0, 0.0));

        assert!(
            boosted > flat * 2.0,
            "a +9 dB boost should raise the band's energy: {boosted} vs {flat}"
        );
        assert!(
            cut < flat * 0.5,
            "a -9 dB cut should lower it: {cut} vs {flat}"
        );
    }

    #[test]
    fn the_preamp_scales_the_whole_signal() {
        let input = sine(48_000, 1_000.0, 4_800);
        let quiet = energy(&through_eq(input.clone(), 0.0, -6.0));
        let full = energy(&through_eq(input.clone(), 0.0, 0.0));
        let loud = energy(&through_eq(input, 0.0, 6.0));

        // -6 dB is half the amplitude, a quarter the energy; +6 dB the reverse.
        assert!(
            (quiet / full - 0.25).abs() < 0.02,
            "the preamp should scale energy by its square: {}",
            quiet / full
        );
        assert!(
            (loud / full - 4.0).abs() < 0.05,
            "and the same going up: {}",
            loud / full
        );
    }

    #[test]
    fn a_band_far_from_the_tone_leaves_it_alone() {
        // A 1 kHz tone through a boost on the 31 Hz band should be untouched.
        let mut settings = EqualizerSettings {
            enabled: true,
            ..EqualizerSettings::default()
        };
        settings.bands[0] = 12.0;

        let shared = Arc::new(EqualizerShared::new(settings));
        let mut source = Equalized::new(tone(48_000, sine(48_000, 1_000.0, 20_000)), shared);
        let output: Vec<f32> = std::iter::from_fn(|| source.next()).collect();

        let reference = through_eq(sine(48_000, 1_000.0, 20_000), 0.0, 0.0);
        let ratio = energy(&output) / energy(&reference);
        assert!(
            (ratio - 1.0).abs() < 0.05,
            "a distant band should barely move a tone: {ratio}"
        );
    }

    #[test]
    fn changing_the_settings_is_heard_without_restarting() {
        let input = sine(48_000, 1_000.0, 20_000);
        let shared = Arc::new(EqualizerShared::new(EqualizerSettings::default()));
        let mut source = Equalized::new(tone(48_000, input), Arc::clone(&shared));

        // Off: the first half passes through.
        let quiet: Vec<f32> = std::iter::from_fn(|| source.next()).take(8_000).collect();
        // Turn on a big boost mid-stream and keep pulling.
        let mut boosted = EqualizerSettings {
            enabled: true,
            ..EqualizerSettings::default()
        };
        boosted.bands[5] = 12.0;
        shared.set(boosted);
        let loud: Vec<f32> = std::iter::from_fn(|| source.next()).take(8_000).collect();

        assert!(
            energy(&loud) > energy(&quiet) * 2.0,
            "turning the equalizer on mid-track should be audible"
        );
    }

    #[test]
    fn setting_identical_values_does_not_bump_the_version() {
        let shared = EqualizerShared::new(EqualizerSettings::default());
        let before = shared.version();

        shared.set(EqualizerSettings::default());
        assert_eq!(shared.version(), before, "no change, no rebuild");

        shared.set(EqualizerSettings {
            enabled: true,
            ..EqualizerSettings::default()
        });
        assert_eq!(shared.version(), before + 1, "a real change rebuilds once");
    }

    #[test]
    fn every_preset_is_named_and_within_range() {
        for preset in PRESETS {
            assert!(!preset.name.is_empty());
            for gain in preset.gains {
                assert!(
                    (MIN_GAIN_DB..=MAX_GAIN_DB).contains(&gain),
                    "{} has an out-of-range gain {gain}",
                    preset.name
                );
            }
        }
    }

    #[test]
    fn a_preset_can_be_looked_up_by_name() {
        let rock = EqualizerSettings::preset("Rock").expect("Rock is a preset");

        assert!(rock.enabled, "choosing a preset turns the equalizer on");
        assert_eq!(rock.bands, EqualizerSettings::preset("Rock").unwrap().bands);
        assert!(EqualizerSettings::preset("Nonsense").is_none());
    }
}
