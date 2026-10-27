//! UI layouts: `LayoutDesc` (`DB_TYPE_UI_LAYOUT`, `0x22000000..=0x22FFFFFE` in the local DAT), a
//! tree of `ElementDesc`s, each with `StateDesc`s that hold the element's attributes, media,
//! transitions and child elements in that state.
//!
//! `LayoutDesc::UnPack`: the `DBObj` DID, `subIDArray` (`PSmartArray<unsigned long>`), the string
//! table DID, display width and height (u16), the element count (u16), 4-byte alignment, then the
//! top-level elements (`elements`, keyed by element id).
//! `ElementDesc::UnPack`: id, type, default state, the rectangle (x0, y0, x1, y1 as i16),
//! z level (u16), the four edge anchors (u8), the state count (u16), alignment, the states,
//! alignment.
//! `StateDesc::UnPack`: id, flags (u8), the rectangle, z level, alignment, then u32-counted
//! attributes (`AttributeDesc::UnPack`), media (type, then `MediaDesc::CreateMediaType`'s
//! `StreamPack`), transitions (`TransitionDesc::StreamPack`) and child elements, alignment.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, ArchiveError, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::layout_media::{pstring_bytes, read_media, read_transition};
pub use crate::layout_media::{MediaDesc, TransitionData, TransitionDesc};

/// `DB_TYPE_UI_LAYOUT`'s DID range.
pub const LAYOUT_FIRST: u32 = 0x2200_0000;
pub const LAYOUT_LAST: u32 = 0x22FF_FFFE;
const MAX_ITEMS: usize = 1 << 16;
/// Element nesting deeper than this is treated as corrupt.
const MAX_DEPTH: usize = 64;

/// `AttributeValue`, by `AttributeDesc::type` (the `UIElement::SetAttribute_*` slot
/// `UIElement::SetAttributeFromDesc` sends it to).
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// 1, `SetAttribute_ID`.
    Id(u32),
    /// 2, `SetAttribute_Int`.
    Int(i32),
    /// 3, `SetAttribute_Float`.
    Float(f32),
    /// 4, `SetAttribute_Bool`.
    Bool(bool),
    /// 5, `SetAttribute_ASCII`.
    Ascii(Vec<u8>),
    /// 6, `SetAttribute_StringID`.
    StringId(u32),
    /// 7, `SetAttribute_DataID`.
    DataId(u32),
    /// Any other type: no value, ignored by `SetAttributeFromDesc`.
    None(u8),
    /// `SetAttribute_InstanceID`: set at run time only (no layout carries one).
    InstanceId(u64),
}

/// One `AttributeDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDesc {
    pub id: u32,
    pub value: AttributeValue,
}

/// An element's or state's rectangle (`x0`, `y0`, `x1`, `y1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
}

/// One `StateDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDesc {
    pub id: u32,
    pub flags: u8,
    pub rect: Rect,
    pub z_level: u16,
    pub attributes: Vec<AttributeDesc>,
    pub media: Vec<MediaDesc>,
    pub transitions: Vec<TransitionDesc>,
    pub children: Vec<Arc<ElementDesc>>,
}

/// One `ElementDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementDesc {
    pub id: u32,
    /// The element class (`IUIManager` makes the matching `UIElement_*`).
    pub kind: u32,
    pub default_state: u32,
    pub rect: Rect,
    pub z_level: u16,
    /// `leftEdge`, `topEdge`, `rightEdge`, `bottomEdge`: how each side follows its parent.
    pub edges: [u8; 4],
    pub states: Vec<StateDesc>,
}

impl ElementDesc {
    /// The element's state `id`.
    pub fn state(&self, id: u32) -> Option<&StateDesc> {
        self.states.iter().find(|state| state.id == id)
    }
}

