//! `gmAvatarSelectionUI`: character generation's first screen, every race and sex standing in
//! a 3D scene (`AvatarSelectionViewport`). On entering character generation each race and sex
//! gets a fresh result with the script's defaults (`InitRaces`, `InitGenders`); one avatar per
//! result stands at its race and sex's location in the land, holding its objects
//! (`InitEntities`). The pointer over an avatar names it (`RaceDescField`'s race and sex state,
//! `MouseOnObject` / `MouseOffObject`); a press on one plays its selected animation and makes its
//! result current, and when the animation ends avatar modification follows. Cancel goes back to
//! character selection.
//!
//! The scene is the client's: it renders [`AvatarSelectionUi::avatars`] in
//! [`AvatarSelectionUi::viewport`], answers the pick and reports the animation's end.
//! The viewport carries `ID_AvSelect_TT_3DAvatar` as its tooltip while an avatar is under the
//! pointer (`MouseOnObject`, cleared by `MouseOffObject`), and the bottom field's `primSong`
//! plays when the screen opens ([`PregameRequest::Music`]).

use ac2_net::protocol::GmRaceSexInfo;
use ac2_weenie::Appearance;

use crate::dialog::DIALOG_WAIT;
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::pregame::chargen::{new_char_gen, result_key, AVATAR_LOCATIONS, RACE_NAMES, SEX_NAMES};
use crate::pregame::flow::{MODE_AVATAR_MODIFICATION, MODE_CHARACTER_SELECTION};
use crate::pregame::{Ctx, PregameRequest, LAYOUT_AVATAR_SELECTION};
use crate::token::{
    tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_MOVE_TOP, MSG_MOUSE_PRESS_TOP, STATELESS,
};

/// One race and sex's avatar (its physics object, appearance and held objects come from its
/// result and race/sex info).
#[derive(Debug, Clone, PartialEq)]
pub struct Avatar {
    pub race: u32,
    pub sex: u32,
    /// `GetPhysObj`: the avatar's physics object.
    pub phys_obj: u32,
    /// `_objInLocHash`: (location, object) held.
    pub held: Vec<(u32, u32)>,
    /// `EnumMapper::GetEnum(0x4000000B, sex + race)` (`MALEHUMAN`): the land's holding
    /// location it stands at.
    pub location: Option<u32>,
    /// `RaceDescField`'s state naming it (`Tokenize(lowercase race + sex)`).
    pub state: u32,
    /// `gmCharGenResult::InqAppliedAppearanceVDesc` of the race and sex's result, applied to
    /// the avatar (`InitEntities`).
    pub appearance: Option<Appearance>,
}

/// `gmAvatarSelectionUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct AvatarSelectionUi {
    viewport: Option<usize>,
    top: Option<usize>,
    bottom: Option<usize>,
    avatars: Vec<Avatar>,
    /// `_objectUnderMouse`.
    under_mouse: Option<usize>,
    /// `_selectingObject`, `_waitForAnim`.
    selecting: bool,
    wait_for_anim: Option<usize>,
    wait_context: u32,
}

