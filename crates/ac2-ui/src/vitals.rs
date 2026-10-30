//! `gmVitalsUI`: the status bar's vitals, each a [`VitalDisplay`]: health with the PK damage
//! under it, vigor with the PK vigor loss under it, focus (a hero's; `hero` / `not_hero` on its
//! meter, its text only for a hero) and the experience bar. A left click (`mouse_press_top` with
//! button `0x40000004`) on the health, vigor or focus field shows or hides its numbers.
//!
//! The health, vigor and (for a hero) focus fields carry a tooltip, `ID_Vital_TT_Health` /
//! `_Vigor` / `_Focus` of the UI table with its `value` variable set to `ID_Vital_TT_Hide` while
//! the numbers show and `ID_Vital_TT_Show` while they are hidden (`gmVitalsUI::SetTooltip`); a
//! player who is no hero has none on the focus field (`UIElement::ClearTooltip`).

use crate::element::UiTree;
use crate::fellow_info::FellowStrings;
use crate::meter::{VitalDisplay, VitalText};
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::ui_string::UiString;

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

/// The left mouse button (`StartInput`'s `0x40000004`).
const LEFT_BUTTON: i32 = 0x4000_0004;

/// `gmVitalsUI::SetTooltip`: the field's tooltip, `string` of the UI table with `value` the
/// `Hide` string while the numbers `visible`, else the `Show` string.
fn tooltip(
    tree: &mut UiTree,
    strings: &dyn FellowStrings,
    visible: bool,
    element: Option<usize>,
    string: &str,
) {
    let Some(element) = element else {
        return;
    };
    let value = id(if visible {
        "ID_Vital_TT_Hide"
    } else {
        "ID_Vital_TT_Show"
    });
    let var = SkillVar::Info(Box::new(UiString::new(value, 0, Vec::new())));
    let text = strings.render(id(string), &[(id("value"), var)]);
    tree.set_tooltip(element, Some(text));
}

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
    /// The modern style's panel (`modern_vitals`); `None` in the classic style.
    modern: Option<crate::modern_vitals::ModernVitals>,
}

impl VitalsUi {
    /// The modern style's panel laid over the status bar (`modern_vitals`).
    pub fn set_modern(
        &mut self,
        tree: &mut UiTree,
        layouts: &[&std::sync::Arc<crate::layout::LayoutDesc>],
    ) {
        self.modern = crate::modern_vitals::ModernVitals::adopt(tree, layouts);
    }

    /// The modern style's panel, if this is the modern style.
    #[cfg(test)]
    pub(crate) fn modern(&self) -> Option<&crate::modern_vitals::ModernVitals> {
        self.modern.as_ref()
    }

    /// The modern panel's layout written into the HUD's layout settings; whether it changed.
    pub fn export_layout(&self, layout: &mut crate::hud_layout::HudLayout) -> bool {
        self.modern.as_ref().is_some_and(|m| {
            let value = match m.current_layout() {
                crate::modern_vitals::VitalsLayout::SideBySide => "side",
                crate::modern_vitals::VitalsLayout::Stacked => "stacked",
            };
            layout.set("vitals.layout", value)
        })
    }

    /// The modern panel's layout as the HUD's layout settings keep it.
    pub fn apply_layout(&self, tree: &mut UiTree, layout: &crate::hud_layout::HudLayout) {
        if let Some(m) = &self.modern {
            let chosen = match layout.get("vitals.layout") {
                Some("stacked") => crate::modern_vitals::VitalsLayout::Stacked,
                _ => crate::modern_vitals::VitalsLayout::SideBySide,
            };
            if m.current_layout() != chosen {
                m.set_layout(tree, chosen);
            }
        }
    }

    /// The modern panel's switch follows the pointer (every frame; nothing in classic).
    pub fn modern_tick(&self, tree: &mut UiTree) {
        if let Some(modern) = &self.modern {
            let values = |d: &Option<VitalDisplay>| d.as_ref().map_or((0, 0), |d| d.values());
            let pk = |d: &Option<VitalDisplay>| values(d).0 > 0;
            let shown = crate::modern_vitals::VitalsShown {
                health: values(&self.health),
                vigor: values(&self.vigor),
                focus: values(&self.focus),
                hero: self.hero,
                pk: pk(&self.pk_damage) || pk(&self.pk_vigor_loss),
            };
            modern.tick(tree, &shown);
        }
    }

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
            modern: None,
        }
    }

    /// The tooltips `gmVitalsUI::gmVitalsUI` gives the health and vigor fields (numbers shown).
    pub fn init_tooltips(&self, tree: &mut UiTree, strings: &dyn FellowStrings) {
        tooltip(tree, strings, true, self.health_field, "ID_Vital_TT_Health");
        tooltip(tree, strings, true, self.vigor_field, "ID_Vital_TT_Vigor");
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
        strings: &dyn FellowStrings,
    ) {
        self.hero = hero;
        let Some(focus) = &mut self.focus else {
            return;
        };
        focus.set_meter_state(tree, id(if hero { "hero" } else { "not_hero" }));
        if hero {
            focus.set_text_state(tree, true);
            focus.update(tree, cur, max, text);
            tooltip(tree, strings, true, self.focus_field, "ID_Vital_TT_Focus");
        } else {
            focus.set_text_state(tree, false);
            focus.update(tree, 1, 1, text);
            if let Some(field) = self.focus_field {
                tree.set_tooltip(field, None);
            }
        }
    }

    /// The vitals the player's script raised before the bar was listening (the latest of each,
    /// hero status first).
    pub fn apply_snapshot(
        &mut self,
        tree: &mut UiTree,
        vitals: &ac2_weenie::PlayerVitals,
        text: VitalText<'_>,
        strings: &dyn FellowStrings,
    ) {
        if let Some(focus) = vitals.focus.filter(|_| vitals.hero) {
            self.update_hero_status(tree, true, focus.current, focus.max, text, strings);
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

    /// `gmVitalsUI::ListenToMessage`: a left click on a field shows or hides its numbers (focus
    /// only for a hero).
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage, strings: &dyn FellowStrings) {
        if self
            .modern
            .as_ref()
            .is_some_and(|m| m.message(tree, message))
        {
            return;
        }
        if message.message != MSG_MOUSE_PRESS_TOP || message.data != LEFT_BUTTON {
            return;
        }
        let pressed = message.element;
        if pressed.is_some() && pressed == self.health_field {
            if let Some(health) = &self.health {
                let visible = health.toggle_text_state(tree);
                tooltip(
                    tree,
                    strings,
                    visible,
                    self.health_field,
                    "ID_Vital_TT_Health",
                );
            }
        }
        if pressed.is_some() && pressed == self.vigor_field {
            if let Some(vigor) = &self.vigor {
                let visible = vigor.toggle_text_state(tree);
                tooltip(
                    tree,
                    strings,
                    visible,
                    self.vigor_field,
                    "ID_Vital_TT_Vigor",
                );
            }
        }
        if pressed.is_some() && pressed == self.focus_field {
            if let Some(focus) = &self.focus {
                if self.hero {
                    let visible = focus.toggle_text_state(tree);
                    tooltip(
                        tree,
                        strings,
                        visible,
                        self.focus_field,
                        "ID_Vital_TT_Focus",
                    );
                } else {
                    focus.set_text_state(tree, false);
                    if let Some(field) = self.focus_field {
                        tree.set_tooltip(field, None);
                    }
                }
            }
        }
    }
}
