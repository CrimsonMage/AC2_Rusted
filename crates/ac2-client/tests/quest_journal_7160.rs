//! The quest journal over the Legions server double (`AC2_DAT_DIR`): the quests the login
//! message carries (`WM_Player` 0x27 argument 3, a `gmQuestInfoList`) reach the player's script,
//! and the journal lists the underway one on its Underway page and the completed one on its
//! Finished page, even when the login message came before the HUD was built (its list notice
//! then had no journal to reach). Needs a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::ToolbarChild;
use hud::Rig;
use legions_server_double::STARTER_QUEST;

#[test]
fn the_login_quests_fill_the_journal() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    // The login state comes before the HUD is built, as from a server at entry.
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_quests();
        server.send_init_player();
    }
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ToolbarChild::Quest, true);
    rig.idle(8);
    let (underway, finished) = rig.view.quest_ids().expect("the quest panel");
    assert_eq!(underway, [STARTER_QUEST]);
    assert_eq!(finished, [STARTER_QUEST + 1]);
}
