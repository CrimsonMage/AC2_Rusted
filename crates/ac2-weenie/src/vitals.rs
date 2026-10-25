//! The player's vitals as the status bar shows them (`gmVitalsUI`): the `CPlayer` script
//! functions that read the player's qualities and raise `CM_Character::SendNotice_UpdateVital`
//! and `SendNotice_UpdateHeroStatus` (Legions WLib 0x56000005).
//!
//! The maxima follow `VitalSystem` 0x1, 0x4 and 0xC. Each adds attribute- and perk-skill terms
//! (`SkillSystem` 0x29 / 0x35, from the player's `SkillRepository`) and effect modifiers
//! (`EffectSystem` 0x20, from its `EffectRegistry`) to a base quality; those script objects
//! belong to the VM, and with no skill or effect found the script adds nothing, which is what
//! this stand-in does. Experience (`CPlayer` 0x20) needs the `CharacterLevelTable` singleton
//! (package 0x380) and is not raised.

use crate::player_script::GameplayNotice;
use crate::qualities::WeenieQualities;

/// `VitalType` ids (enum mapper 0x23000005).
pub mod vital {
    pub const HEALTH: u32 = 0x4000_0001;
    pub const VIGOR: u32 = 0x4000_0002;
    pub const EXPERIENCE: u32 = 0x4000_0003;
    pub const FOCUS: u32 = 0x4000_0004;
    pub const PK_DAMAGE: u32 = 0x4000_0005;
    pub const PK_VIGOR_LOSS: u32 = 0x4000_0006;
}

/// Int qualities (`gmIntStats`).
mod int {
    pub const HEALTH_CURRENT: u32 = 0x102;
    pub const VIGOR_CURRENT: u32 = 0x103;
    pub const HEALTH_CACHED_MAX: u32 = 0x107;
    pub const VIGOR_CACHED_MAX: u32 = 0x108;
    pub const FOCUS_CURRENT: u32 = 0x10A;
    pub const FOCUS_MAX: u32 = 0x10B;
    pub const PK_DAMAGE: u32 = 0x10D;
    pub const PK_VIGOR_LOSS: u32 = 0x10E;
}
/// Float quality `CurrentVitae` (a percentage, 100 when unset).
const FLOAT_VITAE: u32 = 0x11E;
/// Bool quality `IsHero`.
const BOOL_IS_HERO: u32 = 0xFA0;

/// One vital as `VitalDisplay::Update` takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Vital {
    pub current: u64,
    pub max: u64,
}

/// The latest value of each vital the player's script raised (`None` until it raised one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerVitals {
    pub health: Option<Vital>,
    pub vigor: Option<Vital>,
    pub experience: Option<Vital>,
    pub focus: Option<Vital>,
    /// Over the health meter: `current` is the PK damage (0 unless the player shows PK damage),
    /// `max` the maximum health.
    pub pk_damage: Option<Vital>,
    /// Over the vigor meter, as `pk_damage`.
    pub pk_vigor_loss: Option<Vital>,
    /// `gmVitalsUI::RecvNotice_UpdateHeroStatus`: the focus meter shows only for a hero.
    pub hero: bool,
}

impl PlayerVitals {
    /// Keeps a notice's value (`gmVitalsUI::RecvNotice_UpdateVital` / `_UpdateHeroStatus`).
    pub fn apply(&mut self, notice: &GameplayNotice) {
        match *notice {
            GameplayNotice::UpdateVital {
                vital: kind,
                current,
                max,
            } => {
                let slot = match kind {
                    vital::HEALTH => &mut self.health,
                    vital::VIGOR => &mut self.vigor,
                    vital::EXPERIENCE => &mut self.experience,
                    vital::FOCUS => &mut self.focus,
                    vital::PK_DAMAGE => &mut self.pk_damage,
                    vital::PK_VIGOR_LOSS => &mut self.pk_vigor_loss,
                    _ => return,
                };
                *slot = Some(Vital { current, max });
            }
            GameplayNotice::UpdateHeroStatus { hero, current, max } => {
                self.hero = hero;
                if hero {
                    self.focus = Some(Vital { current, max });
                }
            }
            _ => {}
        }
    }
}

/// `gCWRepository::GetRawInt` (`CWRepository::GetInt`): the quality, or `default`.
fn raw_int(q: &WeenieQualities, quality: u32, default: i32) -> i32 {
    q.ints.get(&quality).copied().unwrap_or(default)
}

/// `_ftol` of a float the script computed (`Interp` `F2I`).
fn ftol(value: f64) -> i32 {
    if value.is_nan() || !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        i32::MIN
    } else {
        value as i32
    }
}