/// A `LayoutDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDesc {
    /// The layout's DID (`LayoutDesc`'s embedded identity): the key saved window locations are
    /// filed under (`UILocationData`'s layout id).
    pub did: u32,
    /// `subIDArray`: the DIDs `LayoutDesc::GetSubDataIDs` reports for loading. In every local
    /// layout they are the layout's own string table (`0x25xxxxxx`), and none adds an element:
    /// the element tree is the `elements` alone.
    pub sub_ids: Vec<u32>,
    pub string_table: u32,
    pub width: u16,
    pub height: u16,
    /// The top-level elements in DAT order. Retail hashes them by id and keeps the first of a
    /// repeated id.
    pub elements: Vec<Arc<ElementDesc>>,
}

impl LayoutDesc {
    /// `LayoutDesc::GetRootElementDesc`: the top-level element `id` (the first, when repeated).
    pub fn element(&self, id: u32) -> Option<&Arc<ElementDesc>> {
        self.elements.iter().find(|element| element.id == id)
    }

    /// The element `id` anywhere in the layout, depth first (a panel's templates can be nested in
    /// its page).
    pub fn find_element(&self, id: u32) -> Option<Arc<ElementDesc>> {
        fn walk(element: &Arc<ElementDesc>, id: u32) -> Option<Arc<ElementDesc>> {
            if element.id == id {
                return Some(Arc::clone(element));
            }
            element
                .states
                .iter()
                .flat_map(|state| state.children.iter())
                .find_map(|child| walk(child, id))
        }
        self.elements.iter().find_map(|element| walk(element, id))
    }
}

struct Reader<'a, 'b> {
    cursor: &'b mut ArchiveCursor<'a>,
    qdid: QualifiedDataId,
}

type ReadResult<T> = Result<T, ResourceError>;

impl Reader<'_, '_> {
    fn bad(&self, context: &'static str) -> ResourceError {
        ResourceError::MalformedTypedObject {
            qdid: self.qdid,
            type_name: LayoutDesc::TYPE_NAME,
            context,
        }
    }

    fn count(&mut self, item: usize, context: &'static str) -> ReadResult<usize> {
        let n = self.cursor.read_u32()? as usize;
        if n > MAX_ITEMS || n * item > self.cursor.remaining() {
            return Err(self.bad(context));
        }
        Ok(n)
    }

    fn rect(&mut self) -> Result<Rect, ArchiveError> {
        let mut read = || -> Result<i16, ArchiveError> { Ok(self.cursor.read_u16()? as i16) };
        Ok(Rect {
            x0: read()?,
            y0: read()?,
            x1: read()?,
            y1: read()?,
        })
    }

    /// `AttributeDesc::UnPack`.
    fn attribute(&mut self) -> ReadResult<AttributeDesc> {
        let id = self.cursor.read_u32()?;
        let kind = self.cursor.read_u8()?;
        self.cursor.align_to(4)?;
        let value = match kind {
            1 => AttributeValue::Id(self.cursor.read_u32()?),
            2 => AttributeValue::Int(self.cursor.read_i32()?),
            3 => AttributeValue::Float(self.cursor.read_f32()?),
            4 => AttributeValue::Bool(self.cursor.read_u32()? != 0),
            5 => AttributeValue::Ascii(pstring_bytes(self.cursor)?),
            6 => AttributeValue::StringId(self.cursor.read_u32()?),
            7 => AttributeValue::DataId(self.cursor.read_u32()?),
            other => AttributeValue::None(other),
        };
        self.cursor.align_to(4)?;
        Ok(AttributeDesc { id, value })
    }

