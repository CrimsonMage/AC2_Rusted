use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, ArchiveResult};

use crate::body_decode::{ResourceDbObjBody, ResourceDbObjDecodeContext};
use crate::db_type::ResourceTypeId;
use crate::{ResourceError, ResourceResult};

#[cfg(test)]
#[path = "decode/header_tests.rs"]
mod header_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DbObjHeader {
    pub(crate) did: DataId,
}

#[derive(Debug, Clone, Copy)]
/// Bounded archive error adapter for domain-owned body decoders.
///
/// This is an error/format helper only; it grants no storage or header metadata access.
pub(crate) struct DecodeContext {
    qdid: QualifiedDataId,
    type_name: &'static str,
}

impl DecodeContext {
    pub const fn new(qdid: QualifiedDataId, type_name: &'static str) -> Self {
        Self { qdid, type_name }
    }

    pub fn ensure_resource_type(self, expected: ResourceTypeId) -> ResourceResult<()> {
        if ResourceTypeId::from_type_id(self.qdid.type_id()) == Some(expected) {
            Ok(())
        } else {
            Err(self.malformed("qualified data id type did not match the requested decoder"))
        }
    }

    pub(crate) fn decode_dbobj_header(
        self,
        cursor: &mut ArchiveCursor<'_>,
    ) -> ResourceResult<DbObjHeader> {
        self.align_to(cursor, 4, "missing DBObj header alignment")?;
        let did = DataId::new(self.read_u32(cursor, "missing DBObj DID header")?);
        if did != self.qdid.data_id() {
            return Err(self.malformed("DBObj DID header did not match DAT metadata"));
        }
        Ok(DbObjHeader { did })
    }

    pub(crate) fn align_to(
        self,
        cursor: &mut ArchiveCursor<'_>,
        align: usize,
        context: &'static str,
    ) -> ResourceResult<()> {
        self.map_archive(cursor.align_to(align), context)
    }

    pub fn read_u32(
        self,
        cursor: &mut ArchiveCursor<'_>,
        context: &'static str,
    ) -> ResourceResult<u32> {
        self.map_archive(cursor.read_u32(), context)
    }

    pub fn malformed(self, context: &'static str) -> ResourceError {
        ResourceError::MalformedTypedObject {
            qdid: self.qdid,
            type_name: self.type_name,
            context,
        }
    }

    fn map_archive<T>(self, result: ArchiveResult<T>, context: &'static str) -> ResourceResult<T> {
        result.map_err(|_| self.malformed(context))
    }
}

pub(crate) fn decode_resource_body<'a>(
    qdid: QualifiedDataId,
    type_name: &'static str,
    expected: ResourceTypeId,
    dbobj_pack_version: u16,
    bytes: &'a [u8],
) -> ResourceResult<(ResourceDbObjDecodeContext, ResourceDbObjBody<'a>)> {
    let ctx = DecodeContext::new(qdid, type_name);
    ctx.ensure_resource_type(expected)?;
    let mut cursor = ArchiveCursor::new(bytes);
    ctx.decode_dbobj_header(&mut cursor)?;
    Ok((
        ResourceDbObjDecodeContext {
            qdid,
            dbobj_pack_version,
        },
        ResourceDbObjBody {
            bytes: cursor.remaining_bytes(),
        },
    ))
}

pub(crate) fn decode_resource_packed_body<'a>(
    qdid: QualifiedDataId,
    type_name: &'static str,
    expected: ResourceTypeId,
    dbobj_pack_version: u16,
    bytes: &'a [u8],
) -> ResourceResult<(QualifiedDataId, u16, &'a [u8])> {
    let ctx = DecodeContext::new(qdid, type_name);
    ctx.ensure_resource_type(expected)?;
    Ok((qdid, dbobj_pack_version, bytes))
}
