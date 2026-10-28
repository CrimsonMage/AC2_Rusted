//! `PropertyDesc` (the region's block property record) and its `BlockProperty`s: named properties
//! whose value per landblock cell is picked from a `BlockMap` and, for continuous ones, blended
//! between the four corners of the cell at a position.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{
    read_ac2_base_property_array, BaseProperty, BasePropertyValue, ResourceError,
    ResourcePackedDecode, ResourceResult, ResourceTypeId, RgbaColor,
};

use crate::ac2_water::Ac2BlockMap;
use crate::color::set_color32;
use crate::land_blocks::{blockpart_to_blockcoord, interpolate};

const TYPE_INTEGER: u32 = 0x4000_0002;
const TYPE_FLOAT: u32 = 0x4000_0003;
const TYPE_VECTOR: u32 = 0x4000_0004;
const TYPE_COLOR: u32 = 0x4000_0005;
/// The property `PropertyDesc::GetPassiblity` looks for, and its enum values.
const PASSABILITY_PROPERTY: u32 = 0x8000_0004;
const PASSABLE: u32 = 0x8000_0001;
const IMPASSABLE: u32 = 0x8000_0002;

/// `BlockProperty`: a property name, the block map indexing its values and the value table.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockProperty {
    pub pname: u32,
    pub block_map: DataId,
    /// The map value picks one table entry as it is, with no corner blending.
    pub discrete_block_property: bool,
    pub map_values: Vec<BaseProperty>,
}

impl BlockProperty {
    /// `BlockProperty::GetPropertyType`: the `MasterProperty` type of the property name.
    pub fn property_type(&self) -> Option<u32> {
        ac2_db::ac2_property_type(self.pname)
    }

    /// `BlockProperty::GetProperty`.
    pub fn get_property(&self, index: u32) -> Option<&BaseProperty> {
        self.map_values.get(index as usize)
    }

    fn value(&self, index: u32) -> Option<&BasePropertyValue> {
        self.get_property(index)?.value.as_ref()
    }

    /// `BlockProperty::GetInteger`.
    pub fn get_integer(&self, index: u32) -> Option<i32> {
        self.value(index)?.inq_integer()
    }

    /// `BlockProperty::GetFloat`.
    pub fn get_float(&self, index: u32) -> Option<f32> {
        self.value(index)?.inq_float()
    }

    /// `BlockProperty::GetVector`.
    pub fn get_vector(&self, index: u32) -> Option<[f32; 3]> {
        self.value(index)?.inq_vector().map(|v| [v.x, v.y, v.z])
    }

    /// `BlockProperty::GetColor`.
    pub fn get_color(&self, index: u32) -> Option<RgbaColor> {
        self.value(index)?.inq_color()
    }

    /// `BlockProperty::GetEnum`.
    pub fn get_enum(&self, index: u32) -> Option<u32> {
        self.value(index)?.inq_enum()
    }
}

/// `PropertyDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDesc {
    pub version: u32,
    pub block_props: Vec<BlockProperty>,
}

/// `PropertyDesc::IntegerCornerInterp`: `floor` of the blended corners.
pub fn integer_corner_interp(corners: [i32; 4], x: f32, y: f32) -> i32 {
    let [sw, se, nw, ne] = corners.map(|c| c as f32);
    f64::from(interpolate(x, y, sw, se, nw, ne)).floor() as i32
}

/// `PropertyDesc::FloatCornerInterp`.
pub fn float_corner_interp(corners: [f32; 4], x: f32, y: f32) -> f32 {
    let [sw, se, nw, ne] = corners;
    interpolate(x, y, sw, se, nw, ne)
}

/// `PropertyDesc::VectorCornerInterp`: each component blended on its own.
pub fn vector_corner_interp(corners: [[f32; 3]; 4], x: f32, y: f32) -> [f32; 3] {
    std::array::from_fn(|i| float_corner_interp(corners.map(|c| c[i]), x, y))
}

/// `PropertyDesc::ColorCornerInterp`: each channel blended on its own.
pub fn color_corner_interp(corners: [[f32; 4]; 4], x: f32, y: f32) -> [f32; 4] {
    std::array::from_fn(|i| float_corner_interp(corners.map(|c| c[i]), x, y))
}

impl PropertyDesc {
    /// `PropertyDesc::GetSubDataIDs`: each block property's block map (the call goes through
    /// the word at offset 8, `block_map`, of every 0x20-byte entry).
    pub fn sub_data_ids(&self) -> Vec<DataId> {
        self.block_props.iter().map(|prop| prop.block_map).collect()
    }