/// `I2I`: the int widened with its sign, as the `ulong64` the notice takes.
fn widen(value: i32) -> u64 {
    i64::from(value) as u64
}

/// `DeathSystem` 0x8: `CurrentVitae` (default 100) times 0.01, each product stored as a float.
fn vitae(q: &WeenieQualities) -> f32 {
    let raw = q.floats.get(&FLOAT_VITAE).copied().unwrap_or(100.0);
    (f64::from(raw) * f64::from(0.01f32)) as f32
}

/// `VitalSystem` 0x1 / 0x4: the cached maximum (`gmEntity` 0x100, through the effect
/// modifiers), the attribute and perk-skill terms, scaled by vitae; at least 1 (`intMax`).
fn scaled_max(q: &WeenieQualities, cached_max: u32) -> i32 {
    let sum = raw_int(q, cached_max, 0);
    let scaled = (f64::from(sum as f32) * f64::from(vitae(q))) as f32;
    ftol(f64::from(scaled)).max(1)
}

/// `VitalSystem` 0x1 (`Agent` 0x1A).
pub fn max_health(q: &WeenieQualities) -> i32 {
    scaled_max(q, int::HEALTH_CACHED_MAX)
}

/// `VitalSystem` 0x4 (`Agent` 0x40).
pub fn max_vigor(q: &WeenieQualities) -> i32 {
    scaled_max(q, int::VIGOR_CACHED_MAX)
}

/// `VitalSystem` 0xC (`Agent` 0xAA): 100 plus `Focus_Max` and the skill terms, no vitae.
pub fn max_focus(q: &WeenieQualities) -> i32 {
    100i32.wrapping_add(raw_int(q, int::FOCUS_MAX, 0))
}

fn update(vital: u32, current: i32, max: i32) -> GameplayNotice {
    GameplayNotice::UpdateVital {
        vital,
        current: widen(current),
        max: widen(max),
    }
}

/// `CPlayer` 0x23: health, then the PK damage over it (0 unless `show_pk_damage`,
/// `CPlayer` 0x1A8 = `GameplayOptionsProfile::GetShowPKDamage`).
pub fn health(q: &WeenieQualities, show_pk_damage: bool, out: &mut Vec<GameplayNotice>) {
    let max = max_health(q);
    out.push(update(
        vital::HEALTH,
        raw_int(q, int::HEALTH_CURRENT, 0),
        max,
    ));
    let pk = if show_pk_damage {
        raw_int(q, int::PK_DAMAGE, 0)
    } else {
        0
    };
    out.push(update(vital::PK_DAMAGE, pk, max));
}

/// `CPlayer` 0x49: vigor, then the PK vigor loss over it.
pub fn vigor(q: &WeenieQualities, show_pk_damage: bool, out: &mut Vec<GameplayNotice>) {
    let max = max_vigor(q);
    out.push(update(vital::VIGOR, raw_int(q, int::VIGOR_CURRENT, 0), max));
    let pk = if show_pk_damage {
        raw_int(q, int::PK_VIGOR_LOSS, 0)
    } else {
        0
    };
    out.push(update(vital::PK_VIGOR_LOSS, pk, max));
}

/// `CPlayer` 0x198: focus.
pub fn focus(q: &WeenieQualities, out: &mut Vec<GameplayNotice>) {
    out.push(update(
        vital::FOCUS,
        raw_int(q, int::FOCUS_CURRENT, 0),
        max_focus(q),
    ));
}

/// `CPlayer` 0x65's hero status (also `CPlayer` 0x85 on bool `IsHero`): the flag, the current
/// focus, the maximum focus.
pub fn hero_status(q: &WeenieQualities) -> GameplayNotice {
    GameplayNotice::UpdateHeroStatus {
        hero: q.bools.get(&BOOL_IS_HERO).copied().unwrap_or(0) != 0,
        current: widen(raw_int(q, int::FOCUS_CURRENT, 0)),
        max: widen(max_focus(q)),
    }
}

/// `CPlayer` 0x65 (`WM_Player` 0x27, the login): the hero status, health, vigor and focus
/// (then experience, not raised here).
pub fn login(q: &WeenieQualities, show_pk_damage: bool) -> Vec<GameplayNotice> {
    let mut out = vec![hero_status(q)];
    health(q, show_pk_damage, &mut out);
    vigor(q, show_pk_damage, &mut out);
    focus(q, &mut out);
    out
}

