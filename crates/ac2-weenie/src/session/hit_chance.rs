//! The hit chance a creature's examination carries (a modern addition, AGENTS.md: hit chance on
//! name plates). Retail's servers ended a creature's `ExaminationProfile` (`WM_Examination` 0x9)
//! with the row "Chance to hit this target: " (`examination.stt` 0x9E78D75C) and the value
//! (0xD81912CB, "%", its number the int variable 0xF6620CE8): every one of the 28,062 creature
//! profiles in the 2017 captures has it, last. The session keeps the latest value per object from
//! every profile it receives, and can ask for a creature's profile on its own: the request the
//! examination panel sends for an object (`CExaminationSystem` 0xE, `WM_Examination` 0x7 with
//! `ExaminationRequest::object`), without touching the player's examination request, selection or
//! panel. Its answer is kept and not shown; a cleared request follows it, as the panel's close
//! sends (`CExaminationSystem` 0xC), so the server is left as it was.

use std::collections::HashMap;

use ac2_net::protocol::{
    ExaminationProfile, ExaminationRequest, ExaminationValue, ScriptRequest, StringVariable,
};

use super::{OnlineSession, SessionEvent};

/// `examination.stt`.
const EXAMINATION_TABLE: u32 = 0x2500_0011;
/// "Chance to hit this target: ".
const HIT_CHANCE_LABEL: u32 = 0x9E78_D75C;
/// The value string's variable (an int, the percentage).
const HIT_CHANCE_VARIABLE: u32 = 0xF662_0CE8;
/// "Killed by " (`examination.stt` 0x32A6B06F): a corpse's row naming its killer, the
/// StringInfo variable 0x23A664C8 (every one of the 5,302 corpse profiles in the 2017 captures has
/// it, after the corpse's name and a blank row).
const KILLED_BY: u32 = 0x32A6_B06F;
const KILLER_VARIABLE: u32 = 0x23A6_64C8;
/// How long a request of the session's own waits for its answer before another may go.
const REQUEST_TIMEOUT: f64 = 5.0;

/// The hit chance a profile carries: the int variable of the text after the label.
pub fn hit_chance_of(profile: &ExaminationProfile) -> Option<u32> {
    let texts: Vec<_> = profile
        .nodes
        .iter()
        .filter_map(|node| match &node.value {
            ExaminationValue::Text(info) => Some(info),
            _ => None,
        })
        .collect();
    let at = texts
        .iter()
        .position(|t| t.string_id == HIT_CHANCE_LABEL && t.table_id == EXAMINATION_TABLE)?;
    texts
        .get(at + 1)?
        .variables
        .iter()
        .find_map(|(id, value)| match value {
            StringVariable::Word { value, .. } if *id == HIT_CHANCE_VARIABLE => Some(*value),
            _ => None,
        })
}

/// Who killed a corpse, as its profile names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Killer {
    /// A character, by its name without the grammar tag the servers append (" [M]", " [Fv]").
    Named(String),
    /// A creature or "misadventure": a localized name, no character.
    Other,
}

/// The killer a corpse's profile names, when it has the "Killed by" row.
pub fn killer_of(profile: &ExaminationProfile) -> Option<Killer> {
    let info = profile.nodes.iter().find_map(|node| match &node.value {
        ExaminationValue::Text(info)
            if info.string_id == KILLED_BY && info.table_id == EXAMINATION_TABLE =>
        {
            Some(info)
        }
        _ => None,
    })?;
    let name = info.variables.iter().find_map(|(id, value)| match value {
        StringVariable::Nested(name) if *id == KILLER_VARIABLE => Some(name),
        _ => None,
    })?;
    Some(match &name.literal {
        Some(literal) => Killer::Named(untagged(literal).to_owned()),
        None => Killer::Other,
    })
}

/// A literal name without its trailing grammar tag: "Sytis [F]" is "Sytis" (the 41 killers of
/// the captured corpse profiles end in " [M]", " [F]" or " [Mv]").
pub fn untagged(name: &str) -> &str {
    match name.rfind(" [") {
        Some(at) if name.ends_with(']') => &name[..at],
        _ => name,
    }
}

/// The latest hit chance of each examined object, and the session's own request in flight.
#[derive(Debug, Default)]
pub(crate) struct HitChances {
    /// Object -> (its hit chance, `None` when its profile had none; when it came).
    known: HashMap<u64, (Option<u32>, f64)>,
    /// Corpse -> its killer, from the latest profile with a "Killed by" row.
    killers: HashMap<u64, Killer>,
    /// The object the session asked for, and when.
    asked: Option<(u64, f64)>,
}

impl OnlineSession {
    /// The latest hit chance of `object`'s examinations, in percent.
    pub fn hit_chance(&self, object: u64) -> Option<u32> {
        self.hit_chances.known.get(&object).and_then(|(v, _)| *v)
    }