    /// `PropertyDesc::GetBlockPropertyInternal`: the value of `prop` at `origin` (landblock-local
    /// x and y) in the landblock of `cell_id`, read through `map`. A discrete property answers the
    /// table entry the cell's map value picks; otherwise the four corner entries of the cell are
    /// blended at the position, and the result is `None` unless every corner has a value of the
    /// property's type. `None` too for a cell id outside the world.
    pub fn block_property_at(
        &self,
        prop: &BlockProperty,
        map: &Ac2BlockMap,
        cell_id: u32,
        origin: [f32; 2],
    ) -> Option<BaseProperty> {
        if cell_id == 0 {
            return None;
        }
        let (block_x, block_y) = blockpart_to_blockcoord(cell_id >> 16)?;
        let (x, y) = (block_x as u32, block_y as u32);
        if prop.discrete_block_property {
            return prop.get_property(u32::from(map.value(x, y))).cloned();
        }
        let ptype = prop.property_type()?;
        let mut property = BaseProperty::new(0);
        property.set_property_name(prop.pname);
        let index = |dx: u32, dy: u32| u32::from(map.value(x + dx, y + dy));
        let [sw, se, nw, ne] = [index(0, 0), index(1, 0), index(0, 1), index(1, 1)];
        let [ox, oy] = origin;
        let value = match ptype {
            TYPE_INTEGER => {
                let c = [sw, se, nw, ne].map(|i| prop.get_integer(i));
                let c = [c[0]?, c[1]?, c[2]?, c[3]?];
                BasePropertyValue::Integer(integer_corner_interp(c, ox, oy))
            }
            TYPE_FLOAT => {
                let c = [sw, se, nw, ne].map(|i| prop.get_float(i));
                let c = [c[0]?, c[1]?, c[2]?, c[3]?];
                BasePropertyValue::Float(float_corner_interp(c, ox, oy))
            }
            TYPE_VECTOR => {
                let c = [sw, se, nw, ne].map(|i| prop.get_vector(i));
                let c = [c[0]?, c[1]?, c[2]?, c[3]?];
                let v = vector_corner_interp(c, ox, oy);
                BasePropertyValue::Vector(ac2_math::Vec3::from_array(v))
            }
            TYPE_COLOR => {
                let c = [sw, se, nw, ne].map(|i| prop.get_color(i));
                let c = [c[0]?, c[1]?, c[2]?, c[3]?].map(|c| [c.r, c.g, c.b, c.a]);
                let [r, g, b, a] = color_corner_interp(c, ox, oy);
                BasePropertyValue::Color(RgbaColor { r, g, b, a })
            }
            _ => return Some(property),
        };
        property.value = Some(value);
        Some(property)
    }

    /// `PropertyDesc::UpdateBlockMaterial`'s gather: every block property that has a value at the
    /// position, in table order, for the caller to fold into a physics material
    /// (`PhysicsMaterial::UpdateProperty`). `map_for` loads a block map by data id.
    pub fn block_properties_at(
        &self,
        map_for: &dyn Fn(DataId) -> Option<Ac2BlockMap>,
        cell_id: u32,
        origin: [f32; 2],
    ) -> Vec<BaseProperty> {
        self.block_props
            .iter()
            .filter_map(|prop| {
                if prop.block_map.raw() == 0 {
                    return None;
                }
                let map = map_for(prop.block_map)?;
                self.block_property_at(prop, &map, cell_id, origin)
            })
            .collect()
    }

    /// `PropertyDesc::GetPassiblity`: whether the landblock `(block_x, block_y)` can be entered.
    /// The first passability property's enum picked by the block map decides: impassable only for
    /// `IMPASSABLE`; with no such property, no map or any other value the block is passable.
    pub fn passability(
        &self,
        map_for: &dyn Fn(DataId) -> Option<Ac2BlockMap>,
        block_x: u32,
        block_y: u32,
    ) -> bool {
        let Some(prop) = self
            .block_props
            .iter()
            .find(|prop| prop.pname == PASSABILITY_PROPERTY)
        else {
            return true;
        };
        if prop.block_map.raw() == 0 {
            return true;
        }
        let Some(map) = map_for(prop.block_map) else {
            return true;
        };
        let index = u32::from(map.value(block_x, block_y));
        match prop.get_enum(index) {
            Some(PASSABLE) | None => true,
            Some(IMPASSABLE) => false,
            Some(_) => true,
        }
    }
}

fn bad(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: "AC2 PropertyDesc",
        context,
    }
}

/// A colour property stream is an ARGB word (`RGBAColor::StreamPack`).
fn to_base_property(raw: &ac2_db::Ac2PropertyValue) -> BaseProperty {
    let mut property = raw.to_base_property();
    if raw.value_type == TYPE_COLOR {
        if let Some(word) = raw.word() {
            let [r, g, b, a] = set_color32(word);
            property.value = Some(BasePropertyValue::Color(RgbaColor { r, g, b, a }));
        }
    }
    property
}

impl ResourcePackedDecode for PropertyDesc {
    const TYPE_NAME: &'static str = "AC2 PropertyDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `PropertyDesc` stream: DID, version, then counted `BlockProperty`s (`BlockProperty::
    /// StreamPack`: name, block map, discrete flag, counted `BaseProperty`s). Needs the
    /// `MasterProperty` table (`ac2_db::install_ac2_property_types`).
    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad(qdid, "embedded DID did not match DAT identity"));
        }
        let version = cursor.read_u32()?;
        let count = cursor.read_u32_count()?;
        if count > 65_536 || count * 16 > cursor.remaining() {
            return Err(bad(qdid, "item count exceeded the record"));
        }
        let mut block_props = Vec::with_capacity(count);
        for _ in 0..count {
            let pname = cursor.read_u32()?;
            let block_map = DataId::new(cursor.read_u32()?);
            let discrete = cursor.read_u32()? != 0;
            let values =
                read_ac2_base_property_array(&mut cursor).map_err(|context| bad(qdid, context))?;
            block_props.push(BlockProperty {
                pname,
                block_map,
                discrete_block_property: discrete,
                map_values: values.iter().map(to_base_property).collect(),
            });
        }
        if cursor.remaining() != 0 {
            return Err(bad(qdid, "trailing bytes after PropertyDesc"));
        }
        Ok(Self {
            version,
            block_props,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .block_props
                .iter()
                .map(|p| std::mem::size_of::<BlockProperty>() + p.map_values.len() * 64)
                .sum::<usize>()
    }
}