/// `CPlayer` 0x85, the player's quality-change handler, for the vitals: which `CPlayer`
/// function a changed quality runs (`table` is the `CWRepository` table code: 1 bool, 3 float,
/// 4 int).
pub fn quality_changed(
    q: &WeenieQualities,
    table: u32,
    quality: u32,
    show_pk_damage: bool,
) -> Vec<GameplayNotice> {
    let mut out = Vec::new();
    match (table, quality) {
        // Float `CurrentVitae`: `CPlayer` 0x127 (the vitae notice, not mirrored), 0x23, 0x49.
        (3, FLOAT_VITAE) => {
            health(q, show_pk_damage, &mut out);
            vigor(q, show_pk_damage, &mut out);
        }
        (4, 0x102 | 0x10D | 0x107) => health(q, show_pk_damage, &mut out),
        (4, 0x103 | 0x10E | 0x108) => vigor(q, show_pk_damage, &mut out),
        (4, 0x10A | 0x10B) => focus(q, &mut out),
        (1, BOOL_IS_HERO) => out.push(hero_status(q)),
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qualities(ints: &[(u32, i32)]) -> WeenieQualities {
        let mut q = WeenieQualities::default();
        q.ints.extend(ints.iter().copied());
        q
    }

    fn vital_update(vital: u32, current: i64, max: i64) -> GameplayNotice {
        GameplayNotice::UpdateVital {
            vital,
            current: current as u64,
            max: max as u64,
        }
    }

    #[test]
    fn maxima_scale_by_vitae_and_stay_positive() {
        let mut q = qualities(&[(0x107, 250), (0x108, 101), (0x10B, 20)]);
        assert_eq!(
            (max_health(&q), max_vigor(&q), max_focus(&q)),
            (250, 101, 120)
        );
        // 95% vitae: 0.95 as a float times the float maximum, truncated.
        q.floats.insert(0x11E, 95.0);
        assert_eq!((max_health(&q), max_vigor(&q)), (237, 95));
        // No cached maximum: `intMax(1, 0)`.
        assert_eq!(max_health(&WeenieQualities::default()), 1);
        assert_eq!(max_focus(&WeenieQualities::default()), 100);
    }

    #[test]
    fn login_raises_hero_status_then_health_vigor_and_focus() {
        let mut q = qualities(&[
            (0x102, 80),
            (0x107, 100),
            (0x103, -2),
            (0x108, 60),
            (0x10A, 7),
            (0x10D, 5),
        ]);
        q.bools.insert(0xFA0, 1);
        assert_eq!(
            login(&q, false),
            vec![
                GameplayNotice::UpdateHeroStatus {
                    hero: true,
                    current: 7,
                    max: 100
                },
                vital_update(vital::HEALTH, 80, 100),
                vital_update(vital::PK_DAMAGE, 0, 100),
                vital_update(vital::VIGOR, -2, 60),
                vital_update(vital::PK_VIGOR_LOSS, 0, 60),
                vital_update(vital::FOCUS, 7, 100),
            ]
        );
        // Shown PK damage comes from its quality.
        assert_eq!(login(&q, true)[2], vital_update(vital::PK_DAMAGE, 5, 100));
    }

    #[test]
    fn quality_changes_raise_the_vital_they_feed() {
        let q = qualities(&[(0x102, 3), (0x107, 9), (0x103, 4), (0x108, 8)]);
        let health = vec![
            vital_update(vital::HEALTH, 3, 9),
            vital_update(vital::PK_DAMAGE, 0, 9),
        ];
        assert_eq!(quality_changed(&q, 4, 0x102, false), health);
        assert_eq!(quality_changed(&q, 4, 0x107, false), health);
        assert_eq!(quality_changed(&q, 4, 0x108, false).len(), 2);
        assert_eq!(quality_changed(&q, 3, 0x11E, false).len(), 4);
        assert_eq!(
            quality_changed(&q, 4, 0x10B, false),
            vec![vital_update(vital::FOCUS, 0, 100)]
        );
        // The same id in another table, or another quality, raises nothing.
        assert!(quality_changed(&q, 3, 0x102, false).is_empty());
        assert!(quality_changed(&q, 4, 0x19, false).is_empty());
    }

    #[test]
    fn vitals_keep_the_latest_of_each() {
        let mut vitals = PlayerVitals::default();
        vitals.apply(&vital_update(vital::HEALTH, 3, 9));
        vitals.apply(&vital_update(vital::HEALTH, 4, 9));
        vitals.apply(&GameplayNotice::UpdateHeroStatus {
            hero: true,
            current: 1,
            max: 2,
        });
        assert_eq!(vitals.health, Some(Vital { current: 4, max: 9 }));
        assert_eq!(vitals.focus, Some(Vital { current: 1, max: 2 }));
        assert!(vitals.hero && vitals.vigor.is_none());
    }
}