impl AvatarSelectionUi {
    /// `gmAvatarSelectionUI::gmAvatarSelectionUI`: the roots, then `InitRaces`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let mut ui = Self::default();
        if let Some(layout) = ctx.assets.layout(LAYOUT_AVATAR_SELECTION) {
            let mut root = |name: &str| ctx.tree.create_root(layout.clone(), tokenize(name));
            ui.viewport = root("AvatarSelectionViewport");
            ui.top = root("AvatarSelectionFieldTop");
            ui.bottom = root("AvatarSelectionFieldBottom");
        }
        ui.init_races(ctx);
        // `SoundManager::Play` of the bottom field's `primSong` on `Channel_Main`.
        if let Some(did) = ui.prim_song(ctx) {
            ctx.requests.push(PregameRequest::Music(did));
        }
        ui
    }

    /// `UIElement::GetAttribute_DataID(_avatarSelectionFieldBottom, Tokenize("primSong"))`.
    fn prim_song(&self, ctx: &Ctx<'_>) -> Option<u32> {
        match ctx.tree.get(self.bottom?)?.attribute(tokenize("primSong")) {
            Some(AttributeValue::DataId(did)) => Some(*did),
            _ => None,
        }
    }

    /// The framework's destructor: roots removed, its dialog closed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        for root in [self.viewport, self.top, self.bottom].into_iter().flatten() {
            ctx.tree.remove_root(root);
        }
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
    }

    /// `gmAvatarSelectionUI::InitRaces`: on entering generation every race and sex gets a fresh
    /// result (`InitGenders`), then the avatars are made.
    fn init_races(&mut self, ctx: &mut Ctx<'_>) {
        let Some(races) = ctx.system.race_sex_data() else {
            return;
        };
        let info = ctx.system.race_sex_info();
        if !ctx.data.in_char_gen() {
            for (race, sexes) in &races {
                for &sex in sexes {
                    if *race == 0 {
                        continue;
                    }
                    let cgr = new_char_gen(ctx.system, *race, sex);
                    ctx.data.set_char_gen_result(result_key(*race, sex), cgr);
                }
            }
        }
        ctx.data.set_in_char_gen(true);
        self.init_entities(ctx, &races, &info);
    }

    /// `gmAvatarSelectionUI::InitEntities`: one avatar per race and sex with a result.
    fn init_entities(
        &mut self,
        ctx: &mut Ctx<'_>,
        races: &[(u32, Vec<u32>)],
        info: &[(u32, GmRaceSexInfo)],
    ) {
        if self.viewport.is_none() {
            return;
        }
        for (race, sexes) in races {
            let race_name = ctx
                .system
                .enum_string(RACE_NAMES, *race)
                .unwrap_or_default();
            for &sex in sexes {
                let key = result_key(*race, sex);
                if ctx.data.char_gen_result(key).is_none() {
                    continue;
                }
                let phys_obj = ctx.system.phys_obj(*race, sex);
                if phys_obj == 0 {
                    continue;
                }
                let sex_name = ctx.system.enum_string(SEX_NAMES, sex).unwrap_or_default();
                let name = format!("{race_name}{sex_name}");
                let appearance = ctx
                    .data
                    .char_gen_result(key)
                    .cloned()
                    .and_then(|cgr| ctx.system.applied_appearance(&cgr));
                self.avatars.push(Avatar {
                    appearance,
                    race: *race,
                    sex,
                    phys_obj,
                    held: info
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, i)| i.obj_in_loc.clone())
                        .unwrap_or_default(),
                    location: ctx
                        .system
                        .enum_value(AVATAR_LOCATIONS, &format!("{sex_name}{race_name}")),
                    state: tokenize(&name.to_lowercase()),
                });
            }
        }
    }

    /// The avatars standing in the scene.
    pub fn avatars(&self) -> &[Avatar] {
        &self.avatars
    }

    /// The avatar under the pointer (`_objectUnderMouse`), as the last pick answered.
    pub fn under_mouse(&self) -> Option<usize> {
        self.under_mouse
    }

    /// The `AvatarSelectionViewport` root the avatars stand in.
    pub fn viewport(&self) -> Option<usize> {
        self.viewport
    }

    /// `gmAvatarSelectionUI::MakePleaseWaitDialog`.
    fn make_please_wait_dialog(&mut self, ctx: &mut Ctx<'_>) {
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.wait_context = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
    }

    fn race_desc(&self, ctx: &mut Ctx<'_>, state: u32) {
        if let Some(field) = self
            .bottom
            .and_then(|b| ctx.tree.child_recursive(b, tokenize("RaceDescField")))
        {
            ctx.tree.set_state(field, state);
        }
    }

    /// `gmAvatarSelectionUI::ListenToMessage` for an element's message.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if self.viewport.is_none() {
            return;
        }
        if message.element_id == tokenize("AvatarSelectionViewport") {
            if message.message == MSG_MOUSE_MOVE_TOP
                && !self.selecting
                && self.wait_for_anim.is_none()
            {
                // `UIScene::StartSelection` at the pointer.
                self.selecting = true;
            }
            if message.message == MSG_MOUSE_PRESS_TOP {
                if let Some(avatar) = self.under_mouse {
                    // The avatar's selected animation (behavior 0x41000005), then its result
                    // is the one being made.
                    self.wait_for_anim = Some(avatar);
                    let a = &self.avatars[avatar];
                    ctx.data.set_current(result_key(a.race, a.sex));
                }
            }
        }
        if message.element_id == tokenize("CancelButton")
            && message.message == MSG_BUTTON_RELEASE_ON
        {
            self.make_please_wait_dialog(ctx);
            ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
        }
    }

    /// Whether the scene should pick the avatar under the pointer (`_selectingObject`).
    pub fn wants_pick(&self) -> bool {
        self.selecting && self.wait_for_anim.is_none()
    }

    /// The scene's pick: the avatar under the pointer (`ListenToMessage(UIMESSAGE_GLOBAL_LOOP)`,
    /// `MouseOnObject` / `MouseOffObject`).
    pub(crate) fn finish_pick(&mut self, ctx: &mut Ctx<'_>, avatar: Option<usize>) {
        self.selecting = false;
        let avatar = avatar.filter(|&a| a < self.avatars.len());
        if avatar == self.under_mouse {
            return;
        }
        if self.under_mouse.is_some() {
            self.mouse_off_object(ctx);
        }
        self.under_mouse = avatar;
        if let Some(avatar) = avatar {
            self.mouse_on_object(ctx, avatar);
        }
    }

    /// `gmAvatarSelectionUI::MouseOffObject`: the viewport's tooltip cleared, `RaceDescField`
    /// back to no state.
    fn mouse_off_object(&mut self, ctx: &mut Ctx<'_>) {
        if let Some(viewport) = self.viewport {
            ctx.tree.set_tooltip(viewport, None);
            self.race_desc(ctx, STATELESS);
        }
    }

    /// `gmAvatarSelectionUI::MouseOnObject`: the viewport's tooltip (`ID_AvSelect_TT_3DAvatar`)
    /// and `RaceDescField` in the avatar's race and sex state (`_StateTable`).
    fn mouse_on_object(&mut self, ctx: &mut Ctx<'_>, avatar: usize) {
        if let Some(viewport) = self.viewport {
            let tip = ctx.strings.get("ID_AvSelect_TT_3DAvatar");
            ctx.tree.set_tooltip(viewport, Some(tip));
            let state = self.avatars[avatar].state;
            self.race_desc(ctx, state);
        }
    }

    /// The avatar whose selected animation is playing (`_waitForAnim`).
    pub fn selected_animation(&self) -> Option<usize> {
        self.wait_for_anim
    }

    /// The selected animation ended: avatar modification follows.
    pub(crate) fn animation_done(&mut self, ctx: &mut Ctx<'_>) {
        if self.wait_for_anim.take().is_none() {
            return;
        }
        self.make_please_wait_dialog(ctx);
        ctx.flow.queue_mode(MODE_AVATAR_MODIFICATION, "");
    }
}
