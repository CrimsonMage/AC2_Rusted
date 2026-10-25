//! `ByteStream`: the compiled WSL library a `WLib` carries (`WLib::UnPack` builds a
//! `RuntimeByteStream` and calls `ByteStream::UnPack`).
//!
//! `ByteStream::ReadByteStream`: the magic `BS_MAGIC_NUM` (0x1A015342), the stream version
//! (`PackObj::UNPACK_TYPE`, must be 15), then sections until the data ends, each a type, a byte
//! size and the body (`ByteStream::LoadSection`), with the pointer re-aligned to 4 after each.
//! `ByteStream::PackSectionList` writes them in the order version, opcodes, strings, imports,
//! exports, vtables, valid events, then the four debug sections.

mod tables;

pub use tables::{
    CheckpointExport, ExportData, ExportFunction, ExportPackageArgs, ImportData, ImportSymbol,
    PackageInfo, VTableId, VTableSection,
};

use crate::unpack::{Reader, UnpackError, UnpackResult};

/// `BS_MAGIC_NUM` (set by `RuntimeByteStream`'s static initializer).
pub const BS_MAGIC_NUM: u32 = 0x1A01_5342;
/// The only stream version this interpreter reads (`ByteStream::ReadByteStream`).
pub const BS_VERSION: u32 = 15;

/// Section types (`ByteStream::LoadSection`).
pub mod section {
    pub const OPCODES: u32 = 1;
    pub const STRING_LITERALS: u32 = 2;
    pub const IMPORTS: u32 = 3;
    pub const EXPORTS: u32 = 4;
    pub const VERSION_INFO: u32 = 5;
    pub const VTABLES: u32 = 6;
    pub const VALID_EVENTS: u32 = 7;
    pub const FUNCTION_LOC_DEBUG: u32 = 0x100;
    pub const SOURCE_FILE_DEBUG: u32 = 0x101;
    pub const LINE_NUM_DEBUG: u32 = 0x102;
    pub const FRAME_DEBUG: u32 = 0x103;
}

/// One decoded `ByteStream`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ByteStream {
    /// `VersionTable` (name, value) pairs; `RuntimeByteStream::ValidateVersion` checks them.
    pub versions: Vec<(Vec<u8>, Vec<u8>)>,
    /// `OpcodeStream`: the bytecode of every function in the library.
    pub opcodes: Vec<u8>,
    /// `StringLitTable::m_strtbl`: the string literals (UTF-16, descrambled).
    pub string_literals: Vec<Vec<u16>>,
    /// `StringLitTable::m_offsets`: per literal, the opcode offsets that use it.
    pub string_literal_uses: Vec<Vec<u32>>,
    pub imports: Vec<ImportData>,
    pub exports: Vec<ExportData>,
    pub vtables: VTableSection,
    /// `m_validEvents` (`PHashTable<FunctionID,int>`), as (function id, value) pairs.
    pub valid_events: Vec<(u32, u32)>,
    /// The debug sections (0x100-0x103) as raw bodies: the release DATs carry them empty and
    /// only the admin script debugger reads them.
    pub debug_sections: Vec<(u32, Vec<u8>)>,
}

impl ByteStream {
    /// `ByteStream::ReadByteStream` over the whole remaining `reader`.
    pub fn unpack(reader: &mut Reader<'_>) -> UnpackResult<Self> {
        if reader.u32()? != BS_MAGIC_NUM {
            return Err(reader.bad("magic != BS_MAGIC_NUM"));
        }
        if reader.u32()? != BS_VERSION {
            return Err(reader.bad("unsupported byte stream version"));
        }
        let mut stream = Self::default();
        while reader.remaining() != 0 {
            if reader.remaining() < 8 {
                return Err(reader.bad("size >= sizeof( uint32 ) * 2"));
            }
            let kind = reader.u32()?;
            let size = reader.u32()? as usize;
            let body = reader.bytes(size)?;
            stream.load_section(kind, body).map_err(|e| UnpackError {
                context: e.context,
                position: reader.position() - size + e.position,
            })?;
            // The writer pads to 4 after each section (`PackSection`); the reader skips the
            // pad only when it fits.
            let pad = (4 - reader.position() % 4) % 4;
            if pad != 0 && pad <= reader.remaining() {
                reader.bytes(pad)?;
            }
        }
        stream.validate_versions()?;
        Ok(stream)
    }

