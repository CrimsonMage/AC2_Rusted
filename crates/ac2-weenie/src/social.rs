//! The fellowship and allegiance as the panels read them: plain copies of the native adaptors
//! the scripts fill (`FellowshipAdaptor` / `FellowAdaptor`, `AllegianceProfileAdaptor` /
//! `AllegianceDataAdaptor`) and hand the UI with the `CM_Fellowship` / `CM_Allegiance`
//! notices; [`Social::apply`] keeps the latest of them.

use std::sync::Arc;

use ac2_net::protocol::StringInfo;

use crate::notice::GameplayNotice;

/// The fellowship and allegiance panels' actions (`gmCWInterface::DoFellowship*`,
/// `SwearAllegiance`, `BreakAllegiance`, `AcceptVassal`, `DeclineVassal`,
/// `QueryAllegianceProfile` -> their `CPlayer` IFuns -> the `WM_Fellowship` / `WM_Allegiance`
/// stubs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocialRequest {
    /// `DoFellowshipCreate(name, social, lootingMethod)` (`CPlayer` 0xE3). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipCreate`.
    FellowshipCreate {
        name: String,
        social: u32,
        looting: u32,
    },
    /// `DoFellowshipRecruit(target)` (`CPlayer` 0x9C). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipRecruit`.
    FellowshipRecruit(u64),
    /// `DoFellowshipRecruitAccept` (`CPlayer` 0x14): the recruiter (instance id 400). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipRecruitAccept`.
    FellowshipRecruitAccept,
    /// `DoFellowshipRecruitDecline(context)` (`CPlayer` 0xEB; the panel passes 0x138D). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipRecruitDecline`.
    FellowshipRecruitDecline(u32),
    /// `DoFellowshipLeave` (`CPlayer` 0x1C). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipLeave`.
    FellowshipLeave,
    /// `DoFellowshipDisband` (`CPlayer` 0x86). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipDisband`.
    FellowshipDisband,
    /// `DoFellowshipDismiss(target)` (`CPlayer` 0x26). Its wrapper is `ggmCWInterface::CallIFun_DoFellowshipDismiss`.
    FellowshipDismiss(u64),
    /// `SwearAllegiance` (`CPlayer` 0x7F): to the selection. Its wrapper is `ggmCWInterface::CallIFun_SwearAllegiance`.
    SwearAllegiance,
    /// `BreakAllegiance(target)` (`CPlayer` 0xB). Its wrapper is `ggmCWInterface::CallIFun_BreakAllegiance`.
    BreakAllegiance(u64),
    /// `AcceptVassal` (`CPlayer` 0x12B): the would-be vassal (instance id 450). Its wrapper is `ggmCWInterface::CallIFun_AcceptVassal`.
    AcceptVassal,
    /// `DeclineVassal(context)` (`CPlayer` 0xBF). Its wrapper is `ggmCWInterface::CallIFun_DeclineVassal`.
    DeclineVassal(u32),
    /// `QueryAllegianceProfile(target)` (`CPlayer` 0xEA). Its wrapper is `ggmCWInterface::CallIFun_QueryAllegianceProfile`.
    QueryAllegianceProfile(u64),
}

/// What [`SocialRequest::call`] reads from the player: `Player` 0x77 (instance id quality 400,
/// the fellowship recruiter), `Player` 0x36 (450, the vassal asking), `Player` 0x3D (the
/// selection).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SocialContext {
    pub recruiter: u64,
    pub vassal: u64,
    pub selection: u64,
}

impl SocialRequest {
    /// The `gmCWInterface` IFun the UI calls (`FunctionID` 0xB4xxxx).
    pub fn ifun(&self) -> u32 {
        0x00B4_0000
            | match self {
                Self::FellowshipCreate { .. } => 0xA0,
                Self::FellowshipDisband => 0xA1,
                Self::FellowshipLeave => 0xA2,
                Self::FellowshipRecruit(_) => 0xA3,
                Self::FellowshipDismiss(_) => 0xA4,
                Self::FellowshipRecruitAccept => 0xA5,
                Self::FellowshipRecruitDecline(_) => 0xA6,
                Self::QueryAllegianceProfile(_) => 0xB3,
                Self::SwearAllegiance => 0xB4,
                Self::BreakAllegiance(_) => 0xB5,
                Self::AcceptVassal => 0xB6,
                Self::DeclineVassal(_) => 0xB7,
            }
    }

