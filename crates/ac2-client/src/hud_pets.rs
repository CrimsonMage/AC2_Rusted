//! `gmPetUI` on the HUD: the pet notices reach the panel, and its messages and refresh become the
//! player's script calls (`gmCWInterface::RequestPet*`, `SetSelection`).

use ac2_ui::{PetAction, PetUi, UiTree};
use ac2_weenie::GameplayNotice;

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// `gmPetUI`'s notices: `AddPet` lists the pet, `PetIsUnderAttack` marks it unsafe and
/// `UpdatePetMode` takes the server's mode. `RemovePet` and `UpdatePetName` change nothing in
/// the panel (see `ac2_ui::pets`).
pub(crate) fn pet_notice(
    panel: &mut PetUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::AddPet {
            pet, name, mode, ..
        } => panel.add_pet(tree, *pet, &player.render_info(name), *mode),
        GameplayNotice::PetIsUnderAttack(pet) => panel.pet_under_attack(tree, *pet, now),
        GameplayNotice::UpdatePetMode { pet, mode } => panel.update_pet_mode(tree, *pet, *mode),
        _ => {}
    }
}

/// The panel's requests as gameplay commands, in order.
pub(crate) fn pet_commands(actions: Vec<PetAction>, commands: &mut Vec<Command>) {
    for action in actions {
        commands.push(match action {
            PetAction::Select(pet) => Command::Select(Some(pet)),
            PetAction::Attack(target) => Command::PetAttack(target),
            PetAction::Buff => Command::PetBuff,
            PetAction::Heal => Command::PetHeal,
            PetAction::Return => Command::PetReturn,
            PetAction::Mode { pet, mode } => Command::PetMode { pet, mode },
        });
    }
}
