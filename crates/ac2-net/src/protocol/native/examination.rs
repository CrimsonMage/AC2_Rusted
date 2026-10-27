//! The examination natives (`gmCNativeFactory`): `SelectionInfo` (0xAB), `ExaminationRequest`
//! (0x9A, Legions layout) and `ExaminationProfile` (0x99) with its `ExaminationDataNode`s.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::visual::IconLayer;
use crate::protocol::ProtocolError;

/// `SelectionInfo::StreamPack` (identical in Legions, 0x005CA4D0): the type, then, for a
/// creature (`0x40000001`), its health and vigor as the selection panel shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionInfo {
    pub kind: u32,
    pub cur_health: u32,
    pub pk_damage: u32,
    pub max_health: u32,
    pub cur_vigor: u32,
    pub pk_vigor_loss: u32,
    pub max_vigor: u32,
}

impl SelectionInfo {
    /// The type whose health and vigor are packed.
    pub const CREATURE: u32 = 0x4000_0001;

    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let kind = r.u32()?;
        let mut info = Self {
            kind,
            ..Self::default()
        };
        if kind == Self::CREATURE {
            info.cur_health = r.u32()?;
            info.pk_damage = r.u32()?;
            info.max_health = r.u32()?;
            info.cur_vigor = r.u32()?;
            info.pk_vigor_loss = r.u32()?;
            info.max_vigor = r.u32()?;
        }
        Ok(info)
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.kind);
        if self.kind == Self::CREATURE {
            w.u32(self.cur_health)
                .u32(self.pk_damage)
                .u32(self.max_health)
                .u32(self.cur_vigor)
                .u32(self.pk_vigor_loss)
                .u32(self.max_vigor);
        }
    }
}

/// `ExaminationRequest` as Legions packs it (Legions 0x005C9210, unmatched to admin): the type,
/// the admin flag, then the datum the type names. Legions grew the class by three words
/// (`+0x20`, `+0x24`, `+0x28`, constructed as 0, 0, 0x203): type `0x40000002` (an object) also
/// packs the third, and the Legions-only type `0x4000000D` packs the object and all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExaminationRequest {
    pub kind: u32,
    pub admin: u32,
    /// `_data_enum`: types 0x40000001, 0x40000003-0x40000006.
    pub data_enum: u32,
    /// `_data_DataID`: types 0x40000007, 0x4000000B.
    pub data_id: u32,
    /// `_data_iid`: types 0x40000002, 0x4000000D.
    pub iid: u64,
    /// The Legions words at `+0x20`, `+0x24`, `+0x28`.
    pub extra: [u32; 3],
}

impl Default for ExaminationRequest {
    /// `ExaminationRequest::ExaminationRequest` / `Clear`.
    fn default() -> Self {
        Self {
            kind: 0,
            admin: 0,
            data_enum: 0,
            data_id: 0,
            // `INVALID_IID` is 0.
            iid: 0,
            extra: [0, 0, 0x203],
        }
    }
}

impl ExaminationRequest {
    /// The request type naming an object (`_data_iid`).
    pub const OBJECT: u32 = 0x4000_0002;
    /// The Legions request type with an object and the three extra words.
    pub const OBJECT_EXTENDED: u32 = 0x4000_000D;

    /// A request to examine an object, as `CPlayer` sets it on selection.
    pub fn object(iid: u64) -> Self {
        Self {
            kind: Self::OBJECT,
            iid,
            ..Self::default()
        }
    }

    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let mut request = Self {
            kind: r.u32()?,
            admin: r.u32()?,
            ..Self::default()
        };
        match request.kind {
            0x4000_0001 | 0x4000_0003..=0x4000_0006 => request.data_enum = r.u32()?,
            Self::OBJECT => {
                request.iid = r.u64()?;
                request.extra[2] = r.u32()?;
            }
            0x4000_0007 | 0x4000_000B => request.data_id = r.u32()?,
            Self::OBJECT_EXTENDED => {
                request.iid = r.u64()?;
                request.extra = [r.u32()?, r.u32()?, r.u32()?];
            }
            _ => {}
        }
        Ok(request)
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.kind).u32(self.admin);
        match self.kind {
            0x4000_0001 | 0x4000_0003..=0x4000_0006 => {
                w.u32(self.data_enum);
            }
            Self::OBJECT => {
                w.u64(self.iid).u32(self.extra[2]);
            }
            0x4000_0007 | 0x4000_000B => {
                w.u32(self.data_id);
            }
            Self::OBJECT_EXTENDED => {
                w.u64(self.iid);
                for word in self.extra {
                    w.u32(word);
                }
            }
            _ => {}
        }
    }
}

