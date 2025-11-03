// IO crate

use std::fs::File;

use symphonia::core::{
    codecs::{Decoder, DecoderOptions},
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::TimeBase,
};

/// Represents an opened audio file with all necessary information for playback and analysis
pub struct AudioFile {
    pub format: Box<dyn FormatReader>,
    pub decoder: Box<dyn Decoder>,
    pub track_id: u32,
    pub time_base: TimeBase,
    pub sample_rate: u32,
    pub channels: u8,
}

impl AudioFile {
    /// Opens an audio file and returns an AudioFile struct containing decoder and format info
    pub fn open(path: &std::path::PathBuf) -> Result<Self, String> {
        let file: File = File::open(path).map_err(|e| format!("failed to open media: {}", e))?;
        let mss: MediaSourceStream = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint for which decoder to use based on the file's extension
        let mut hint: Hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext); // e.g., "mp3" or "wav"
        }

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| format!("failed to probe media: {}", e))?;

        // Get the format reader yielded by the probe operation.
        let format = probed.format;

        // Get the default track.
        let track = format
            .default_track()
            .ok_or("no audio tracks found in media")?;

        // Get codec parameters
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.ok_or("sample rate not found")?;
        let channels = codec_params
            .channels
            .ok_or("channel count not found")?
            .count() as u8;
        let time_base = codec_params.time_base.ok_or("time base not found")?;

        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs()
            .make(codec_params, &decoder_opts)
            .map_err(|e| format!("failed to create decoder: {}", e))?;

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        Ok(AudioFile {
            format,
            decoder,
            track_id,
            time_base,
            sample_rate,
            channels,
        })
    }
}
