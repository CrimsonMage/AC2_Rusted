//! Every portal font decodes to its last byte and draws its glyphs (`AC2_DAT_DIR`).

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use crate::load_ac2_font;

#[test]
fn every_font_decodes_and_has_latin_glyphs() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x4200_0000, 0x4200_0FFE)
        .collect();
    assert!(!ids.is_empty());
    let store = ResourceStore::new(dataset);
    for did in ids {
        let font = load_ac2_font(&store, DataId::new(did))
            .unwrap_or_else(|e| panic!("{did:08x}: {e}"))
            .unwrap_or_else(|| panic!("{did:08x} missing"));
        let a = *font.char_desc(u16::from(b'A')).expect("an A");
        assert_eq!(a.unicode, u16::from(b'A'), "{did:08x}");
        let inked = (0..font.max_char_height)
            .flat_map(|row| (0..a.width).map(move |x| (x, row)))
            .filter(|&(x, row)| font.coverage(a.offset + x, row) != 0)
            .count();
        assert!(inked > 0, "{did:08x}: the A has ink");
        let units: Vec<u16> = "Hi\nthere".encode_utf16().collect();
        let there: Vec<u16> = "there".encode_utf16().collect();
        assert_eq!(font.string_width(&units), font.string_width(&there));
    }
}
