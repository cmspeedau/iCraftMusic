#![allow(dead_code)]

use std::{fs::File, path::Path, time::Duration};

use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TrackMetadata {
    pub duration: Option<Duration>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub codec: Option<String>,
}

impl TrackMetadata {
    pub fn empty() -> Self {
        Self {
            duration: None,
            sample_rate: None,
            channels: None,
            codec: None,
        }
    }
}

pub fn read_metadata(path: &Path) -> Result<TrackMetadata, String> {
    let file = File::open(path).map_err(|error| format!("Could not open file: {error}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(extension);
    }

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|error| format!("Could not probe format: {error}"))?;

    let format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "No audio track found".to_string())?;

    let codec = Some(format!("{:?}", track.codec_params.codec));
    let sample_rate = track.codec_params.sample_rate;
    let channels = track.codec_params.channels.map(|c| c.count() as u16);

    // Try to get duration from codec params or format
    let duration = track.codec_params.n_frames.and_then(|frames| {
        track
            .codec_params
            .sample_rate
            .map(|rate| Duration::from_secs_f64(frames as f64 / rate as f64))
    });

    Ok(TrackMetadata {
        duration,
        sample_rate,
        channels,
        codec,
    })
}

#[cfg(test)]
mod tests {
    use super::TrackMetadata;

    #[test]
    fn empty_metadata_has_no_values() {
        let meta = TrackMetadata::empty();
        assert!(meta.duration.is_none());
        assert!(meta.sample_rate.is_none());
        assert!(meta.channels.is_none());
        assert!(meta.codec.is_none());
    }

    #[test]
    fn read_metadata_fails_for_missing_file() {
        let result = super::read_metadata(std::path::Path::new("/nonexistent/file.mp3"));
        assert!(result.is_err());
    }

    #[test]
    fn read_metadata_fails_for_unsupported_file() {
        // Create a temp text file
        let dir = std::env::temp_dir();
        let path = dir.join("test_not_audio.txt");
        std::fs::write(&path, "this is not audio").unwrap();
        let result = super::read_metadata(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }
}
