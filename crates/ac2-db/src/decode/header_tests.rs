use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;

use super::{decode_resource_body, DecodeContext};
use crate::ResourceTypeId;

fn decode_header(
    type_id: u32,
    did: u32,
    bytes: &[u8],
) -> crate::ResourceResult<super::DbObjHeader> {
    let ctx = DecodeContext::new(
        QualifiedDataId::new(DataId::new(did), type_id),
        "header test",
    );
    let mut cursor = ArchiveCursor::new(bytes);
    ctx.decode_dbobj_header(&mut cursor)
}

#[test]
fn reads_the_did_header() {
    for &(type_id, did) in &[(0, 0x0100_0001_u32), (0x7FFF_FFFF, 0xDEAD_BEEF)] {
        let header = decode_header(type_id, did, &did.to_le_bytes()).expect("header decodes");
        assert_eq!(header.did, DataId::new(did));
    }
}

#[test]
fn rejects_a_mismatched_or_missing_did() {
    assert!(decode_header(0, 0x0100_0001, &0x0100_0002_u32.to_le_bytes()).is_err());
    assert!(decode_header(0, 0x0100_0001, &[1, 0]).is_err());
}

#[test]
fn body_seam_exposes_only_the_metadata_validated_payload() {
    let did: u32 = 0x0500_0001;
    let mut bytes = did.to_le_bytes().to_vec();
    bytes.extend_from_slice(&[9, 8, 7]);
    let (context, body) = decode_resource_body(
        QualifiedDataId::new(DataId::new(did), ResourceTypeId::Unknown as u32),
        "body test",
        ResourceTypeId::Unknown,
        0x002A,
        &bytes,
    )
    .expect("resource body should validate");
    assert_eq!(context.qdid().data_id(), DataId::new(did));
    assert_eq!(context.dbobj_pack_version(), 0x002A);
    assert_eq!(body.bytes(), &[9, 8, 7]);
}
