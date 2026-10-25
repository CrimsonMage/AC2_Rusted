//! `gmVitalsUI`: the status bar's vitals, each a [`VitalDisplay`]: health with the PK damage
//! under it, vigor with the PK vigor loss under it, focus (a hero's; `hero` / `not_hero` on its
//! meter, its text only for a hero) and the experience bar. A right click (`mouse_press_top` with
//! button `0x40000004`) on the health, vigor or focus field shows or hides its numbers.
//!
//! Not modelled yet: the fields' tooltips (`SetTooltip`, `ID_Vital_TT_*`).

use crate::element::UiTree;
use crate::meter::{VitalDisplay, VitalText};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `CM_Character::SendNotice_UpdateVital`'s vitals.
pub const VITAL_HEALTH: u32 = 0x4000_0001;
pub const VITAL_VIGOR: u32 = 0x4000_0002;
pub const VITAL_EXPERIENCE: u32 = 0x4000_0003;
pub const VITAL_FOCUS: u32 = 0x4000_0004;
pub const VITAL_PK_DAMAGE: u32 = 0x4000_0005;
pub const VITAL_PK_VIGOR_LOSS: u32 = 0x4000_0006;

/// The right mouse button (`StartInput`'s `0x40000004`).
const RIGHT_BUTTON: i32 = 0x4000_0004;

/// `gmVitalsUI`'s state.
#[derive(Debug, Clone)]
pub struct VitalsUi {
    health_field: Option<usize>,
    vigor_field: Option<usize>,
    focus_field: Option<usize>,
    health: Option<VitalDisplay>,
    pk_damage: Option<VitalDisplay>,
    vigor: Option<VitalDisplay>,
    pk_vigor_loss: Option<VitalDisplay>,
    experience: Option<VitalDisplay>,
    focus: Option<VitalDisplay>,
    hero: bool,
}

impl VitalsUi {
    /// `gmVitalsUI::gmVitalsUI`: each field (`IUIManager` lookup) and its displays; the PK
    /// displays' text hidden.
    pub fn create(tree: &mut UiTree) -> Self {
        let field = |tree: &UiTree, name: &str| {
            tree.roots().iter().find_map(|&r| {
                if tree.get(r).is_some_and(|e| e.id() == id(name)) {
                    Some(r)
                } else {
                    tree.child_recursive(r, id(name))
                }
            })
        };
        let display = |tree: &UiTree, field: Option<usize>, meter: &str, text: &str| {
            let field = field?;
            Some(VitalDisplay::new(
                tree,
                tree.child_recursive(field, id(meter)),
                tree.child_recursive(field, id(text)),
            ))
        };
        let health_field = field(tree, "HealthField");
        let vigor_field = field(tree, "VigorField");
        let focus_field = field(tree, "FocusField");
        let experience_field = field(tree, "ExperienceField");
        let pk_damage = display(tree, health_field, "PKMeter", "PKText");
        let pk_vigor_loss = display(tree, vigor_field, "PKVigorlossMeter", "PKVigorlossText");
        for pk in [&pk_damage, &pk_vigor_loss].into_iter().flatten() {
            pk.set_text_state(tree, false);
        }
        Self {
            health: display(tree, health_field, "HealthMeter", "HealthText"),
            pk_damage,
            vigor: display(tree, vigor_field, "VigorMeter", "VigorText"),
            pk_vigor_loss,
            focus: display(tree, focus_field, "FocusMeter", "FocusText"),
            experience: display(tree, experience_field, "ExperienceMeter", "ExperienceText"),
            health_field,
            vigor_field,
            focus_field,
            hero: false,
        }
    }

    /// `RecvNotice_UpdateVital`: focus only for a hero.
    pub fn update_vital(
        &mut self,
        tree: &mut UiTree,
        vital: u32,
        cur: u64,
        max: u64,
        text: VitalText<'_>,
    ) {
        let display = match vital {
            VITAL_HEALTH => &mut self.health,
            VITAL_VIGOR => &mut self.vigor,
            VITAL_EXPERIENCE => &mut self.experience,
            VITAL_FOCUS if self.hero => &mut self.focus,
            VITAL_PK_DAMAGE => &mut self.pk_damage,
            VITAL_PK_VIGOR_LOSS => &mut self.pk_vigor_loss,
            _ => return,
        };
        if let Some(display) = display {
            display.update(tree, cur, max, text);
        }
    }

    /// `RecvNotice_UpdateHeroStatus`: the focus meter in `hero` with its numbers, or in
    /// `not_hero` full and without them.
    pub fn update_hero_status(
        &mut self,
        tree: &mut UiTree,
        hero: bool,
        cur: u64,
        max: u64,
        text: VitalText<'_>,
    ) {
        self.hero = hero;
        let Some(focus) = &mut self.focus else {
            return;
        };
        focus.set_meter_state(tree, id(if hero { "hero" } else { "not_hero" }));
        if hero {
            focus.set_text_state(tree, true);
            focus.update(tree, cur, max, text);
        } else {
            focus.set_text_state(tree, false);
            focus.update(tree, 1, 1, text);
        }
    }

    /// The vitals the player's script raised before the bar was listening (the latest of each,
    /// hero status first).
    pub fn apply_snapshot(
        &mut self,
        tree: &mut UiTree,
        vitals: &ac2_weenie::PlayerVitals,
        text: VitalText<'_>,
    ) {
        if let Some(focus) = vitals.focus.filter(|_| vitals.hero) {
            self.update_hero_status(tree, true, focus.current, focus.max, text);
        }
        for (vital, value) in [
            (VITAL_HEALTH, vitals.health),
            (VITAL_VIGOR, vitals.vigor),
            (VITAL_EXPERIENCE, vitals.experience),
            (VITAL_PK_DAMAGE, vitals.pk_damage),
            (VITAL_PK_VIGOR_LOSS, vitals.pk_vigor_loss),
        ] {
            if let Some(value) = value {
                self.update_vital(tree, vital, value.current, value.max, text);
            }
        }
    }

    /// `gmVitalsUI::ListenToMessage`: a right click on a field shows or hides its numbers (focus
    /// only for a hero).
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) {
        if message.message != MSG_MOUSE_PRESS_TOP || message.data != RIGHT_BUTTON {
            return;
        }
        let pressed = message.element;
        if pressed.is_some() && pressed == self.health_field {
            if let Some(health) = &self.health {
                health.toggle_text_state(tree);
            }
        }
        if pressed.is_some() && pressed == self.vigor_field {
            if let Some(vigor) = &self.vigor {
                vigor.toggle_text_state(tree);
            }
        }
        if pressed.is_some() && pressed == self.focus_field {
            if let Some(focus) = &self.focus {
                if self.hero {
                    focus.toggle_text_state(tree);
                } else {
                    focus.set_text_state(tree, false);
                }
            }
        }
    }
}
