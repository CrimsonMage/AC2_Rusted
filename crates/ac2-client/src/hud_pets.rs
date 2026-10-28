//! `gmPetUI` on the HUD: the pet notices are run by `ac2_ui::pet_notice`, and the panel's requests
//! become the player's script calls (`gmCWInterface::RequestPet*`, `SetSelection`).

use ac2_ui::PetAction;

use crate::gameplay::Command;

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
