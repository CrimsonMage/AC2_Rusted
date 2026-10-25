//! The plain data some `CM_*` notices hand the UI: a store (`UIShop` with its `UIShopSale`s),
//! a craft skill (`CraftSkillAdaptor`) and a floating damage text (`DamageTextBlob`), as the
//! player's scripts fill them.

use ac2_net::protocol::StringInfo;

/// `UIShopSale`: one item for sale.
#[derive(Debug, Clone, PartialEq)]
pub struct ShopSale {
    /// `m_uiOrdinal` (the store keeps its sales sorted by it).
    pub ordinal: u32,
    /// `m_iidItem` (`INVALID_IID`, 0, when none).
    pub item: u64,
    pub item_did: u32,
    pub icon_did: u32,
    pub item_desc: StringInfo,
    pub cost_desc: StringInfo,
    pub cost_icon: u32,
    pub cost_quantity: f32,
    pub stock: i32,
    /// `m_uiStatus`: 1 restricted, 2 expired, 3 sold.
    pub status: u32,
}

impl Default for ShopSale {
    /// `UIShopSale::UIShopSale`.
    fn default() -> Self {
        Self {
            ordinal: 0,
            item: 0,
            item_did: 0,
            icon_did: 0,
            item_desc: StringInfo::default(),
            cost_desc: StringInfo::default(),
            cost_icon: 0,
            cost_quantity: 0.0,
            stock: 0,
            status: 0,
        }
    }
}

/// `UIShop`: a store page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shop {
    pub id: u32,
    pub name: StringInfo,
    pub desc: StringInfo,
    pub sorter: StringInfo,
    pub portrait: u32,
    pub sales: Vec<ShopSale>,
    pub position: i32,
    pub store_size: i32,
}

/// `CraftSkillAdaptor`.
#[derive(Debug, Clone, PartialEq)]
pub struct CraftSkill {
    pub did: u32,
    pub name: StringInfo,
    pub description: StringInfo,
    pub level: u32,
    pub raw_level: u32,
    pub cost_to_raise: u64,
    pub icon: u32,
    /// `m_mod`: the usage experience modifier.
    pub xp_mod: f32,
}

impl Default for CraftSkill {
    /// `CraftSkillAdaptor::CraftSkillAdaptor`.
    fn default() -> Self {
        Self {
            did: 0,
            name: StringInfo::default(),
            description: StringInfo::default(),
            level: 0,
            raw_level: 0,
            cost_to_raise: 0,
            icon: 0,
            xp_mod: 1.0,
        }
    }
}

/// `DamageTextBlob`: a floating damage number.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DamageText {
    pub text: String,
    /// `_fontColor` (r, g, b, a).
    pub color: [f32; 4],
    pub duration: f32,
    pub scale: f32,
}
