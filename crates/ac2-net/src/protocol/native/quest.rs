//! `gmQuestInfo` (0xA0) and `gmQuestInfoList` (0xA1): a quest as the quest journal shows it,
//! and the list the player's script keeps of them.

use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// `gmQuestInfo`; `StreamPack` (Legions 0x005CBC10) packs every field but `_bMaxedOut`.
#[derive(Debug, Clone, PartialEq)]
pub struct GmQuestInfo {
    /// `_questID`.
    pub id: u32,
    /// `_strQuestName`.
    pub name: StringInfo,
    /// `_strQuestDescription`.
    pub description: StringInfo,
    /// `_didIcon`.
    pub icon: u32,
    /// `_questStatus` (0x10000000: done).
    pub status: u32,
    /// `_uiCurPhase`.
    pub phase: u32,
    /// `_strCurJournalEntry`.
    pub journal: StringInfo,
    /// `_iChallengeLevel`.
    pub challenge_level: i32,
    /// `_ttBestowal` (-1: never).
    pub bestowal: f64,
    /// `_ttDone` (-1: not done).
    pub done: f64,
    /// `_bExpires`.
    pub expires: bool,
    /// `_bMaxedOut` (not packed).
    pub maxed_out: bool,
    /// `_ttSecondsRemaining`.
    pub seconds_remaining: f64,
    /// `_ttSecondsUntilRetry`.
    pub seconds_until_retry: f64,
    /// `_playFXOnUpdate`.
    pub play_fx_on_update: bool,
}

impl Default for GmQuestInfo {
    /// `gmQuestInfo::gmQuestInfo` / `Clear`.
    fn default() -> Self {
        Self {
            id: 0,
            name: StringInfo::default(),
            description: StringInfo::default(),
            icon: 0,
            status: 0,
            phase: 0,
            journal: StringInfo::default(),
            challenge_level: 0,
            bestowal: -1.0,
            done: -1.0,
            expires: false,
            maxed_out: false,
            seconds_remaining: 0.0,
            seconds_until_retry: 0.0,
            play_fx_on_update: true,
        }
    }
}

impl GmQuestInfo {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            id: r.u32()?,
            name: StringInfo::read(r)?,
            description: StringInfo::read(r)?,
            icon: r.u32()?,
            status: r.u32()?,
            phase: r.u32()?,
            journal: StringInfo::read(r)?,
            challenge_level: r.u32()? as i32,
            bestowal: r.f64()?,
            done: r.f64()?,
            expires: r.u32()? != 0,
            maxed_out: false,
            seconds_remaining: r.f64()?,
            seconds_until_retry: r.f64()?,
            play_fx_on_update: r.u32()? != 0,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.id);
        self.name.write(w);
        self.description.write(w);
        w.u32(self.icon).u32(self.status).u32(self.phase);
        self.journal.write(w);
        w.u32(self.challenge_level as u32)
            .f64(self.bestowal)
            .f64(self.done)
            .u32(u32::from(self.expires))
            .f64(self.seconds_remaining)
            .f64(self.seconds_until_retry)
            .u32(u32::from(self.play_fx_on_update));
    }

    /// `PList<gmQuestInfo>::UnPack`: the count, then each quest.
    pub(super) fn read_list(r: &mut Reader<'_>) -> Result<Vec<Self>, ProtocolError> {
        let n = r.u32()? as usize;
        if n > MAX_ITEMS || n.saturating_mul(60) > r.remaining() {
            return Err(ProtocolError::Invalid("count"));
        }
        (0..n).map(|_| Self::read(r)).collect()
    }

    pub(super) fn write_list(list: &[Self], w: &mut Writer) {
        w.u32(list.len() as u32);
        for quest in list {
            quest.write(w);
        }
    }
}