/// One `ExaminationDataNode`'s value, by node type (`ExaminationProfile::Add*`).
#[derive(Debug, Clone, PartialEq)]
pub enum ExaminationValue {
    /// 0x40000001 `AddInt`.
    Int(i32),
    /// 0x40000002 `AddFloat`.
    Float(f32),
    /// 0x40000003 `AddBool`: the int the script passed, unnormalised (retail packs `valBool`'s
    /// word as is).
    Bool(u32),
    /// 0x40000004 `AddStringInfo`.
    Text(StringInfo),
    /// 0x40000005 `AddImage`: an image DID (no appearance).
    Image(u32),
    /// 0x40000006 `AddBreak`.
    Break,
    /// 0x40000007 `AddTab` / `AddTabs`: the tab count (no appearance).
    Tabs(u32),
    /// 0x40000008 `AddCountdown`: seconds (`double`).
    Countdown(f64),
    /// 0x40000009 `AddIconDesc` (no appearance).
    Icon(Vec<IconLayer>),
    /// 0x4000000A `AddLongInt`.
    LongInt(i64),
    /// 0x4000000C `AddEffectID`: the effect id; `appearance` holds the depth.
    Effect(u32),
    /// Any other type: nothing more is packed.
    Other(u32),
}

/// `ExaminationDataNode::StreamPack` (Legions 0x005C95C0): type, order, then the appearance id
/// (for the types that have one) and the value.
#[derive(Debug, Clone, PartialEq)]
pub struct ExaminationDataNode {
    pub order: u32,
    /// `_appearanceID` (types 1-4, 8, 0xA, 0xC); 0 where the type packs none.
    pub appearance: u32,
    pub value: ExaminationValue,
}

impl ExaminationDataNode {
    fn kind(&self) -> u32 {
        match &self.value {
            ExaminationValue::Int(_) => 0x4000_0001,
            ExaminationValue::Float(_) => 0x4000_0002,
            ExaminationValue::Bool(_) => 0x4000_0003,
            ExaminationValue::Text(_) => 0x4000_0004,
            ExaminationValue::Image(_) => 0x4000_0005,
            ExaminationValue::Break => 0x4000_0006,
            ExaminationValue::Tabs(_) => 0x4000_0007,
            ExaminationValue::Countdown(_) => 0x4000_0008,
            ExaminationValue::Icon(_) => 0x4000_0009,
            ExaminationValue::LongInt(_) => 0x4000_000A,
            ExaminationValue::Effect(_) => 0x4000_000C,
            ExaminationValue::Other(kind) => *kind,
        }
    }

    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let kind = r.u32()?;
        let order = r.u32()?;
        let mut appearance = 0;
        let mut appearance_then = |r: &mut Reader<'_>| -> Result<(), ProtocolError> {
            appearance = r.u32()?;
            Ok(())
        };
        let value = match kind {
            0x4000_0001 => {
                appearance_then(r)?;
                ExaminationValue::Int(r.u32()? as i32)
            }
            0x4000_0002 => {
                appearance_then(r)?;
                ExaminationValue::Float(r.f32()?)
            }
            0x4000_0003 => {
                appearance_then(r)?;
                ExaminationValue::Bool(r.u32()?)
            }
            0x4000_0004 => {
                appearance_then(r)?;
                ExaminationValue::Text(StringInfo::read(r)?)
            }
            0x4000_0005 => ExaminationValue::Image(r.u32()?),
            0x4000_0006 => ExaminationValue::Break,
            0x4000_0007 => ExaminationValue::Tabs(r.u32()?),
            0x4000_0008 => {
                appearance_then(r)?;
                ExaminationValue::Countdown(r.f64()?)
            }
            0x4000_0009 => {
                // `IconDesc::StreamPack`: `PList<IconLayerDesc>`, a count then the layers.
                let layers = (0..r.count()?)
                    .map(|_| {
                        Ok(IconLayer {
                            layer: r.u32()?,
                            image: r.u32()?,
                            color: r.u32()?,
                        })
                    })
                    .collect::<Result<_, ProtocolError>>()?;
                ExaminationValue::Icon(layers)
            }
            0x4000_000A => {
                appearance_then(r)?;
                ExaminationValue::LongInt(r.u64()? as i64)
            }
            0x4000_000C => {
                appearance_then(r)?;
                ExaminationValue::Effect(r.u32()?)
            }
            other => ExaminationValue::Other(other),
        };
        Ok(Self {
            order,
            appearance,
            value,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.kind()).u32(self.order);
        match &self.value {
            ExaminationValue::Int(v) => {
                w.u32(self.appearance).u32(*v as u32);
            }
            ExaminationValue::Float(v) => {
                w.u32(self.appearance).f32(*v);
            }
            ExaminationValue::Bool(v) => {
                w.u32(self.appearance).u32(*v);
            }
            ExaminationValue::Text(text) => {
                w.u32(self.appearance);
                text.write(w);
            }
            ExaminationValue::Image(v) | ExaminationValue::Tabs(v) => {
                w.u32(*v);
            }
            ExaminationValue::Countdown(v) => {
                w.u32(self.appearance).f64(*v);
            }
            ExaminationValue::Icon(layers) => {
                w.u32(layers.len() as u32);
                for layer in layers {
                    w.u32(layer.layer).u32(layer.image).u32(layer.color);
                }
            }
            ExaminationValue::LongInt(v) => {
                w.u32(self.appearance).u64(*v as u64);
            }
            ExaminationValue::Effect(v) => {
                w.u32(self.appearance).u32(*v);
            }
            ExaminationValue::Break | ExaminationValue::Other(_) => {}
        }
    }
}

