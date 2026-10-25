//! What the inventory panels show of an item, asked of the item's own script process as
//! `ItemDisplay::Update` and the containers do (`gmCWInterface::GetQuantity`, `GetIconID`,
//! `GetPotentiallyPluralizedName`, `GetStackable`, `GetMaxStackSize`, `HasDurability`,
//! `GetDurabilityLevel`, `GetMaxDurabilityLevel`, `GetMaxContainerSize`: `gmEntity` 0x10,
//! 0x23, 0x10B, 0xB, 0x2E, 0x158, 0x159, 0x15B, 0xC over the item's qualities), plus the
//! local `VisualDesc`'s icon layers and the value its `WeenieDesc` carries.

use ac2_net::protocol::StringInfo;
use ac2_script::host::{native_for_wire, IFunArg, IFunResult, IFunReturn};
use ac2_script::vm::InterpStatus;

use crate::client::Client;

/// The `gmCWInterface` IFuns.
const GET_ICON_ID: u32 = 0x00B4_001C;
const GET_MAX_CONTAINER_SIZE: u32 = 0x00B4_001D;
const GET_STACKABLE: u32 = 0x00B4_0063;
const GET_QUANTITY: u32 = 0x00B4_0064;
const GET_MAX_STACK_SIZE: u32 = 0x00B4_0065;
const GET_POTENTIALLY_PLURALIZED_NAME: u32 = 0x00B4_00C7;
const HAS_DURABILITY: u32 = 0x00B4_00FF;
const GET_MAX_DURABILITY_LEVEL: u32 = 0x00B4_0102;
const GET_DURABILITY_LEVEL: u32 = 0x00B4_0103;

/// One item as `ItemDisplay` shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemDisplay {
    /// The tooltip: the name, or "N <plural>" for a stack of more than one.
    pub name: StringInfo,
    /// `GetIconID` (`DataID` 257), what the slot shows when the visual has no icon layers.
    pub icon: u32,
    /// The local `VisualDesc`'s icon layers (layer id, image, shift colour); see Known Gaps:
    /// retail's `GetTotalIconDesc` merges the DAT parent's in too.
    pub icon_layers: Vec<(u32, u32, u32)>,
    pub quantity: u32,
    pub stackable: bool,
    pub max_stack: u32,
    /// Durability (current, max) when the item has one.
    pub durability: Option<(u32, u32)>,
    /// `WeenieDesc` value (int 0x264).
    pub value: Option<i32>,
}

impl Client {
    /// `ggmCWInterface::CallIFun_*` on `id`'s process: the return slot read back.
    fn object_query(
        &mut self,
        id: u64,
        function: u32,
        args: &[IFunArg],
    ) -> Result<IFunResult, String> {
        let scripts = self.scripts.as_mut().ok_or("no script VM")?;
        let result = scripts
            .host
            .query(id, function, args, IFunReturn::Word)
            .map_err(|e| format!("{e:?}"));
        let outcome = result
            .as_ref()
            .map(|r| r.outcome.clone())
            .map_err(Clone::clone);
        let mut out = Vec::new();
        self.script_outcome(id, None, outcome, &mut out);
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(out);
        }
        let result = result?;
        if result.outcome.status != InterpStatus::Terminated {
            return Err(format!(
                "{id:#x} {function:#010x}: {:?} {:?}",
                result.outcome.status, result.outcome.fault
            ));
        }
        Ok(result)
    }

    fn object_word(&mut self, id: u64, function: u32, args: &[IFunArg]) -> Result<u32, String> {
        Ok(self.object_query(id, function, args)?.retval as u32)
    }

    /// What `ItemDisplay::Update` shows of item `id` (its process answers).
    pub fn item_display(&mut self, id: u64) -> Result<ItemDisplay, String> {
        let quantity = self.object_word(id, GET_QUANTITY, &[])?;
        let icon = self.object_word(id, GET_ICON_ID, &[])?;
        let named = self.object_query(
            id,
            GET_POTENTIALLY_PLURALIZED_NAME,
            &[IFunArg::Int(quantity)],
        )?;
        let name = self
            .returned_string_info(named.retval as u32)
            .unwrap_or_default();
        let stackable = self.object_word(id, GET_STACKABLE, &[])? != 0;
        let max_stack = self.object_word(id, GET_MAX_STACK_SIZE, &[])?;
        let durability = if self.object_word(id, HAS_DURABILITY, &[])? != 0 {
            Some((
                self.object_word(id, GET_DURABILITY_LEVEL, &[])?,
                self.object_word(id, GET_MAX_DURABILITY_LEVEL, &[])?,
            ))
        } else {
            None
        };
        let object = self.session.objects().get(id);
        Ok(ItemDisplay {
            name,
            icon,
            icon_layers: object
                .map(|o| o.appearance.icon.clone())
                .unwrap_or_default(),
            quantity,
            stackable,
            max_stack,
            durability,
            value: object
                .and_then(|o| o.qualities.weenie.as_ref())
                .and_then(|w| w.value),
        })
    }

    /// `gmCWInterface::GetMaxContainerSize` (`gmEntity` 0xC: int 280 through the effects):
    /// how many items container `id` holds.
    pub fn container_capacity(&mut self, id: u64) -> Result<u32, String> {
        self.object_word(id, GET_MAX_CONTAINER_SIZE, &[])
    }

    /// The `StringInfo` a script returned by reference.
    fn returned_string_info(&self, reference: u32) -> Option<StringInfo> {
        let host = self.scripts.as_ref()?.host.vm();
        let object = host.refs.heap(reference).ok()?;
        match native_for_wire(object.native_type, object.native.as_ref()?)? {
            ac2_net::protocol::NativeObject::StringInfo(info) => Some(info),
            _ => None,
        }
    }
}
