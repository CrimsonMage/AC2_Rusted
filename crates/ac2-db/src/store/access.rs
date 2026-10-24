use std::io::Read;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use flate2::read::ZlibDecoder;

use crate::{ResourceError, ResourceResult};

pub(super) fn decompress_record_bytes(
    file_kind: DatFileKind,
    did: DataId,
    packed: Vec<u8>,
) -> ResourceResult<Option<Vec<u8>>> {
    if packed.len() < 4 {
        return Err(ResourceError::DecompressionIo {
            file_kind,
            did,
            message: "packed payload was missing the uncompressed-length prefix".to_string(),
        });
    }

    let expected_len = u32::from_le_bytes([packed[0], packed[1], packed[2], packed[3]]) as usize;
    let mut decoder = ZlibDecoder::new(&packed[4..]);
    let mut out = Vec::with_capacity(expected_len);
    decoder
        .read_to_end(&mut out)
        .map_err(|error| ResourceError::DecompressionIo {
            file_kind,
            did,
            message: error.to_string(),
        })?;
    if out.len() != expected_len {
        return Err(ResourceError::DecompressionLengthMismatch {
            file_kind,
            did,
            expected_len,
            actual_len: out.len(),
        });
    }
    Ok(Some(out))
}