/// `ExaminationProfile::StreamPack`: the request it answers, then `PList<ExaminationDataNode>`
/// (a count, then the nodes).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExaminationProfile {
    pub request: ExaminationRequest,
    pub nodes: Vec<ExaminationDataNode>,
}

impl ExaminationProfile {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let request = ExaminationRequest::read(r)?;
        let nodes = (0..r.count()?)
            .map(|_| ExaminationDataNode::read(r))
            .collect::<Result<_, _>>()?;
        Ok(Self { request, nodes })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        self.request.write(w);
        w.u32(self.nodes.len() as u32);
        for node in &self.nodes {
            node.write(w);
        }
    }

    /// The shared loop of the `ExaminationProfile::Add*` family (admin 0x00608800-0x00608EE0):
    /// the node goes before the first node whose order is greater than `order` (equal orders
    /// keep their place), else at the end.
    fn add_node(&mut self, order: u32, appearance: u32, value: ExaminationValue) {
        let node = ExaminationDataNode {
            order,
            appearance,
            value,
        };
        match self.nodes.iter().position(|n| n.order > order) {
            Some(at) => self.nodes.insert(at, node),
            None => self.nodes.push(node),
        }
    }

    /// `ExaminationProfile::AddBool` (admin 0x00608800).
    pub fn add_bool(&mut self, value: u32, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::Bool(value));
    }

    /// `ExaminationProfile::AddCountdown` (admin 0x006088A0): seconds as a `double`.
    pub fn add_countdown(&mut self, seconds: f64, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::Countdown(seconds));
    }

    /// `ExaminationProfile::AddFloat` (admin 0x00608950).
    pub fn add_float(&mut self, value: f32, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::Float(value));
    }

    /// `ExaminationProfile::AddInt` (admin 0x006089F0).
    pub fn add_int(&mut self, value: i32, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::Int(value));
    }

    /// `ExaminationProfile::AddLongInt` (admin 0x00608A90).
    pub fn add_long_int(&mut self, value: i64, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::LongInt(value));
    }

    /// `ExaminationProfile::AddStringInfo` (admin 0x00608B40): a copy of the string info.
    pub fn add_string_info(&mut self, info: StringInfo, appearance: u32, order: u32) {
        self.add_node(order, appearance, ExaminationValue::Text(info));
    }

    /// `ExaminationProfile::AddSimpleStringInfo` (admin 0x006090E0): a string info holding only
    /// the string id and its table.
    pub fn add_simple_string_info(
        &mut self,
        string_id: u32,
        table_id: u32,
        appearance: u32,
        order: u32,
    ) {
        let info = StringInfo {
            string_id,
            table_id,
            literal: None,
            variables: Vec::new(),
        };
        self.add_string_info(info, appearance, order);
    }

    /// `ExaminationProfile::AddImage` (admin 0x00608BF0): an image DID, with no appearance.
    pub fn add_image(&mut self, did: u32, order: u32) {
        self.add_node(order, 0, ExaminationValue::Image(did));
    }

    /// `ExaminationProfile::AddIconDesc` (admin 0x00608C80): the icon's layers, no appearance.
    pub fn add_icon_desc(&mut self, layers: Vec<IconLayer>, order: u32) {
        self.add_node(order, 0, ExaminationValue::Icon(layers));
    }

    /// `ExaminationProfile::AddTabs` (admin 0x00608E50): a tab count, no appearance.
    pub fn add_tabs(&mut self, number: u32, order: u32) {
        self.add_node(order, 0, ExaminationValue::Tabs(number));
    }

    /// `ExaminationProfile::AddTab` (admin 0x00608DC0): a tab node whose count is the zeroed
    /// value of `ExaminationDataNode`'s constructor.
    pub fn add_tab(&mut self, order: u32) {
        self.add_tabs(0, order);
    }

    /// `ExaminationProfile::AddBreak` (admin 0x00608EE0): a line break, no value.
    pub fn add_break(&mut self, order: u32) {
        self.add_node(order, 0, ExaminationValue::Break);
    }

    /// `ExaminationProfile::AddEffectID` (admin 0x00608D20): an effect node (type `0x4000000C`)
    /// with the effect id as its value and `depth` as its appearance.
    pub fn add_effect_id(&mut self, eid: u32, depth: u32, order: u32) {
        self.add_node(order, depth, ExaminationValue::Effect(eid));
    }
}