    /// The stub's call as the `CPlayer` IFun makes it: `WM_Fellowship` 0x22 (looting, social,
    /// the name as a `wpstring`), 0x2A, 0x25, 0x2F (context, recruiter), 0x29, 0x2C, 0x23;
    /// `WM_Allegiance` 0x39, 0x32, 0x31, 0x3B (context, vassal), 0x36.
    pub fn call(&self, who: SocialContext) -> ac2_net::protocol::ScriptCall {
        use ac2_net::protocol::{NativeObject, PackValue::*};
        let (function, args) = match self {
            Self::FellowshipCreate {
                name,
                social,
                looting,
            } => (
                0x820C_001B,
                vec![
                    Int(*looting),
                    Int(*social),
                    ac2_net::protocol::PackValue::native(NativeObject::WString(name.clone())),
                ],
            ),
            Self::FellowshipRecruit(target) => (0x820C_0015, vec![LongInt(*target)]),
            Self::FellowshipRecruitAccept => (0x820C_000D, vec![LongInt(who.recruiter)]),
            Self::FellowshipRecruitDecline(context) => {
                (0x820C_001A, vec![Int(*context), LongInt(who.recruiter)])
            }
            Self::FellowshipLeave => (0x820C_001C, Vec::new()),
            Self::FellowshipDisband => (0x820C_0003, Vec::new()),
            Self::FellowshipDismiss(target) => (0x820C_000E, vec![LongInt(*target)]),
            Self::SwearAllegiance => (0x8203_000A, vec![LongInt(who.selection)]),
            Self::BreakAllegiance(target) => (0x8203_0009, vec![LongInt(*target)]),
            Self::AcceptVassal => (0x8203_001C, vec![LongInt(who.vassal)]),
            Self::DeclineVassal(context) => (0x8203_0029, vec![Int(*context), LongInt(who.vassal)]),
            Self::QueryAllegianceProfile(target) => (0x8203_0005, vec![LongInt(*target)]),
        };
        ac2_net::protocol::ScriptCall::new(function, args).expect("the stubs' layouts")
    }
}

/// `FellowAdaptor`: one member.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FellowInfo {
    pub name: StringInfo,
    pub level: u32,
    pub health: u32,
    pub max_health: u32,
    pub vigor: u32,
    pub max_vigor: u32,
    pub pk_damage: u32,
    pub pk_vigor_loss: u32,
    pub cell: u32,
}

/// `FellowshipAdaptor`: the fellowship (`m_table`: members by id, in the order added).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FellowshipInfo {
    pub name: String,
    pub leader: u64,
    pub social: bool,
    pub looting_method: i32,
    pub fellows: Vec<(u64, FellowInfo)>,
}

/// `AllegianceDataAdaptor`: one member of an allegiance.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AllegianceMember {
    pub id: u64,
    pub name: StringInfo,
    pub title: StringInfo,
    pub xp_passed: u64,
    pub species: u32,
    pub level: u32,
    pub gender: u32,
    pub officer: bool,
    pub faction: u32,
    pub rank: u32,
}

/// `AllegianceProfileAdaptor`: the player's place in its allegiance.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AllegianceProfile {
    pub monarch: AllegianceMember,
    pub patron: AllegianceMember,
    pub member: AllegianceMember,
    pub monarch_online: bool,
    pub patron_online: bool,
    pub member_online: bool,
    /// The vassals and whether each is online.
    pub vassals: Vec<(AllegianceMember, bool)>,
    pub total: u32,
    pub motd: StringInfo,
    pub name: StringInfo,
    pub faction: u32,
    pub officers: Vec<u64>,
}

/// The latest fellowship and allegiance the notices carried.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Social {
    /// Shared with the client frame: a change copies the record (`Arc::make_mut`) only while a
    /// frame still holds the old one.
    pub fellowship: Option<Arc<FellowshipInfo>>,
    pub allegiance: Option<Arc<AllegianceProfile>>,
}

impl Social {
    fn fellow(&mut self, id: u64) -> Option<&mut FellowInfo> {
        Arc::make_mut(self.fellowship.as_mut()?)
            .fellows
            .iter_mut()
            .find(|(f, _)| *f == id)
            .map(|(_, info)| info)
    }

