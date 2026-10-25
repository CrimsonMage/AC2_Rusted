//! The fellowship and allegiance as the panels read them: plain copies of the native adaptors
//! the scripts fill (`FellowshipAdaptor` / `FellowAdaptor`, `AllegianceProfileAdaptor` /
//! `AllegianceDataAdaptor`) and hand the UI with the `CM_Fellowship` / `CM_Allegiance`
//! notices; [`Social::apply`] keeps the latest of them.

use ac2_net::protocol::StringInfo;

use crate::notice::GameplayNotice;

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
    pub fellowship: Option<FellowshipInfo>,
    pub allegiance: Option<AllegianceProfile>,
}

impl Social {
    fn fellow(&mut self, id: u64) -> Option<&mut FellowInfo> {
        self.fellowship
            .as_mut()?
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
            N::UpdateFellowship(info) => self.fellowship = Some(info.clone()),
            N::ClearFellowship => self.fellowship = None,
            N::UpdateFellow { fellow, info } => {
                if let Some(f) = self.fellow(*fellow) {
                    *f = info.clone();
                } else if let Some(fs) = self.fellowship.as_mut() {
                    fs.fellows.push((*fellow, info.clone()));
                }
            }
            N::ClearFellow(id) => {
                if let Some(fs) = self.fellowship.as_mut() {
                    fs.fellows.retain(|(f, _)| f != id);
                }
            }
            N::UpdateLeader(id) => {
                if let Some(fs) = self.fellowship.as_mut() {
                    fs.leader = *id;
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
            N::UpdateAllegianceProfile(profile) => self.allegiance = Some((**profile).clone()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
