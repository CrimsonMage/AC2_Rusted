//! `EncodedWave` files: RIFF WAVE files whose data is PCM or MPEG layer 3 (`WAVE_FORMAT_MPEGLAYER3`,
//! which retail decoded through the ACM codec DirectMusic loaded them with), decoded to samples.

use std::io::Cursor;
use std::sync::Arc;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::dls::pcm_from;
use super::riff::{riff, MusicFormatError, MusicResult, Reader};
use crate::data::Pcm;

const WAVE_FORMAT_PCM: u16 = 1;
const WAVE_FORMAT_MPEGLAYER3: u16 = 0x55;

/// Decodes a WAVE file.
pub fn decode(file: &[u8]) -> MusicResult<Pcm> {
    let top = riff(file)?;
    if !top.is_list(b"WAVE") {
        return Err(MusicFormatError("not a WAVE file"));
    }
    let format = top
        .child(b"fmt ")
        .ok_or(MusicFormatError("no wave format"))?;
    let data = top.child(b"data").ok_or(MusicFormatError("no wave data"))?;
    match Reader::new(format.data).u16() {
        WAVE_FORMAT_PCM => pcm_from(format.data, data.data),
        WAVE_FORMAT_MPEGLAYER3 => mp3(data.data),
        _ => Err(MusicFormatError("unsupported wave format")),
    }
}

/// An MPEG audio stream's samples, interleaved.
fn mp3(stream: &[u8]) -> MusicResult<Pcm> {
    let bad = |_| MusicFormatError("undecodable MPEG audio");
    let source = MediaSourceStream::new(Box::new(Cursor::new(stream.to_vec())), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            source,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(bad)?;
    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or(MusicFormatError("no MPEG audio track"))?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(bad)?;
    let mut samples = Vec::new();
    let (mut rate, mut channels) = (0u32, 0u16);
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let Ok(decoded) = decoder.decode(&packet) else {
            continue;
        };
        let spec = *decoded.spec();
        rate = spec.rate;
        channels = spec.channels.count() as u16;
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buffer.samples());
    }
    if rate == 0 || channels == 0 {
        return Err(MusicFormatError("empty MPEG audio"));
    }
    Ok(Pcm {
        rate,
        channels,
        samples: Arc::from(samples),
    })
}
