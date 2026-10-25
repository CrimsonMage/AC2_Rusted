//! The `ByteStream` import, export and vtable sections.

use crate::unpack::{Reader, UnpackResult};

/// `VTableID`: a function slot of a package, packed as one u32 (`funcNum` low, `pkgNum` high;
/// 0xFFFF/0xFFFF is the undefined id).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VTableId(pub u32);

impl VTableId {
    pub const UNDEF: Self = Self(u32::MAX);

    pub fn func_num(self) -> u16 {
        self.0 as u16
    }

    pub fn pkg_num(self) -> u16 {
        (self.0 >> 16) as u16
    }
}

/// `ImportSymbolInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSymbol {
    pub name: Vec<u8>,
    pub offsets: Vec<u32>,
    pub from_addr: u32,
}

/// `ImportData` (`ImportData::StreamPack`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportData {
    pub package_name: Vec<u8>,
    pub symbols: Vec<ImportSymbol>,
    pub package_offsets: Vec<u32>,
}

/// `CheckpointExportData`, keyed by member name in `ExportPackageArgs::m_checkpoint`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointExport {
    pub name: Vec<u8>,
    pub offset: u32,
    pub tag: u8,
}

/// `ExportPackageArgs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPackageArgs {
    pub name: Vec<u8>,
    pub base_name: Vec<u8>,
    pub flags: u32,
    pub size: u32,
    pub checksum: u32,
    pub pkg_id: u32,
    /// The parent package's export index, `u32::MAX` for none.
    pub parent_index: u32,
    pub checkpoint: Vec<CheckpointExport>,
}

/// `ExportFunctionData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFunction {
    pub name: Vec<u8>,
    pub fid: u32,
    pub offset: u32,
    pub size: u32,
    pub flags: u32,
    pub deps: Vec<VTableId>,
}

/// `ExportData`: one package of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportData {
    pub args: ExportPackageArgs,
    pub functions: Vec<ExportFunction>,
}

/// `PackageInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PackageInfo {
    pub size: u32,
    pub checksum: u32,
}

/// `VTableSection`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VTableSection {
    /// `m_funcMapper`, by package id: the function slots each package maps.
    pub func_mapper: Vec<Vec<VTableId>>,
    /// `m_vtbl`, by export index: each package's function opcode offsets (`u32::MAX` empty).
    pub vtables: Vec<Vec<u32>>,
    /// `m_pkgInfo`, by package id.
    pub package_info: Vec<PackageInfo>,
    /// `m_pkgIdMap`: package id -> export/vtable index (`u32::MAX` for none).
    pub package_id_map: Vec<u32>,
    /// `m_pkgIdStrMap`: (package name, package id).
    pub package_names: Vec<(Vec<u8>, u32)>,
}

impl VTableSection {
    /// `VTableSection::StreamPack`.
    pub fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let n = r.count(4, "func mapper count")?;
        let func_mapper = (0..n)
            .map(|_| Ok(r.u32_array()?.into_iter().map(VTableId).collect()))
            .collect::<UnpackResult<_>>()?;
        let n = r.count(4, "vtable count")?;
        let vtables = (0..n).map(|_| r.u32_array()).collect::<UnpackResult<_>>()?;
        let n = r.count(8, "package info count")?;
        let package_info = (0..n)
            .map(|_| {
                Ok(PackageInfo {
                    size: r.u32()?,
                    checksum: r.u32()?,
                })
            })
            .collect::<UnpackResult<_>>()?;
        let package_id_map = r.u32_array()?;
        let n = r.hash_count(8, "package name count")?;
        let package_names = (0..n)
            .map(|_| Ok((r.pstring()?, r.u32()?)))
            .collect::<UnpackResult<_>>()?;
        Ok(Self {
            func_mapper,
            vtables,
            package_info,
            package_id_map,
            package_names,
        })
    }
}

/// `PSmartArray<ImportData>::StreamPack`.
pub(super) fn imports(r: &mut Reader<'_>) -> UnpackResult<Vec<ImportData>> {
    let n = r.count(4, "import count")?;
    (0..n)
        .map(|_| {
            let package_name = r.pstring()?;
            let m = r.count(4, "import symbol count")?;
            let symbols = (0..m)
                .map(|_| {
                    Ok(ImportSymbol {
                        name: r.pstring()?,
                        offsets: r.u32_array()?,
                        from_addr: r.u32()?,
                    })
                })
                .collect::<UnpackResult<_>>()?;
            Ok(ImportData {
                package_name,
                symbols,
                package_offsets: r.u32_array()?,
            })
        })
        .collect()
}

/// `PSmartArray<ExportData>::StreamPack` (`ExportData::StreamPack`: `ExportPackageArgs`, then
/// `PSmartArray<ExportFunctionData>`).
pub(super) fn exports(r: &mut Reader<'_>) -> UnpackResult<Vec<ExportData>> {
    let n = r.count(4, "export count")?;
    (0..n)
        .map(|_| {
            let args = package_args(r)?;
            let m = r.count(4, "export function count")?;
            let functions = (0..m)
                .map(|_| {
                    Ok(ExportFunction {
                        name: r.pstring()?,
                        fid: r.u32()?,
                        offset: r.u32()?,
                        size: r.u32()?,
                        flags: r.u32()?,
                        deps: r.u32_array()?.into_iter().map(VTableId).collect(),
                    })
                })
                .collect::<UnpackResult<_>>()?;
            Ok(ExportData { args, functions })
        })
        .collect()
}

/// `ExportPackageArgs::StreamPack`; the checkpoint table is
/// `DualHash<PStringBase<char>,CheckpointExportData>::UnPack` (u32 count, then name and
/// `CheckpointExportData::UnPack`: offset, tag byte, alignment).
fn package_args(r: &mut Reader<'_>) -> UnpackResult<ExportPackageArgs> {
    let name = r.pstring()?;
    let base_name = r.pstring()?;
    let flags = r.u32()?;
    let size = r.u32()?;
    let checksum = r.u32()?;
    let pkg_id = r.u32()?;
    let parent_index = r.u32()?;
    let n = r.count(8, "checkpoint count")?;
    let checkpoint = (0..n)
        .map(|_| {
            let name = r.pstring()?;
            let offset = r.u32()?;
            let tag = r.u8()?;
            r.align()?;
            Ok(CheckpointExport { name, offset, tag })
        })
        .collect::<UnpackResult<_>>()?;
    Ok(ExportPackageArgs {
        name,
        base_name,
        flags,
        size,
        checksum,
        pkg_id,
        parent_index,
        checkpoint,
    })
}