    /// Every object's latest known hit chance.
    pub fn hit_chances(&self) -> impl Iterator<Item = (u64, u32)> + '_ {
        self.hit_chances
            .known
            .iter()
            .filter_map(|(id, (v, _))| v.map(|v| (*id, v)))
    }

    /// Whether `object`'s hit chance is unknown (never examined) or its last profile is older
    /// than `max_age` seconds.
    pub fn hit_chance_stale(&self, object: u64, max_age: f64) -> bool {
        self.hit_chances
            .known
            .get(&object)
            .is_none_or(|(_, at)| self.cur_time() - at > max_age)
    }

    /// Who killed the corpse `object`, when a profile of it named them.
    pub fn corpse_killer(&self, object: u64) -> Option<&Killer> {
        self.hit_chances.killers.get(&object)
    }

    /// Asks the server for `object`'s profile for its hit chance: the examination panel's object
    /// request, sent only while the player examines nothing and no such request of the session's
    /// waits. Whether it was sent.
    pub fn examine_for_hit_chance(&mut self, object: u64) -> bool {
        self.examine_quietly(object)
    }

    /// [`Self::examine_for_hit_chance`] for any use: the profile's answer is kept (its hit chance,
    /// its killer) and not shown.
    pub fn examine_quietly(&mut self, object: u64) -> bool {
        let now = self.cur_time();
        if self
            .hit_chances
            .asked
            .is_some_and(|(_, at)| now - at < REQUEST_TIMEOUT)
            || self.script.is_examining()
        {
            return false;
        }
        self.hit_chances.asked = Some((object, now));
        self.send(ScriptRequest::Examine(ExaminationRequest::object(object)).to_event());
        true
    }

    /// A profile arrived: its hit chance is kept for its object. `true` when it answers the
    /// session's own request and the player does not examine that object, so it is not shown
    /// (a cleared request follows it while the player examines nothing).
    pub(super) fn observe_examination(&mut self, profile: &ExaminationProfile) -> bool {
        let request = profile.request;
        if !matches!(
            request.kind,
            ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
        ) {
            return false;
        }
        let now = self.cur_time();
        self.hit_chances
            .known
            .insert(request.iid, (hit_chance_of(profile), now));
        if let Some(killer) = killer_of(profile) {
            self.hit_chances.killers.insert(request.iid, killer);
        }
        if self.hit_chances.asked.map(|(id, _)| id) != Some(request.iid) {
            return false;
        }
        self.hit_chances.asked = None;
        let examining = self.script.is_examining();
        if examining && self.script.examination_request().iid == request.iid {
            return false;
        }
        if !examining {
            self.send(ScriptRequest::Examine(ExaminationRequest::default()).to_event());
        }
        true
    }

    /// The objects that left the world are forgotten.
    pub(super) fn forget_hit_chance(&mut self, object: u64) {
        self.hit_chances.known.remove(&object);
        self.hit_chances.killers.remove(&object);
    }
}

/// The events of a profile the session kept for itself (none: it is not shown).
pub(super) fn kept() -> Vec<SessionEvent> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::{ExaminationDataNode, StringInfo};

    fn text(string_id: u32, variables: Vec<(u32, StringVariable)>) -> ExaminationDataNode {
        ExaminationDataNode {
            order: 2,
            appearance: 0,
            value: ExaminationValue::Text(StringInfo {
                string_id,
                table_id: EXAMINATION_TABLE,
                literal: None,
                variables,
            }),
        }
    }

    #[test]
    fn a_corpse_names_its_killer_without_the_grammar_tag() {
        let named = |literal: Option<&str>| {
            vec![(
                KILLER_VARIABLE,
                StringVariable::Nested(Box::new(StringInfo {
                    literal: literal.map(str::to_owned),
                    string_id: if literal.is_some() { 0 } else { 0x8A15_568C },
                    table_id: if literal.is_some() {
                        0
                    } else {
                        EXAMINATION_TABLE
                    },
                    variables: vec![],
                })),
            )]
        };
        let profile = |variables| ExaminationProfile {
            nodes: vec![text(0x509A_8FF6, vec![]), text(KILLED_BY, variables)],
            ..ExaminationProfile::default()
        };
        assert_eq!(
            killer_of(&profile(named(Some("Alexis The Proud [Mv]")))),
            Some(Killer::Named("Alexis The Proud".into()))
        );
        assert_eq!(killer_of(&profile(named(None))), Some(Killer::Other));
        assert_eq!(killer_of(&ExaminationProfile::default()), None);
        assert_eq!(untagged("Hand Of Doom [F]"), "Hand Of Doom");
        assert_eq!(untagged("Plain"), "Plain");
    }

    #[test]
    fn the_value_is_the_int_after_the_label() {
        let value = vec![(
            HIT_CHANCE_VARIABLE,
            StringVariable::Word { kind: 2, value: 51 },
        )];
        let mut profile = ExaminationProfile {
            nodes: vec![
                text(0x1234, vec![]),
                text(HIT_CHANCE_LABEL, vec![]),
                text(0xD819_12CB, value.clone()),
            ],
            ..ExaminationProfile::default()
        };
        assert_eq!(hit_chance_of(&profile), Some(51));
        // Without the label, the same text is not a hit chance.
        profile.nodes.remove(1);
        assert_eq!(hit_chance_of(&profile), None);
    }
}