    /// Keeps a `CM_Fellowship` / `CM_Allegiance` notice's state (`gmFellowshipUI`,
    /// `gmAllegianceUI` keep the same from their handlers).
    pub fn apply(&mut self, notice: &GameplayNotice) {
        use GameplayNotice as N;
        match notice {
            N::UpdateFellowship(info) => self.fellowship = Some(Arc::new(info.clone())),
            N::ClearFellowship => self.fellowship = None,
            N::UpdateFellow { fellow, info } => {
                if let Some(f) = self.fellow(*fellow) {
                    *f = info.clone();
                } else if let Some(fs) = self.fellowship.as_mut() {
                    Arc::make_mut(fs).fellows.push((*fellow, info.clone()));
                }
            }
            N::ClearFellow(id) => {
                if let Some(fs) = self.fellowship.as_mut() {
                    Arc::make_mut(fs).fellows.retain(|(f, _)| f != id);
                }
            }
            N::UpdateLeader(id) => {
                if let Some(fs) = self.fellowship.as_mut() {
                    Arc::make_mut(fs).leader = *id;
                }
            }
            N::UpdateFellowHealth { fellow, value, pk } => {
                if let Some(f) = self.fellow(*fellow) {
                    (f.health, f.pk_damage) = (*value, *pk);
                }
            }
            N::UpdateFellowVigor { fellow, value, pk } => {
                if let Some(f) = self.fellow(*fellow) {
                    (f.vigor, f.pk_vigor_loss) = (*value, *pk);
                }
            }
            N::UpdateFellowMaxHealth { fellow, value } => {
                if let Some(f) = self.fellow(*fellow) {
                    f.max_health = *value;
                }
            }
            N::UpdateFellowMaxVigor { fellow, value } => {
                if let Some(f) = self.fellow(*fellow) {
                    f.max_vigor = *value;
                }
            }
            N::UpdateFellowLevel { fellow, level } => {
                if let Some(f) = self.fellow(*fellow) {
                    f.level = *level;
                }
            }
            N::UpdateFellowCell { fellow, cell } => {
                if let Some(f) = self.fellow(*fellow) {
                    f.cell = *cell;
                }
            }
            N::UpdateAllegianceProfile(profile) => {
                self.allegiance = Some(Arc::new((**profile).clone()))
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn social_requests_pack_as_their_stubs() {
        let who = SocialContext {
            recruiter: 0x5000_0000_0000_0042,
            vassal: 0x5000_0000_0000_0043,
            selection: 0x5000_0000_0000_0044,
        };
        let bytes = |r: SocialRequest| r.call(who).to_event().encode().1;
        // `WM_Fellowship` 0x2F: the context, then the recruiter (int tag 1, 64-bit tag 2).
        assert_eq!(
            bytes(SocialRequest::FellowshipRecruitDecline(0x138D)),
            words(&[
                0x0001_0053,
                0x820C_001A,
                20,
                1,
                0x138D,
                2,
                0x42,
                0x5000_0000
            ])
        );
        assert_eq!(
            bytes(SocialRequest::FellowshipRecruitAccept),
            words(&[0x0001_0053, 0x820C_000D, 12, 2, 0x42, 0x5000_0000])
        );
        // `WM_Allegiance` 0x39 swears to the selection; 0x3B declines the asking vassal.
        assert_eq!(
            bytes(SocialRequest::SwearAllegiance),
            words(&[0x0001_0053, 0x8203_000A, 12, 2, 0x44, 0x5000_0000])
        );
        assert_eq!(
            bytes(SocialRequest::DeclineVassal(7)),
            words(&[0x0001_0053, 0x8203_0029, 20, 1, 7, 2, 0x43, 0x5000_0000])
        );
        assert_eq!(
            bytes(SocialRequest::FellowshipDisband),
            words(&[0x0001_0053, 0x820C_0003, 0])
        );
        assert_eq!(SocialRequest::AcceptVassal.ifun(), 0x00B4_00B6);
    }

    #[test]
    fn fellowship_notices_keep_the_latest_state() {
        let mut social = Social::default();
        social.apply(&GameplayNotice::UpdateFellowship(FellowshipInfo {
            name: "Wanderers".into(),
            leader: 1,
            fellows: vec![(1, FellowInfo::default())],
            ..FellowshipInfo::default()
        }));
        social.apply(&GameplayNotice::UpdateFellow {
            fellow: 2,
            info: FellowInfo::default(),
        });
        social.apply(&GameplayNotice::UpdateFellowHealth {
            fellow: 2,
            value: 40,
            pk: 3,
        });
        social.apply(&GameplayNotice::UpdateLeader(2));
        social.apply(&GameplayNotice::ClearFellow(1));
        let fs = social.fellowship.as_ref().unwrap();
        assert_eq!((fs.leader, fs.fellows.len()), (2, 1));
        assert_eq!((fs.fellows[0].1.health, fs.fellows[0].1.pk_damage), (40, 3));
        social.apply(&GameplayNotice::ClearFellowship);
        assert!(social.fellowship.is_none());
    }
}
