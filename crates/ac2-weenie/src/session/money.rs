//! The money bag (`gmMoneyBagUI`): the player's money (int 3100, `MoneySystem` 0x4), the two
//! requests the bag sends (`gmCWInterface::DragToMoneyBag` -> `CPlayer` 0xD0 -> `CMoneySystem`
//! 0x4 -> `WM_Money` 0x7; `gmCWInterface::DragFromMoneyBag` -> `CPlayer` 0xD5 ->
//! `CMoneySystem` 0x5 -> `WM_Money` 0x8) and the server's answers (`WM_Money` 0x6 and 0x9).

use ac2_net::protocol::{PackValue, ScriptCall};

use super::gameplay::player_qualities;
use super::{OnlineSession, SessionEvent};
use crate::items::item_quality::{MONEY, QUANTITY};
use crate::player_script::GameplayNotice;

/// `WM_Money` 0x7 / 0x8: the requests.
const DRAG_TO_MONEY_BAG: u32 = 0x820F_0002;
const DRAG_FROM_MONEY_BAG: u32 = 0x820F_0001;
/// `WM_Money` 0x6 / 0x9: the answers.
const MONEY_BAG_DONE: u32 = 0x820F_0006;
const MONEY_ITEM_MADE: u32 = 0x820F_0009;
/// `CM_Inventory` codes the bag's notices carry: an item left for the bag, one came out.
const ITEM_INTO_BAG: u32 = 0x4000_0005;
const ITEM_FROM_BAG: u32 = 0x4000_0004;
/// `gmWErrorType`: a bad quantity, not enough money.
pub const MONEY_BAD_QUANTITY: u32 = 2002;
pub const MONEY_NOT_ENOUGH: u32 = 2003;

/// Whether `function` is a `WM_Money` call the stand-in answers.
pub(super) fn is_money_call(function: u32) -> bool {
    matches!(function, MONEY_BAG_DONE | MONEY_ITEM_MADE)
}

impl OnlineSession {
    /// `MoneySystem` 0x4: the player's money (int 3100, 0 before its qualities come).
    pub fn money(&self) -> i32 {
        player_qualities(&self.world).inq_int(MONEY).unwrap_or(0)
    }

    /// `gmCWInterface::DragToMoneyBag(item, quantity, fromSlot, fromContainer)`: `CMoneySystem`
    /// 0x4 checks the drag (`MoneySystem` 0x6; mirrored: the quantity is at least one and at
    /// most the stack's), says the item left its slot (`SendNotice_UpdateContainer(player,
    /// 0x40000005, item, fromSlot)`), ghosts the bag and sends `WM_Money` 0x7.
    pub fn drag_to_money_bag(
        &mut self,
        item: u64,
        quantity: u32,
        from_slot: i32,
        from_container: u64,
    ) -> Result<Vec<SessionEvent>, u32> {
        let player = self.world.player().ok_or(MONEY_BAD_QUANTITY)?;
        let stack = self
            .world
            .objects
            .get(item)
            .and_then(|o| o.qualities.inq_int(QUANTITY))
            .unwrap_or(1);
        if quantity < 1 || quantity > stack as u32 {
            return Err(MONEY_BAD_QUANTITY);
        }
        let call = ScriptCall::new(
            DRAG_TO_MONEY_BAG,
            vec![
                PackValue::LongInt(from_container),
                PackValue::Int(from_slot as u32),
                PackValue::Int(quantity),
                PackValue::LongInt(item),
            ],
        )
        .expect("WM_Money 0x7's layout");
        self.send(call.to_event());
        Ok(vec![
            SessionEvent::Notice(GameplayNotice::UpdateContainer {
                container: player,
                code: ITEM_INTO_BAG,
                item,
                slot: from_slot,
            }),
            SessionEvent::Notice(GameplayNotice::GhostMoneyBag(true)),
        ])
    }

    /// `gmCWInterface::DragFromMoneyBag(quantity, toSlot, toContainer)`: `CMoneySystem` 0x5
    /// checks it (`MoneySystem` 0x2: at least one, no more than the player has; its room check,
    /// `InventorySystem` 0x1E, is not mirrored), ghosts the bag and sends `WM_Money` 0x8.
    pub fn drag_from_money_bag(
        &mut self,
        quantity: u32,
        to_slot: i32,
        to_container: u64,
    ) -> Result<Vec<SessionEvent>, u32> {
        if quantity < 1 {
            return Err(MONEY_BAD_QUANTITY);
        }
        if (self.money() as u32) < quantity {
            return Err(MONEY_NOT_ENOUGH);
        }
        let call = ScriptCall::new(
            DRAG_FROM_MONEY_BAG,
            vec![
                PackValue::LongInt(to_container),
                PackValue::Int(to_slot as u32),
                PackValue::Int(quantity),
            ],
        )
        .expect("WM_Money 0x8's layout");
        self.send(call.to_event());
        Ok(vec![SessionEvent::Notice(GameplayNotice::GhostMoneyBag(
            true,
        ))])
    }

    /// `WM_Money` 0x6 (`CMoneySystem` 0x7: the bag unghosts) and 0x9 (`CMoneySystem` 0x3: a
    /// made item lands at its slot of the player's pack, `SendNotice_UpdateContainer(player,
    /// 0x40000004, item, slot)`, then the bag unghosts). `None` for other functions.
    pub(super) fn receive_money(
        &mut self,
        function: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        if !is_money_call(function) {
            return None;
        }
        let call = ScriptCall::decode(function, arguments)?.ok()?;
        let mut out = Vec::new();
        if let [_, PackValue::Int(slot), PackValue::LongInt(item)] = call.arguments[..] {
            if let (true, Some(player)) = (item != 0, self.world.player()) {
                out.push(SessionEvent::Notice(GameplayNotice::UpdateContainer {
                    container: player,
                    code: ITEM_FROM_BAG,
                    item,
                    slot: slot as i32,
                }));
            }
        }
        out.push(SessionEvent::Notice(GameplayNotice::GhostMoneyBag(false)));
        Some(out)
    }

    /// `CPlayer` 0x85 (`OnStatChanged`) on int 3100 -> `CPlayer` 0x71:
    /// `SendNotice_UpdateMoney(money)`.
    pub(super) fn money_changed(&self, table: u32, quality: u32) -> Option<SessionEvent> {
        (table == 4 && quality == MONEY)
            .then(|| SessionEvent::Notice(GameplayNotice::UpdateMoney(self.money() as u32)))
    }
}
