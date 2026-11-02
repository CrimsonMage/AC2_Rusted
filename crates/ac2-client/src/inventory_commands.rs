//! The inventory panels' requests (`gmCWInterface::MoveItem`, `EquipItem`, `UnEquipItem`,
//! `UpdateOpen`, `LootAllFromContainer`, `TransmuteAllFromContainer`): each is its IFun on the
//! player's script (`ggmCWInterface::CallIFun_*`), whose `CContainer` / `CInventorySystem` code
//! builds the `InvMoveDesc` / `InvEquipDesc` / `InvTakeAllDesc` and sends it (`WM_Inventory`).
//! The client only calls the IFun with the arguments the wrapper pushes.

use ac2_script::host::IFunArg;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::gameplay::Command;
use crate::scripts::{
    IFUN_EQUIP_ITEM, IFUN_LOOT_ALL_FROM_CONTAINER, IFUN_MOVE_ITEM,
    IFUN_TRANSMUTE_ALL_FROM_CONTAINER, IFUN_UNEQUIP_ITEM, IFUN_UPDATE_OPEN,
};

impl Client {
    /// One of the inventory commands, as its IFun on the player's script.
    pub(crate) fn inventory_command(&mut self, command: &Command) -> Vec<SessionEvent> {
        let (function, args) = match command {
            // `CallIFun_MoveItem(item, fromID, fromSlot, targetID, targetSlot, quantity)`.
            Command::MoveItem {
                item,
                from,
                from_slot,
                target,
                target_slot,
                quantity,
            } => (
                IFUN_MOVE_ITEM,
                vec![
                    IFunArg::Long(*item),
                    IFunArg::Long(*from),
                    IFunArg::Int(*from_slot as u32),
                    IFunArg::Long(*target),
                    IFunArg::Int(*target_slot as u32),
                    IFunArg::Int(*quantity as u32),
                ],
            ),
            Command::EquipItem { item, location } => (
                IFUN_EQUIP_ITEM,
                vec![IFunArg::Long(*item), IFunArg::Int(*location)],
            ),
            Command::UnequipItem { item, target_slot } => (
                IFUN_UNEQUIP_ITEM,
                vec![IFunArg::Long(*item), IFunArg::Int(*target_slot as u32)],
            ),
            // `gmExternalContainerUI::ListenToMessage`: the container's own weenie gets the
            // call, with the player as the updater (`UpdateOpen(updater, open)`).
            Command::SetContainerOpen { container, open } => {
                let Some(player) = self.session.player() else {
                    return Vec::new();
                };
                let args = [IFunArg::Long(player), IFunArg::Int(u32::from(!*open))];
                return self.container_ifun(*container, IFUN_UPDATE_OPEN, &args);
            }
            Command::LootAll(container) => (
                IFUN_LOOT_ALL_FROM_CONTAINER,
                vec![IFunArg::Long(*container)],
            ),
            Command::TransmuteAll(container) => (
                IFUN_TRANSMUTE_ALL_FROM_CONTAINER,
                vec![IFunArg::Long(*container)],
            ),
            _ => return Vec::new(),
        };
        self.script_request(function, &args)
    }

    /// `ggmCWInterface::CallIFun_*` on `container`'s own process (not the player's), its output
    /// routed as a call of the VM.
    fn container_ifun(
        &mut self,
        container: u64,
        function: u32,
        args: &[IFunArg],
    ) -> Vec<SessionEvent> {
        let Some(scripts) = self.scripts.as_mut() else {
            return Vec::new();
        };
        if scripts.host.instance(container).is_none() {
            return Vec::new();
        }
        let result = scripts
            .host
            .call_ifun(container, function, args)
            .map_err(|e| format!("{e:?}"));
        let mut out = Vec::new();
        self.script_outcome(
            container,
            Some(function),
            result.map(|r| r.outcome),
            &mut out,
        );
        out
    }
}
