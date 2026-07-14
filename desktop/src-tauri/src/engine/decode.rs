//! In-process audio decode: symphonia demux/decode + rubato resample to
//! 16 kHz mono f32 — no ffmpeg dependency.

use std::fs::File;
use std::path::Path;

use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("could not open {0}")]
    Open(String),
    #[error("unsupported or corrupt audio: {0}")]
    Unsupported(String),
    #[error("decode failed: {0}")]
    Decode(String),
    #[error("resample failed: {0}")]
    Resample(String),
    #[error("file contains no audio samples")]
    Empty,
}

/// Decode any supported container/codec to 16 kHz mono f32 samples.
pub fn decode_to_mono_16k(path: &Path) -> Result<Vec<f32>, DecodeError> {
    let file = File::open(path)
        .map_err(|e| DecodeError::Open(format!("{}: {e}", path.display())))?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| DecodeError::Unsupported(e.to_string()))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| DecodeError::Unsupported("no decodable audio track".into()))?;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| DecodeError::Unsupported(e.to_string()))?;

    let mut source_rate = track.codec_params.sample_rate.unwrap_or(0);
    let mut mono: Vec<f32> = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            // Normal end of stream.
            Err(SymphoniaError::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(DecodeError::Decode(e.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // Skip a corrupt packet, keep the file going.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(DecodeError::Decode(e.to_string())),
        };

        let spec = *decoded.spec();
        source_rate = spec.rate;
        let channels = spec.channels.count().max(1);

        let needed = decoded.capacity() as u64;
        let recreate = match &sample_buf {
            Some(buf) => buf.capacity() < decoded.capacity() * channels,
            None => true,
        };
        if recreate {
            sample_buf = Some(SampleBuffer::<f32>::new(needed, spec));
        }
        let buf = sample_buf.as_mut().unwrap();
        buf.copy_interleaved_ref(decoded);

        let samples = buf.samples();
        if channels == 1 {
            mono.extend_from_slice(samples);
        } else {
            // Downmix: average all channels per frame.
            mono.extend(
                samples
                    .chunks_exact(channels)
                    .map(|frame| frame.iter().sum::<f32>() / channels as f32),
            );
        }
    }

    if mono.is_empty() {
        return Err(DecodeError::Empty);
    }
    if source_rate == 0 {
        return Err(DecodeError::Unsupported("unknown sample rate".into()));
    }
    if source_rate == TARGET_SAMPLE_RATE {
        return Ok(mono);
    }
    resample_to_target(&mono, source_rate)
}

fn resample_to_target(input: &[f32], source_rate: u32) -> Result<Vec<f32>, DecodeError> {
    let ratio = TARGET_SAMPLE_RATE as f64 / source_rate as f64;
    let chunk = 4096;

    let params = SincInterpolationParameters {
        sinc_len: 128,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::Blackman2,
    };
    let mut resampler = SincFixedIn::<f32>::new(ratio, 1.1, params, chunk, 1)
        .map_err(|e| DecodeError::Resample(e.to_string()))?;

    let mut out: Vec<f32> =
        Vec::with_capacity((input.len() as f64 * ratio) as usize + chunk);
    let mut pos = 0;

    while pos + chunk <= input.len() {
        let frames = resampler
            .process(&[&input[pos..pos + chunk]], None)
            .map_err(|e| DecodeError::Resample(e.to_string()))?;
        out.extend_from_slice(&frames[0]);
        pos += chunk;
    }

    let tail = &input[pos..];
    if !tail.is_empty() {
        let frames = resampler
            .process_partial(Some(&[tail]), None)
            .map_err(|e| DecodeError::Resample(e.to_string()))?;
        out.extend_from_slice(&frames[0]);
    }
    // Flush the sinc filter's delay line so the clip isn't cut short.
    let frames = resampler
        .process_partial::<&[f32]>(None, None)
        .map_err(|e| DecodeError::Resample(e.to_string()))?;
    out.extend_from_slice(&frames[0]);

    Ok(out)
}
