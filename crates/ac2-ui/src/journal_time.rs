//! `TimeDisplay` (`ac2/client/src/ui/gameplay/helper/timedisplay.cpp.c`): a quest's time left as
//! the journal shows it (`gmJournalQuestsUI::DoQuestTime`). `Init` sets the text at once and
//! `TimeDisplay::ListenToMessage` counts it down on the global loop, rewriting the text only when
//! the words change and then waiting until the next change is due (`m_nextUpdateTime`). Below
//! one second left the display stops listening.
//!
//! The words: `ComputeTimeLeft` picks months (`ID_Time_Months`, months and days), days
//! (`ID_Time_Days`, days and hours), hours (`ID_Time_Hours`, hours and minutes) or minutes
//! (`ID_Time_Minutes`, minutes and seconds) by the thresholds `m_monthTimeMin`, `m_dayTimeMin`
//! and `m_minuteTimeMin` (the journal sets 1, 1 and 60). The minute and second fields are
//! formatted `%02d` (the format string read from the admin build).

use crate::element::UiTree;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// One month as `ComputeTimeLeft` counts it: 31 days.
pub const MONTH_SECONDS: i64 = 31 * DAY_SECONDS;
const DAY_SECONDS: i64 = 86_400;
const HOUR_SECONDS: i64 = 3_600;
const MINUTE_SECONDS: i64 = 60;

/// What the display shows for `seconds` left, and how long until the words change.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeLeft {
    pub text: UiString,
    /// Seconds until the next change (`ComputeTimeLeft`'s `m_nextUpdateTime` offset).
    pub next_update: f64,
}

/// `ComputeTimeLeft`'s choice of words for `seconds` left (C division and remainder, so a
/// negative count keeps its sign as retail's does).
pub fn time_left(seconds: i64, month_min: i64, day_min: i64, minute_min: i64) -> TimeLeft {
    let months = seconds / MONTH_SECONDS;
    let days = seconds / DAY_SECONDS;
    let minutes = seconds / MINUTE_SECONDS;
    let (string_id, vars, next) = if month_min < months {
        (
            "ID_Time_Months",
            vec![
                ("months", SkillVar::UInt(months as u64)),
                (
                    "days",
                    SkillVar::UInt(((seconds % MONTH_SECONDS) / DAY_SECONDS) as u64),
                ),
            ],
            seconds % DAY_SECONDS,
        )
    } else if day_min < days {
        (
            "ID_Time_Days",
            vec![
                ("days", SkillVar::UInt(days as u64)),
                (
                    "hours",
                    SkillVar::UInt(((seconds % DAY_SECONDS) / HOUR_SECONDS) as u64),
                ),
            ],
            seconds % HOUR_SECONDS,
        )
    } else if minute_min < minutes {
        (
            "ID_Time_Hours",
            vec![
                ("hours", SkillVar::Int(seconds / HOUR_SECONDS)),
                (
                    "minutes",
                    SkillVar::Text(two_digits((seconds % HOUR_SECONDS) / MINUTE_SECONDS)),
                ),
            ],
            seconds % MINUTE_SECONDS,
        )
    } else {
        (
            "ID_Time_Minutes",
            vec![
                ("minutes", SkillVar::Int(minutes)),
                (
                    "seconds",
                    SkillVar::Text(two_digits(seconds % MINUTE_SECONDS)),
                ),
            ],
            1,
        )
    };
    TimeLeft {
        text: UiString::ui(tokenize(string_id), vars_by_id(vars)),
        next_update: next as f64,
    }
}

/// The variable names as the words' tokens (`token_months`, ...).
fn vars_by_id(vars: Vec<(&str, SkillVar)>) -> Vec<(u32, SkillVar)> {
    vars.into_iter()
        .map(|(name, value)| (tokenize(name), value))
        .collect()
}

fn two_digits(value: i64) -> String {
    format!("{value:02}")
}

/// `TimeDisplay`'s state: the text element and the end time (`m_endTime`).
#[derive(Debug, Clone)]
pub struct QuestTimeDisplay {
    /// `m_text`: the row's `QuestTime` element.
    text: usize,
    /// `m_endTime`: the time the quest runs out, on the `now` clock.
    end: f64,
    month_min: i64,
    day_min: i64,
    minute_min: i64,
    /// `m_nextUpdateTime`.
    next_update: f64,
    /// `m_oldText`: the words last shown.
    shown: Option<UiString>,
    /// Whether the global loop still counts it (`UnregisterForGlobalMessage` below one second).
    listening: bool,
}

impl QuestTimeDisplay {
    /// `TimeDisplay::Init`: the display listens and shows the time left at `now`.
    pub fn init(
        tree: &mut UiTree,
        strings: &dyn UiStrings,
        text: usize,
        end: f64,
        thresholds: (i64, i64, i64),
        now: f64,
    ) -> Self {
        let mut display = Self {
            text,
            end,
            month_min: thresholds.0,
            day_min: thresholds.1,
            minute_min: thresholds.2,
            next_update: 0.0,
            shown: None,
            listening: true,
        };
        display.compute(tree, strings, now);
        display
    }

    /// `TimeDisplay::ListenToMessage` on the global loop: the words are computed again when the
    /// next change is due.
    pub fn global_loop(&mut self, tree: &mut UiTree, strings: &dyn UiStrings, now: f64) {
        if self.listening && self.next_update <= now {
            self.compute(tree, strings, now);
        }
    }

    /// `TimeDisplay::ComputeTimeLeft`.
    fn compute(&mut self, tree: &mut UiTree, strings: &dyn UiStrings, now: f64) {
        let seconds = (self.end - now) as i64;
        let left = time_left(seconds, self.month_min, self.day_min, self.minute_min);
        if self.shown.as_ref() != Some(&left.text) {
            tree.set_text(self.text, &strings.render_string(&left.text));
            self.shown = Some(left.text);
        }
        self.next_update = now + left.next_update;
        if seconds < 1 {
            self.listening = false;
        }
    }
}