#[cfg(test)]
mod tests {
    use super::super::NativeObject;
    use super::*;

    fn round_trip(object: NativeObject) -> Vec<u8> {
        let mut w = Writer::new();
        object.write(&mut w);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(
            NativeObject::read(object.native_type(), &mut r).unwrap(),
            object
        );
        r.finish().unwrap();
        for end in 0..w.bytes.len() {
            let mut r = Reader::new(&w.bytes[..end]);
            assert!(NativeObject::read(object.native_type(), &mut r).is_err());
        }
        w.bytes
    }

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn selection_info_packs_health_and_vigor_only_for_creatures() {
        let creature = SelectionInfo {
            kind: SelectionInfo::CREATURE,
            cur_health: 80,
            pk_damage: 1,
            max_health: 100,
            cur_vigor: 50,
            pk_vigor_loss: 2,
            max_vigor: 60,
        };
        assert_eq!(
            round_trip(NativeObject::SelectionInfo(creature)),
            words(&[0x4000_0001, 80, 1, 100, 50, 2, 60])
        );
        let item = SelectionInfo {
            kind: 0x4000_0002,
            ..SelectionInfo::default()
        };
        assert_eq!(
            round_trip(NativeObject::SelectionInfo(item)),
            words(&[0x4000_0002])
        );
    }

    #[test]
    fn examination_requests_follow_the_legions_layout() {
        let object = ExaminationRequest::object(0x8000_0001_0000_0002);
        let mut expected = words(&[0x4000_0002, 0]);
        expected.extend(0x8000_0001_0000_0002u64.to_le_bytes());
        expected.extend(0x203u32.to_le_bytes());
        assert_eq!(
            round_trip(NativeObject::ExaminationRequest(object)),
            expected
        );
        let extended = ExaminationRequest {
            kind: ExaminationRequest::OBJECT_EXTENDED,
            iid: 7,
            extra: [1, 2, 3],
            ..ExaminationRequest::default()
        };
        let mut expected = words(&[0x4000_000D, 0, 7, 0, 1, 2, 3]);
        assert_eq!(
            round_trip(NativeObject::ExaminationRequest(extended)),
            expected
        );
        let skill = ExaminationRequest {
            kind: 0x4000_0004,
            admin: 1,
            data_enum: 9,
            ..ExaminationRequest::default()
        };
        expected = words(&[0x4000_0004, 1, 9]);
        assert_eq!(
            round_trip(NativeObject::ExaminationRequest(skill)),
            expected
        );
        let none = ExaminationRequest::default();
        assert_eq!(
            round_trip(NativeObject::ExaminationRequest(none)),
            words(&[0, 0])
        );
    }

    #[test]
    fn examination_profile_nodes_take_their_type_layouts() {
        let profile = ExaminationProfile {
            request: ExaminationRequest::object(5),
            nodes: vec![
                ExaminationDataNode {
                    order: 1,
                    appearance: 0x11,
                    value: ExaminationValue::Int(-3),
                },
                ExaminationDataNode {
                    order: 2,
                    appearance: 0,
                    value: ExaminationValue::Break,
                },
                ExaminationDataNode {
                    order: 3,
                    appearance: 0,
                    value: ExaminationValue::Tabs(2),
                },
                ExaminationDataNode {
                    order: 4,
                    appearance: 0x12,
                    value: ExaminationValue::Countdown(1.5),
                },
                ExaminationDataNode {
                    order: 5,
                    appearance: 0,
                    value: ExaminationValue::Icon(vec![IconLayer {
                        layer: 1,
                        image: 0x4100_0001,
                        color: 0,
                    }]),
                },
                ExaminationDataNode {
                    order: 6,
                    appearance: 0x13,
                    value: ExaminationValue::Text(StringInfo {
                        string_id: 4,
                        table_id: 0x2300_0001,
                        ..StringInfo::default()
                    }),
                },
            ],
        };
        let bytes = round_trip(NativeObject::ExaminationProfile(profile));
        let mut expected = words(&[0x4000_0002, 0, 5, 0, 0x203, 6]);
        expected.extend(words(&[0x4000_0001, 1, 0x11, (-3i32) as u32]));
        expected.extend(words(&[0x4000_0006, 2]));
        expected.extend(words(&[0x4000_0007, 3, 2]));
        expected.extend(words(&[0x4000_0008, 4, 0x12]));
        expected.extend(1.5f64.to_le_bytes());
        expected.extend(words(&[0x4000_0009, 5, 1, 1, 0x4100_0001, 0]));
        expected.extend(words(&[0x4000_0004, 6, 0x13, 4, 0x2300_0001, 0]));
        assert_eq!(bytes, expected);
    }

