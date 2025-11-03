// Runtime crate - Audio playback functionality

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use mogbox_io::AudioFile;
use std::sync::{Arc, Mutex};
use symphonia::core::audio::Signal;

/// Represents an audio player that handles playback of audio files
pub struct AudioPlayer {
    _stream: Stream,
}

impl AudioPlayer {
    /// Creates a new audio player and starts playing the audio file
    pub fn play(mut audio_file: AudioFile) -> Result<Self, String> {
        // Get the default host and device
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device available")?;

        // Get the configuration that matches the audio file
        let config = StreamConfig {
            channels: audio_file.channels as u16,
            sample_rate: cpal::SampleRate(audio_file.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        println!("Playing audio with config: channels={}, sample_rate={}",
                 config.channels, config.sample_rate.0);

        // Shared state for audio playback
        let audio_state = Arc::new(Mutex::new(AudioPlaybackState {
            samples: Vec::new(),
            position: 0,
            finished: false,
        }));

        let state_clone = Arc::clone(&audio_state);

        // Create the audio stream callback
        let stream = device
            .build_output_stream(
                &config,
                move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut state = state_clone.lock().unwrap();

                    // Fill output buffer with samples
                    for sample in output.iter_mut() {
                        if state.position < state.samples.len() {
                            *sample = state.samples[state.position];
                            state.position += 1;
                        } else {
                            *sample = 0.0;
                            state.finished = true;
                        }
                    }
                },
                move |err| {
                    eprintln!("Stream error: {}", err);
                },
            )
            .map_err(|e| format!("failed to build output stream: {}", e))?;

        // Start playback
        stream
            .play()
            .map_err(|e| format!("failed to play stream: {}", e))?;

        // Decode all audio packets into samples
        decode_all_samples(&mut audio_file, &audio_state)?;

        Ok(AudioPlayer { _stream: stream })
    }
}

/// Internal state for audio playback
struct AudioPlaybackState {
    samples: Vec<f32>,
    position: usize,
    finished: bool,
}

/// Decodes all audio packets and stores samples in the playback state
fn decode_all_samples(
    audio_file: &mut AudioFile,
    state: &Arc<Mutex<AudioPlaybackState>>,
) -> Result<(), String> {
    let mut sample_count = 0;

    loop {
        // Get the next packet from the format reader
        let packet = match audio_file.format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(_)) => {
                // End of stream
                break;
            }
            Err(e) => {
                return Err(format!("failed to get packet: {}", e));
            }
        };

        // Only process packets for our track
        if packet.track_id() != audio_file.track_id {
            continue;
        }

        // Decode the packet
        match audio_file.decoder.decode(&packet) {
            Ok(audio_buf_ref) => {
                // Extract samples from the decoded audio buffer
                let mut samples = vec![];

                // Match on the audio buffer type and convert to f32
                match audio_buf_ref {
                    symphonia::core::audio::AudioBufferRef::F32(audio_buf) => {
                        // Get the number of frames in this buffer
                        let num_frames = audio_buf.frames();
                        let num_channels = audio_buf.spec().channels.count();

                        // Interleave samples from all channels
                        for frame in 0..num_frames {
                            for ch in 0..num_channels {
                                let sample = audio_buf.chan(ch)[frame];
                                samples.push(sample);
                            }
                        }

                        sample_count += num_frames;
                    }
                    symphonia::core::audio::AudioBufferRef::S16(audio_buf) => {
                        // Convert S16 to F32
                        let num_frames = audio_buf.frames();
                        let num_channels = audio_buf.spec().channels.count();

                        // Interleave samples from all channels
                        for frame in 0..num_frames {
                            for ch in 0..num_channels {
                                let sample = audio_buf.chan(ch)[frame] as f32 / 32768.0;
                                samples.push(sample);
                            }
                        }

                        sample_count += num_frames;
                    }
                    symphonia::core::audio::AudioBufferRef::U8(audio_buf) => {
                        // Convert U8 to F32
                        let num_frames = audio_buf.frames();
                        let num_channels = audio_buf.spec().channels.count();

                        for frame in 0..num_frames {
                            for ch in 0..num_channels {
                                let sample = (audio_buf.chan(ch)[frame] as f32 - 128.0) / 128.0;
                                samples.push(sample);
                            }
                        }

                        sample_count += num_frames;
                    }
                    symphonia::core::audio::AudioBufferRef::S32(audio_buf) => {
                        // Convert S32 to F32
                        let num_frames = audio_buf.frames();
                        let num_channels = audio_buf.spec().channels.count();

                        for frame in 0..num_frames {
                            for ch in 0..num_channels {
                                let sample = audio_buf.chan(ch)[frame] as f32 / 2147483648.0;
                                samples.push(sample);
                            }
                        }

                        sample_count += num_frames;
                    }
                    symphonia::core::audio::AudioBufferRef::S24(audio_buf) => {
                        // Convert S24 to F32
                        // i24 in symphonia stores a 24-bit signed integer
                        // We access it via the Debug implementation or through byte extraction
                        let num_frames = audio_buf.frames();
                        let num_channels = audio_buf.spec().channels.count();

                        for frame in 0..num_frames {
                            for ch in 0..num_channels {
                                // Use format! to get the string representation, then extract value
                                let sample_i24 = audio_buf.chan(ch)[frame];
                                let debug_str = format!("{:?}", sample_i24);
                                // Parse from "i24(12345)" format
                                if let Ok(value) = debug_str
                                    .trim_start_matches("i24(")
                                    .trim_end_matches(")")
                                    .parse::<i32>()
                                {
                                    let sample = value as f32 / 8388607.0;
                                    samples.push(sample);
                                }
                            }
                        }

                        sample_count += num_frames;
                    }
                    _ => {
                        // For other sample formats, try to skip silently
                        eprintln!("Unsupported audio format in packet");
                    }
                }

                // Add samples to the playback state
                if !samples.is_empty() {
                    let mut state = state.lock().unwrap();
                    state.samples.extend(samples);
                }
            }
            Err(symphonia::core::errors::Error::IoError(_)) => {
                break;
            }
            Err(e) => {
                return Err(format!("failed to decode packet: {}", e));
            }
        }
    }

    println!("Decoded {} samples", sample_count);
    Ok(())
}