    /// `StateDesc::UnPack`.
    fn state(&mut self, depth: usize) -> ReadResult<StateDesc> {
        let id = self.cursor.read_u32()?;
        let flags = self.cursor.read_u8()?;
        let rect = self.rect()?;
        let z_level = self.cursor.read_u16()?;
        self.cursor.align_to(4)?;
        let n = self.count(5, "attribute count")?;
        let attributes = (0..n)
            .map(|_| self.attribute())
            .collect::<ReadResult<_>>()?;
        let n = self.count(8, "media count")?;
        let mut media = Vec::with_capacity(n);
        for _ in 0..n {
            let kind = self.cursor.read_u32()?;
            match read_media(self.cursor, kind, MAX_ITEMS)? {
                Some(item) => media.push(item),
                None => return Err(self.bad("media type")),
            }
        }
        let n = self.count(12, "transition count")?;
        let transitions = (0..n)
            .map(|_| read_transition(self.cursor))
            .collect::<Result<_, _>>()?;
        let n = self.count(28, "child element count")?;
        let children = (0..n)
            .map(|_| self.element(depth + 1).map(Arc::new))
            .collect::<ReadResult<_>>()?;
        self.cursor.align_to(4)?;
        Ok(StateDesc {
            id,
            flags,
            rect,
            z_level,
            attributes,
            media,
            transitions,
            children,
        })
    }

    /// `ElementDesc::UnPack`.
    fn element(&mut self, depth: usize) -> ReadResult<ElementDesc> {
        if depth > MAX_DEPTH {
            return Err(self.bad("element nesting"));
        }
        let id = self.cursor.read_u32()?;
        let kind = self.cursor.read_u32()?;
        let default_state = self.cursor.read_u32()?;
        let rect = self.rect()?;
        let z_level = self.cursor.read_u16()?;
        let mut edges = [0; 4];
        for edge in &mut edges {
            *edge = self.cursor.read_u8()?;
        }
        let n = usize::from(self.cursor.read_u16()?);
        self.cursor.align_to(4)?;
        let states = (0..n)
            .map(|_| self.state(depth))
            .collect::<ReadResult<_>>()?;
        self.cursor.align_to(4)?;
        Ok(ElementDesc {
            id,
            kind,
            default_state,
            rect,
            z_level,
            edges,
            states,
        })
    }
}

impl ResourcePackedDecode for LayoutDesc {
    const TYPE_NAME: &'static str = "AC2 LayoutDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        let mut reader = Reader {
            cursor: &mut cursor,
            qdid,
        };
        if reader.cursor.read_u32()? != qdid.data_id().raw() {
            return Err(reader.bad("embedded DID did not match DAT identity"));
        }
        let n = reader.count(4, "sub id count")?;
        let sub_ids = (0..n)
            .map(|_| reader.cursor.read_u32())
            .collect::<Result<_, _>>()?;
        let string_table = reader.cursor.read_u32()?;
        let width = reader.cursor.read_u16()?;
        let height = reader.cursor.read_u16()?;
        let n = usize::from(reader.cursor.read_u16()?);
        reader.cursor.align_to(4)?;
        let elements = (0..n)
            .map(|_| reader.element(0).map(Arc::new))
            .collect::<ReadResult<_>>()?;
        if reader.cursor.remaining() != 0 {
            return Err(reader.bad("trailing bytes after LayoutDesc"));
        }
        Ok(Self {
            did: qdid.data_id().raw(),
            sub_ids,
            string_table,
            width,
            height,
            elements,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        fn element_bytes(element: &ElementDesc) -> usize {
            std::mem::size_of::<ElementDesc>()
                + element
                    .states
                    .iter()
                    .map(|state| {
                        std::mem::size_of::<StateDesc>()
                            + state.attributes.len() * std::mem::size_of::<AttributeDesc>()
                            + state.media.len() * std::mem::size_of::<MediaDesc>()
                            + state.transitions.len() * std::mem::size_of::<TransitionDesc>()
                            + state
                                .children
                                .iter()
                                .map(|c| element_bytes(c))
                                .sum::<usize>()
                    })
                    .sum::<usize>()
        }
        std::mem::size_of::<Self>()
            + self.sub_ids.len() * 4
            + self
                .elements
                .iter()
                .map(|e| element_bytes(e))
                .sum::<usize>()
    }
}

/// The layout `did` from the local DAT.
pub fn load_layout(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<LayoutDesc>>> {
    store.load_cached_packed::<LayoutDesc>(TypedResourceRef::new(DatFileKind::Local, did, 0))
}