    /// `ByteStream::LoadSection`: each loader must consume its whole body
    /// (`PackObj::GET_SIZE_LEFT( data, start, size ) == 0`).
    fn load_section(&mut self, kind: u32, body: &[u8]) -> UnpackResult<()> {
        let mut r = Reader::new(body);
        match kind {
            section::OPCODES => {
                // `OpcodeStream::StreamPack`: byte count, then the bytes.
                let n = r.u32()? as usize;
                self.opcodes = r.bytes(n)?.to_vec();
            }
            section::STRING_LITERALS => {
                // `StringLitTable::StreamPack`: `PSmartArray<PStringBase<unsigned short>>`,
                // then `PSmartArray<PSmartArray<unsigned long>>` of use offsets.
                let n = r.count(4, "string literal count")?;
                self.string_literals = (0..n).map(|_| r.wpstring()).collect::<Result<_, _>>()?;
                let n = r.count(4, "string literal use count")?;
                self.string_literal_uses =
                    (0..n).map(|_| r.u32_array()).collect::<Result<_, _>>()?;
            }
            section::IMPORTS => self.imports = tables::imports(&mut r)?,
            section::EXPORTS => self.exports = tables::exports(&mut r)?,
            section::VERSION_INFO => {
                // `VersionTable::StreamPack`: `PSmartArray<VersionTable::ver>` of
                // (name, value) strings.
                let n = r.count(4, "version count")?;
                self.versions = (0..n)
                    .map(|_| Ok((r.pstring()?, r.pstring()?)))
                    .collect::<UnpackResult<_>>()?;
            }
            section::VTABLES => self.vtables = VTableSection::unpack(&mut r)?,
            section::VALID_EVENTS => {
                let n = r.hash_count(8, "valid event count")?;
                self.valid_events = (0..n)
                    .map(|_| Ok((r.u32()?, r.u32()?)))
                    .collect::<UnpackResult<_>>()?;
            }
            section::FUNCTION_LOC_DEBUG..=section::FRAME_DEBUG => {
                self.debug_sections.push((kind, body.to_vec()));
                return Ok(());
            }
            _ => return Err(r.bad("unknown byte stream section")),
        }
        r.finish("section body not fully read")
    }

    /// `RuntimeByteStream::ValidateVersion`: a `$wcc` entry must say compiler version 13.
    fn validate_versions(&self) -> UnpackResult<()> {
        for (name, value) in &self.versions {
            if name.as_slice() == b"$wcc" && value.as_slice() != b"13" {
                return Err(UnpackError {
                    context: "script compiler version is not 13",
                    position: 0,
                });
            }
        }
        Ok(())
    }

    /// `VM::GetPackageID(name)` over `m_pkgIdStrMap`: the package id of a package name. A
    /// package's singleton state is the WState `0x70000000 + (id & 0xFFFF)`
    /// (`VM::GetSingletonPackage`), and `vtables.package_id_map[id]` is its export/vtable index.
    pub fn package_id(&self, name: &str) -> Option<u32> {
        self.vtables
            .package_names
            .iter()
            .find(|(n, _)| n.as_slice() == name.as_bytes())
            .map(|(_, id)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(out: &mut Vec<u8>, w: u32) {
        out.extend(w.to_le_bytes());
    }

    fn section(out: &mut Vec<u8>, kind: u32, body: &[u8]) {
        push(out, kind);
        push(out, body.len() as u32);
        out.extend(body);
        while out.len() % 4 != 0 {
            out.push(0);
        }
    }

    #[test]
    fn sections_load_and_strings_descramble() {
        let mut stream = Vec::new();
        push(&mut stream, BS_MAGIC_NUM);
        push(&mut stream, BS_VERSION);
        // Opcodes: a 6-byte stream (the section is padded to 4 afterwards).
        let mut body = Vec::new();
        push(&mut body, 6);
        body.extend([1, 2, 3, 4, 5, 6]);
        section(&mut stream, section::OPCODES, &body);
        // One string literal "Hi!" used at opcode offset 8.
        let mut chars: Vec<u8> = "Hi!".encode_utf16().flat_map(u16::to_le_bytes).collect();
        ac2_core::pstring::munge(&mut chars);
        let mut body = Vec::new();
        push(&mut body, 1);
        body.extend(3u16.to_le_bytes());
        body.extend(&chars);
        body.extend([0, 0, 0, 0][..(4 - (2 + chars.len()) % 4) % 4].iter());
        push(&mut body, 1);
        push(&mut body, 1);
        push(&mut body, 8);
        section(&mut stream, section::STRING_LITERALS, &body);
        // An empty valid-event table (count 0, bucket index 4).
        section(
            &mut stream,
            section::VALID_EVENTS,
            &0x0400_0000u32.to_le_bytes(),
        );
        let parsed = ByteStream::unpack(&mut Reader::new(&stream)).unwrap();
        assert_eq!(parsed.opcodes, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(
            String::from_utf16(&parsed.string_literals[0]).unwrap(),
            "Hi!"
        );
        assert_eq!(parsed.string_literal_uses, vec![vec![8]]);
        assert!(parsed.valid_events.is_empty());
    }

    #[test]
    fn bad_magic_unknown_sections_and_short_bodies_fail() {
        let mut stream = Vec::new();
        push(&mut stream, 0x1234_5678);
        push(&mut stream, BS_VERSION);
        assert!(ByteStream::unpack(&mut Reader::new(&stream)).is_err());
        let mut stream = Vec::new();
        push(&mut stream, BS_MAGIC_NUM);
        push(&mut stream, BS_VERSION);
        section(&mut stream, 0x42, &[0, 0, 0, 0]);
        assert!(ByteStream::unpack(&mut Reader::new(&stream)).is_err());
        let mut stream = Vec::new();
        push(&mut stream, BS_MAGIC_NUM);
        push(&mut stream, BS_VERSION);
        // An opcode section whose body has a byte left over.
        section(&mut stream, section::OPCODES, &[1, 0, 0, 0, 9, 9]);
        assert!(ByteStream::unpack(&mut Reader::new(&stream)).is_err());
    }
}
