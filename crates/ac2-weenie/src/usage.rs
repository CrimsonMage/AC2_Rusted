//! Using objects: the `CUsageSystem` script (Legions WLib 0x56000005) that builds the
//! `UsageDesc` a use sends, and shows its outcome.
//!
//! Not mirrored: `CUsageSystem` 0xB (`SanityCheckUse`, the client's own permission checks
//! through the item's `UsageAction`s), the multi-item collection `HandleItemUsage` starts for
//! an item that needs targets (`CUsageSystem` 0x3/0x5, `SendNotice_SetTargetingPointer`; the UI
//! sends the use with its target directly), the inventory "in use" state (`CUsageSystem` 0x4)
//! and the error strings `CUsageSystem` 0x2 looks up by error code.

use ac2_net::protocol::{ScriptRequest, StringInfo, UsageDesc, USAGE_FLAG_SILENT};

use crate::player_script::GameplayNotice;

/// `gmCEntity` 0x1A's display type for usage messages.
pub const DISPLAY_USAGE: u32 = 8;
/// String table `0x25000014` (enum 0x41000019, the usage strings).
const TABLE_USAGE: u32 = 0x2500_0014;
/// "You have nothing selected" (`CPlayer` 0x79).
const STRING_NOTHING_SELECTED: u32 = 0xAE62_433B;

/// `CUsageSystem` 0x9 (`HandleItemUsage`) and 0x3 (`TryToAddToUsageList`): `player` uses
/// `item`, on `target` (0 for none). Using the player itself sends nothing.
pub fn use_item(player: u64, item: u64, target: u64) -> Option<ScriptRequest> {
    (item != player).then(|| ScriptRequest::Use(UsageDesc::new(player, item, target)))
}

/// `CPlayer` 0x79 (`UseSelection`) with nothing selected: the usage message it shows.
pub fn nothing_selected() -> GameplayNotice {
    GameplayNotice::DisplayStringInfo {
        kind: DISPLAY_USAGE,
        text: StringInfo {
            string_id: STRING_NOTHING_SELECTED,
            table_id: TABLE_USAGE,
            ..StringInfo::default()
        },
    }
}

/// `CUsageSystem` 0xA: the server's outcome. Success shows the desc's message unless its
/// silent flag is set (`UsageDesc` 0x1B); an error goes to `CUsageSystem` 0x2.
pub fn usage_done(desc: UsageDesc) -> Option<GameplayNotice> {
    if desc.error != 0 {
        return Some(GameplayNotice::UsageError {
            error: desc.error,
            item: desc.item,
            target: desc.target,
        });
    }
    (desc.flags & USAGE_FLAG_SILENT == 0).then_some(GameplayNotice::DisplayStringInfo {
        kind: DISPLAY_USAGE,
        text: desc.message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_use_names_the_player_item_and_target() {
        assert_eq!(
            use_item(1, 2, 3),
            Some(ScriptRequest::Use(UsageDesc::new(1, 2, 3)))
        );
        assert_eq!(use_item(1, 1, 0), None);
    }

    #[test]
    fn outcomes_show_the_message_or_the_error() {
        let mut desc = UsageDesc::new(1, 2, 0);
        desc.message.string_id = 7;
        assert_eq!(
            usage_done(desc.clone()),
            Some(GameplayNotice::DisplayStringInfo {
                kind: 8,
                text: desc.message.clone()
            })
        );
        desc.flags |= USAGE_FLAG_SILENT;
        assert_eq!(usage_done(desc.clone()), None);
        desc.error = 0x2711;
        assert_eq!(
            usage_done(desc),
            Some(GameplayNotice::UsageError {
                error: 0x2711,
                item: 2,
                target: 0
            })
        );
    }
}