    #[test]
    fn add_effect_id_sorts_by_order_and_packs_the_effect_layout() {
        let mut profile = ExaminationProfile {
            request: ExaminationRequest::object(5),
            nodes: vec![ExaminationDataNode {
                order: 2,
                appearance: 0,
                value: ExaminationValue::Break,
            }],
        };
        profile.add_effect_id(55, 0x11, 1);
        profile.add_effect_id(77, 0x12, 2);
        profile.add_effect_id(88, 0x13, 9);
        let orders: Vec<u32> = profile.nodes.iter().map(|n| n.order).collect();
        assert_eq!(orders, vec![1, 2, 2, 9]);
        // Equal orders keep their place: the older node stays first.
        assert_eq!(profile.nodes[1].value, ExaminationValue::Break);
        assert_eq!(profile.nodes[2].value, ExaminationValue::Effect(77));
        let bytes = round_trip(NativeObject::ExaminationProfile(profile));
        let mut expected = words(&[0x4000_0002, 0, 5, 0, 0x203, 4]);
        expected.extend(words(&[0x4000_000C, 1, 0x11, 55]));
        expected.extend(words(&[0x4000_0006, 2]));
        expected.extend(words(&[0x4000_000C, 2, 0x12, 77]));
        expected.extend(words(&[0x4000_000C, 9, 0x13, 88]));
        assert_eq!(bytes, expected);
    }

    #[test]
    fn add_methods_pack_the_retail_node_layouts() {
        let mut profile = ExaminationProfile {
            request: ExaminationRequest::object(5),
            nodes: Vec::new(),
        };
        profile.add_int(-2, 0x11, 1);
        // `AddBool` keeps the int as passed, not normalised to 0 or 1.
        profile.add_bool(2, 0x12, 2);
        profile.add_float(1.5, 0x13, 3);
        profile.add_tab(4);
        profile.add_image(0x2222, 5);
        profile.add_break(6);
        profile.add_countdown(1.5, 0x14, 7);
        profile.add_long_int(-1, 0x15, 8);
        profile.add_icon_desc(
            vec![IconLayer {
                layer: 2,
                image: 22,
                color: 2,
            }],
            9,
        );
        let bytes = round_trip(NativeObject::ExaminationProfile(profile.clone()));
        let mut expected = words(&[0x4000_0002, 0, 5, 0, 0x203, 9]);
        expected.extend(words(&[0x4000_0001, 1, 0x11, 0xFFFF_FFFE]));
        expected.extend(words(&[0x4000_0003, 2, 0x12, 2]));
        expected.extend(words(&[0x4000_0002, 3, 0x13, 1.5f32.to_bits()]));
        expected.extend(words(&[0x4000_0007, 4, 0]));
        expected.extend(words(&[0x4000_0005, 5, 0x2222]));
        expected.extend(words(&[0x4000_0006, 6]));
        expected.extend(words(&[0x4000_0008, 7, 0x14, 0, 0x3FF8_0000]));
        expected.extend(words(&[0x4000_000A, 8, 0x15, 0xFFFF_FFFF, 0xFFFF_FFFF]));
        expected.extend(words(&[0x4000_0009, 9, 1, 2, 22, 2]));
        assert_eq!(bytes, expected);
    }

    #[test]
    fn add_simple_string_info_holds_only_the_id_and_table() {
        let mut profile = ExaminationProfile::default();
        profile.add_simple_string_info(7, 0x4000_0026, 3, 1);
        assert_eq!(
            profile.nodes,
            vec![ExaminationDataNode {
                order: 1,
                appearance: 3,
                value: ExaminationValue::Text(StringInfo {
                    string_id: 7,
                    table_id: 0x4000_0026,
                    literal: None,
                    variables: Vec::new(),
                }),
            }]
        );
    }
}
