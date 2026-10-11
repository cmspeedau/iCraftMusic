//! The one audio output device the application plays through.
//!
//! Opening a device is fallible and environment dependent, so it is kept in a
//! small type of its own: everything above the device can be exercised on a
//! machine with no sound card.

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

use super::AudioError;

/// An open output device together with the player bound to its mixer.
///
/// The field order matters: the player is dropped first, so queued sources are
/// released before the underlying stream is torn down.
pub struct Output {
    player: Player,
    device: MixerDeviceSink,
}

impl Output {
    /// Opens the system's default output device.
    ///
    /// Returns [`AudioError::OutputDevice`] when there is no usable device, for
    /// example on a headless machine or when another application holds the
    /// device exclusively.
    pub fn open() -> Result<Self, AudioError> {
        let device = DeviceSinkBuilder::open_default_sink()
            .map_err(|source| AudioError::OutputDevice { source })?;
        let player = Player::connect_new(device.mixer());
        Ok(Self { player, device })
    }

    /// The player bound to this device's mixer.
    ///
    /// One player is enough for gapless track changes: sources are appended to
    /// it and it plays them in order.
    #[must_use]
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// The sample rate the device negotiated, in hertz.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.device.config().sample_rate().get()
    }

    /// The number of output channels the device negotiated.
    #[must_use]
    pub fn channel_count(&self) -> u16 {
        self.device.config().channel_count().get()
    }

    /// A short description of the negotiated stream, e.g. `48.0 kHz · 2 ch`.
    ///
    /// This is what the settings screen shows so the user can tell that an
    /// output device really was opened.
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "{:.1} kHz · {} ch",
            self.sample_rate() as f64 / 1000.0,
            self.channel_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_a_device_either_succeeds_or_reports_a_device_error() {
        // A CI machine may have no sound card, so both outcomes are valid; what
        // matters is that a failure is typed as a device problem rather than
        // panicking.
        match Output::open() {
            Ok(output) => {
                assert!(output.sample_rate() > 0);
                assert!(output.channel_count() > 0);
                assert!(
                    output.description().contains("kHz"),
                    "the description should be human readable: {}",
                    output.description()
                );
            }
            Err(error) => {
                assert!(
                    !error.is_file_problem(),
                    "an open failure is not a file problem"
                );
                assert_eq!(error.path(), None);
            }
        }
    }
}
