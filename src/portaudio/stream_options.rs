use libportaudio_sys as ffi;

use ffi::PaSampleFormat;

use crate::error::{Error, Result};
use crate::stream_options::Format;

#[allow(dead_code)]
pub fn unpack_pa_formats(format_bitfield: ffi::PaSampleFormat) -> Result<Vec<Format>> {
    if format_bitfield.intersects(PaSampleFormat::paCustomFormat | PaSampleFormat::paNonInterleaved)
    {
        return Err(Error::Unknown(
            "Does not support custom and/or interleaved channels.",
        ));
    }
    let mut formats = Vec::new();
    for &(pa_format, format) in &[
        (PaSampleFormat::paFloat32, Format::F32),
        (PaSampleFormat::paInt32, Format::I32),
        (PaSampleFormat::paInt24, Format::I24),
        (PaSampleFormat::paInt16, Format::I16),
        (PaSampleFormat::paInt8, Format::I8),
        (PaSampleFormat::paUInt8, Format::U8),
    ] {
        if format_bitfield.contains(pa_format) {
            formats.push(format);
        }
    }
    Ok(formats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpacks_formats() {
        assert_eq!(
            unpack_pa_formats(PaSampleFormat::paFloat32).unwrap(),
            vec![Format::F32]
        );
        assert_eq!(
            unpack_pa_formats(PaSampleFormat::paFloat32 | PaSampleFormat::paInt24).unwrap(),
            vec![Format::F32, Format::I24]
        );
        assert!(unpack_pa_formats(
            PaSampleFormat::paFloat32 | PaSampleFormat::paInt24 | PaSampleFormat::paCustomFormat
        )
        .is_err());
    }
}
