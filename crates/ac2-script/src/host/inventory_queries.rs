//! The inventory IFuns the UI asks of the player's script (`gmCWInterface::DoIHaveItem`,
//! `InqSimpleInvByLocTable`, `InqAllInventoryIIDList`, `IsOnInventory`). The player's script keeps
//! the contents and equipment from the `WM_Inventory` messages it is sent, so these answer from
//! the VM; each returns `None` when the VM cannot answer (no script, a call that stopped for
//! data), and the caller falls back to the frame's mirror.

use ac2_net::protocol::ntype;

use super::ifun::{IFunArg, IFunResult, IFunReturn, OutValue};
use super::queries::ScriptQueries;
use super::ui_calls;
use crate::native::{NativeObject, RefKey};
use crate::vm::InterpStatus;

/// `ggmCWInterface::CallIFun_DoIHaveItem` (`gmCWInterface::DoIHaveItem`: the item, an int).
pub const DO_I_HAVE_ITEM: u32 = 0x00B4_0015;
/// `ggmCWInterface::CallIFun_InqSimpleInvByLocTable`: an `ALHash` the script fills, location
/// to item.
pub const INQ_SIMPLE_INV_BY_LOC_TABLE: u32 = 0x00B4_0014;
/// `ggmCWInterface::CallIFun_InqAllInventoryIIDList`: an `LList` of instance ids the script
/// fills.
pub const INQ_ALL_INVENTORY_IID_LIST: u32 = 0x00B4_011A;
/// `ggmCWInterface::CallIFun_IsOnInventory` (`gmCWInterface::IsOnInventory`: the item, an int).
pub const IS_ON_INVENTORY: u32 = 0x00B4_00D0;

/// The inventory queries, on any [`ScriptQueries`] (the client implements the port).
pub trait InventoryQueries: ScriptQueries {
    /// An inventory query on the player's script, its result when the VM ran it to the end.
    fn inventory_query(&mut self, function: u32, args: &[IFunArg]) -> Option<IFunResult> {
        let result = ui_calls::script_query(self, function, args, IFunReturn::Word)?;
        (result.outcome.status == InterpStatus::Terminated && result.outcome.fault.is_none())
            .then_some(result)
    }

    /// `gmCWInterface::DoIHaveItem`: the player lists `item` in a pack or wears it. `None`
    /// without an answer from the script.
    fn do_i_have_item(&mut self, item: u64) -> Option<bool> {
        let result = self.inventory_query(DO_I_HAVE_ITEM, &[IFunArg::Long(item)])?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::IsOnInventory`: the script's answer for `item` (see the money bag's drop
    /// test). `None` without an answer from the script.
    fn is_on_inventory(&mut self, item: u64) -> Option<bool> {
        let result = self.inventory_query(IS_ON_INVENTORY, &[IFunArg::Long(item)])?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::InqSimpleInvByLocTable`: each worn location and the item at it, in the
    /// script's table order. `None` without an answer from the script.
    fn equipped_by_location(&mut self) -> Option<Vec<(u32, u64)>> {
        let result = self.inventory_query(
            INQ_SIMPLE_INV_BY_LOC_TABLE,
            &[IFunArg::Filled(ntype::AL_HASH)],
        )?;
        match result.outs.first()? {
            Some(OutValue::Vm(NativeObject::Hash(hash))) => Some(
                hash.nodes()
                    .filter_map(|node| match node.key {
                        RefKey::Value(location) => Some((location as u32, node.value)),
                        RefKey::Ref { .. } => None,
                    })
                    .collect(),
            ),
            _ => None,
        }
    }

    /// `gmCWInterface::InqAllInventoryIIDList`: every item the player carries (its containers'
    /// and equipment's ids, the script's order). `None` without an answer from the script.
    fn inventory_ids(&mut self) -> Option<Vec<u64>> {
        let result = self.inventory_query(
            INQ_ALL_INVENTORY_IID_LIST,
            &[IFunArg::Filled(ntype::L_LIST)],
        )?;
        match result.outs.first()? {
            Some(OutValue::Values(ids)) => Some(ids.clone()),
            _ => None,
        }
    }
}

impl<P: ScriptQueries + ?Sized> InventoryQueries for P {}
