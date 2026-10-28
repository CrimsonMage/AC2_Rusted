//! `gmQuestInfo` (`ac2/shared/src/quest/gmquestinfo.cpp.c`): the rules a quest's status, time and
//! extra text follow. The quest itself is [`GmQuestInfo`] off the wire (`ac2-net`).

use ac2_net::protocol::{GmQuestInfo, StringInfo};

/// `gmQuestInfo::_questStatus`: underway (`GetStatus` 0x10000000, the status the time left and
/// the extra text read), failed (0x20000000) and completed (0x30000000).
pub const STATUS_UNDERWAY: u32 = 0x1000_0000;
pub const STATUS_FAILED: u32 = 0x2000_0000;
pub const STATUS_COMPLETED: u32 = 0x3000_0000;

/// `gmQuestInfo::MaxedOut`: a quest that is not underway and was marked maxed out.
pub fn maxed_out(quest: &GmQuestInfo) -> bool {
    quest.status != STATUS_UNDERWAY && quest.maxed_out
}

/// `gmQuestInfo::GetSecondsRemaining`: the time left (`_ttSecondsRemaining`) while underway, else
/// the time until a retry (`_ttSecondsUntilRetry`), truncated toward zero as `_ftol2` does.
pub fn seconds_remaining(quest: &GmQuestInfo) -> i64 {
    let seconds = if quest.status == STATUS_UNDERWAY {
        quest.seconds_remaining
    } else {
        quest.seconds_until_retry
    };
    seconds as i64
}

/// `gmQuestInfo::PlaysFXOnUpdate` (the script efun 0x00A00007 reads it through `ac2-script`).
pub fn plays_fx_on_update(quest: &GmQuestInfo) -> bool {
    quest.play_fx_on_update
}

/// The challenge level the extra text shows (`_iChallengeLevel`; -999 is "any").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeLevel {
    /// `ID_Quest_ChallengeLevelAny` (`AddVariable_StringID`).
    Any,
    /// The level as a number (`AddVariable_Int`).
    Level(i32),
}

/// What `gmQuestInfo::InqExtraText` builds: the table enum 0x41000018 with the words the quest
/// journal's extra text shows, by status.
#[derive(Debug, Clone, PartialEq)]
pub enum QuestExtraText<'a> {
    /// Underway: `ID_Quest_Description` with `DESC`, `JOURNAL` and `LEVEL`.
    Underway {
        description: &'a StringInfo,
        journal: &'a StringInfo,
        challenge: ChallengeLevel,
    },
    /// Failed: `ID_Quest_Failed`.
    Failed,
    /// Completed: `ID_Quest_Completed`.
    Completed,
}

/// `gmQuestInfo::InqExtraText`: the extra text for a quest's status, or none for any other.
pub fn extra_text(quest: &GmQuestInfo) -> Option<QuestExtraText<'_>> {
    match quest.status {
        STATUS_UNDERWAY => Some(QuestExtraText::Underway {
            description: &quest.description,
            journal: &quest.journal,
            challenge: if quest.challenge_level == -999 {
                ChallengeLevel::Any
            } else {
                ChallengeLevel::Level(quest.challenge_level)
            },
        }),
        STATUS_FAILED => Some(QuestExtraText::Failed),
        STATUS_COMPLETED => Some(QuestExtraText::Completed),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(status: u32) -> GmQuestInfo {
        GmQuestInfo {
            status,
            ..GmQuestInfo::default()
        }
    }

    #[test]
    fn maxed_out_needs_a_quest_not_underway() {
        let mut q = quest(STATUS_UNDERWAY);
        q.maxed_out = true;
        assert!(!maxed_out(&q));
        q.status = STATUS_FAILED;
        assert!(maxed_out(&q));
        q.status = STATUS_COMPLETED;
        assert!(maxed_out(&q));
        q.maxed_out = false;
        assert!(!maxed_out(&q));
    }

    #[test]
    fn seconds_remaining_reads_the_field_for_the_status() {
        let mut q = quest(STATUS_UNDERWAY);
        q.seconds_remaining = 3_700.9;
        q.seconds_until_retry = 42.0;
        assert_eq!(seconds_remaining(&q), 3_700);
        q.status = STATUS_COMPLETED;
        assert_eq!(seconds_remaining(&q), 42);
        q.seconds_until_retry = -5.5;
        assert_eq!(seconds_remaining(&q), -5);
    }

    #[test]
    fn plays_fx_on_update_is_the_flag() {
        let mut q = quest(STATUS_UNDERWAY);
        assert!(plays_fx_on_update(&q));
        q.play_fx_on_update = false;
        assert!(!plays_fx_on_update(&q));
    }

    #[test]
    fn extra_text_by_status() {
        let mut q = quest(STATUS_UNDERWAY);
        q.challenge_level = 7;
        assert!(matches!(
            extra_text(&q),
            Some(QuestExtraText::Underway {
                challenge: ChallengeLevel::Level(7),
                ..
            })
        ));
        q.challenge_level = -999;
        assert!(matches!(
            extra_text(&q),
            Some(QuestExtraText::Underway {
                challenge: ChallengeLevel::Any,
                ..
            })
        ));
        q.status = STATUS_FAILED;
        assert_eq!(extra_text(&q), Some(QuestExtraText::Failed));
        q.status = STATUS_COMPLETED;
        assert_eq!(extra_text(&q), Some(QuestExtraText::Completed));
        q.status = 0;
        assert_eq!(extra_text(&q), None);
    }
}
