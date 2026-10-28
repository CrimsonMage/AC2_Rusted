//! What stands in each pregame viewport's scene: the `UIScene`'s own physics objects
//! (`UIScene::AddEntity`) and what the screen adds.
//!
//! - Character selection's `Viewport`: one arm (`armdid`) per character slot at the arm's frame
//!   (`gmCharacterSelectionUI::InitArms`), each character's figure made from its total
//!   appearance (`AddPhysObjToArm`: `VisualDesc::GetTotalVisualDesc`, `makePartGroupObject`, or
//!   `nologininfo` when that fails) and hung from its arm's holding location 1
//!   (`UIScenePhysObjEntity::AddChild`). Only figures answer a pick (`_physObjTable`).
//! - Avatar selection: each race and sex's avatar (`gmAvatarSelectionUI::InitEntities`:
//!   `makeObject(GetPhysObj)`, going to its line-up mode 0x41000006) hung from the land mesh's
//!   (`CHARGEN_LANDMESH`, entity 0x8100000B) holding location for its sex and race, holding its
//!   race's objects (`objInLocHash`) at their locations.
//! - Avatar modification: the current result's avatar at the scene origin
//!   (`gmAvatarModificationUI::Init`, mode 0x41000008).

use ac2_math::{Frame, Vec3};
use ac2_physics::local_visual_desc;
use ac2_render::ui_scene::data::SceneData;
use ac2_render::ui_scene::figure::{figure_key as key, scene_figures, At, Figure};
use ac2_weenie::Appearance;

use super::{identity, CharacterSelectionUi, PregameUi};
use crate::SceneViewport;

/// `AddPhysObjToArm`: figures hang from the arm's holding location 1.
const ARM_LOCATION: u32 = 1;
/// The land mesh avatars stand on (`UIScene::GetEntity(0x8100000B)`).
const LAND_MESH: &str = "CHARGEN_LANDMESH";
/// The mode avatar selection's avatars go to (`DoMode`), and avatar modification's.
pub const MODE_AVATAR_SELECTION: u32 = 0x4100_0006;
pub const MODE_AVATAR_MODIFICATION: u32 = 0x4100_0008;

/// The figures of `viewport`'s scene, parents before the figures they hold.
pub fn figures(ui: &PregameUi, viewport: &SceneViewport, scene: &SceneData) -> Vec<Figure> {
    let did = viewport.scene;
    let mut out: Vec<Figure> = scene_figures(did, scene);
    let root = Some(viewport.root);
    if let Some(selection) = ui.character_selection().filter(|s| s.viewport() == root) {
        character_selection(ui, selection, did, &mut out);
    }
    if let Some(avatars) = ui.avatar_selection().filter(|s| s.viewport() == root) {
        let land = scene
            .entities
            .iter()
            .position(|e| e.name.as_deref() == Some(LAND_MESH));
        for (i, avatar) in avatars.avatars().iter().enumerate() {
            let (Some(land), Some(location)) = (land, avatar.location) else {
                continue;
            };
            let index = out.len();
            out.push(Figure {
                key: key(did, "avatar", i),
                object: avatar.phys_obj,
                local: avatar.appearance.as_ref().map(local_visual_desc),
                at: At::Held {
                    parent: land,
                    location,
                },
                pick: Some(i),
                mode: Some(MODE_AVATAR_SELECTION),
            });
            for (j, &(location, object)) in avatar.held.iter().enumerate() {
                out.push(Figure {
                    key: key(did, "held", i * 64 + j),
                    object,
                    local: None,
                    at: At::Held {
                        parent: index,
                        location,
                    },
                    pick: None,
                    mode: None,
                });
            }
        }
    }
    if let Some(modification) = ui.avatar_modification().filter(|m| m.viewport() == root) {
        if modification.phys_obj() != 0 {
            out.push(Figure {
                key: key(did, "avatar", 0),
                object: modification.phys_obj(),
                local: modification.applied().map(local_visual_desc),
                at: At::Scene(Frame::IDENTITY, Vec3::ONE),
                pick: None,
                mode: Some(MODE_AVATAR_MODIFICATION),
            });
        }
    }
    out
}

/// The arms and the figures on them.
fn character_selection(
    ui: &PregameUi,
    selection: &CharacterSelectionUi,
    did: u32,
    out: &mut Vec<Figure>,
) {
    let Some(arm) = selection.scene(ui.tree()).1 else {
        return;
    };
    let first_arm = out.len();
    for (slot, entity) in selection.arms().arms().iter().enumerate() {
        out.push(Figure {
            key: key(did, "arm", slot),
            object: arm,
            local: None,
            at: At::Scene(entity.frame(), Vec3::ONE),
            pick: None,
            mode: None,
        });
    }
    let characters = ui.data().characters();
    for occupant in selection.occupants() {
        let (object, local) = match occupant.stand_in {
            Some(stand_in) => (stand_in, None),
            None => {
                let Some(identity) = characters.and_then(|set| identity(set, occupant.slot)) else {
                    continue;
                };
                let appearance = Appearance::from_delta(&identity.visual);
                let local = ac2_physics::local_visual_desc(&appearance);
                (appearance.parent, Some(local))
            }
        };
        out.push(Figure {
            key: key(did, "figure", occupant.slot),
            object,
            local,
            at: At::Held {
                parent: first_arm + occupant.slot,
                location: ARM_LOCATION,
            },
            pick: Some(occupant.slot),
            mode: None,
        });
    }
}
