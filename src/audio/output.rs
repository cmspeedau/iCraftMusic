//! The one audio output device the application plays through.
//!
//! Opening a device is fallible and environment dependent, so it is kept in a
//! small type of its own: everything above the device can be exercised on a
//! machine with no sound card.

use std::str::FromStr;

use rodio::cpal;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

use super::AudioError;

/// An output device the user can choose to play through.
///
/// The identifier is what gets remembered between runs — names are not unique
/// across hosts and can change with the locale — while the name is what the
/// settings screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    /// Stable identifier the backend reports, e.g. `alsa:default`.
    pub id: String,
    /// Human-readable name, e.g. `Built-in Audio Analog Stereo`.
    pub name: String,
    /// Whether this is the system's default output device.
    pub is_default: bool,
}

/// Every output device the system offers, the default first.
///
/// A machine with no audio at all, or a backend that refuses to enumerate, has
/// no devices — an empty list rather than a failure, because "there is nothing
/// to choose" is an answer the settings screen can show.
#[must_use]
pub fn available_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let default = host.default_output_device();

    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };

    let mut listed: Vec<AudioDevice> = devices
        .filter_map(|device| {
            let id = device.id().ok()?.to_string();
            let name = device
                .description()
                .map(|description| description.name().to_owned())
                .unwrap_or_else(|_| id.clone());

            Some(AudioDevice {
                id,
                name,
                is_default: false,
            })
        })
        .collect();

    if let Some(default) = default
        && let Ok(default_id) = default.id()
    {
        let default_id = default_id.to_string();
        for device in &mut listed {
            device.is_default = device.id == default_id;
        }
    }

    // The system's default plays first in the list: it is the choice that
    // follows the desktop's own output setting, which is what most users want.
    listed.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| left.name.cmp(&right.name))
    });

    listed
}

/// An open output device together with the player bound to its mixer.
///
/// The field order matters: the player is dropped first, so queued sources are
/// released before the underlying stream is torn down.
pub struct Output {
    player: Player,
    device: MixerDeviceSink,
    /// The device this output was opened through, as [`available_devices`]
    /// names it. `None` when there was nothing to enumerate.
    id: Option<String>,
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
        let id = current_device_id();

        Ok(Self { player, device, id })
    }

    /// Opens the output device named by `id`.
    ///
    /// This is what makes a saved preference mean anything: the device the user
    /// chose is opened again next run rather than whatever the desktop happens
    /// to call its default.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::DeviceNotFound`] when no device carries that
    /// identifier — it was unplugged, or the backend renamed it — and
    /// [`AudioError::OutputDevice`] when the device is there but cannot be
    /// opened.
    pub fn open_device(id: &str) -> Result<Self, AudioError> {
        let not_found = || AudioError::DeviceNotFound {
            device: id.to_owned(),
        };

        let device_id = cpal::DeviceId::from_str(id).map_err(|_| not_found())?;
        let device = cpal::default_host()
            .device_by_id(&device_id)
            .ok_or_else(not_found)?;

        let sink = DeviceSinkBuilder::from_device(device)
            .and_then(DeviceSinkBuilder::open_stream)
            .map_err(|source| AudioError::OutputDevice { source })?;

        let player = Player::connect_new(sink.mixer());

        Ok(Self {
            player,
            device: sink,
            id: Some(id.to_owned()),
        })
    }

    /// The identifier of the device this output is playing through.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
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

/// The identifier of the system's default output device, when it has one.
///
/// Used to record which device a default-opened output is playing through, so
/// the settings screen can say what is actually in use rather than what the
/// preference says.
fn current_device_id() -> Option<String> {
    cpal::default_host()
        .default_output_device()?
        .id()
        .ok()
        .map(|id| id.to_string())
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

    #[test]
    fn every_listed_device_is_identified_and_at_most_one_is_the_default() {
        let devices = available_devices();

        for device in &devices {
            assert!(!device.id.is_empty(), "a device with no id: {device:?}");
            assert!(!device.name.is_empty(), "a device with no name: {device:?}");
        }

        assert!(
            devices.iter().filter(|device| device.is_default).count() <= 1,
            "the list claims two default devices: {devices:?}"
        );

        // The default, when there is one, is what the list leads with.
        if let Some(first) = devices.first()
            && devices.iter().any(|device| device.is_default)
        {
            assert!(first.is_default, "the default device should come first");
        }
    }

    #[test]
    fn a_device_that_is_not_there_is_reported_by_name() {
        // Well-formed enough to parse and absent on every machine: this is the
        // path a saved preference takes when the interface was unplugged.
        let error = match Output::open_device("nonexistent-host:no-such-device") {
            Ok(_) => panic!("there is no such device"),
            Err(error) => error,
        };
        assert!(
            matches!(&error, AudioError::DeviceNotFound { device } if device == "nonexistent-host:no-such-device"),
            "a missing device must be named: {error:?}"
        );
        assert_eq!(error.path(), None);
        assert!(!error.is_file_problem());
    }
}
