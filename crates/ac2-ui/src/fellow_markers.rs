//! `FellowshipHelper`'s map markers (marker type 4): each fellow but the player is a note on the
//! map at its cell, named, with the fellow marker image (`FileArray::GetDIDFromEnum(0x41000010,
//! 0x40000003)`), unless the player's gameplay options hide fellows on the map (profile bit
//! 0x8000, [`crate::PROFILE_HIDE_FELLOWS_ON_MAP`]):
//!
//! - `AddFellow`: a new fellow's note (`SendNotice_AddMarker(4, id, cell, image, name)`).
//! - `RecvNotice_UpdateFellowCellID`: a note at the fellow's new cell.
//! - `RemoveFellow`: its note goes (`SendNotice_RemoveMarker(4, id)`).
//! - `RemoveAllFellows` (a fellowship cleared or replaced): every fellow note goes
//!   (`SendNotice_ClearMarkers(4)`).
//! - `RecvNotice_ReloadGameplayOptions`: shown, a note for each fellow; hidden, every note goes.
//!
//! Retail runs this in both helpers (the social page's and the mini panel's); the second note
//! for a fellow is dropped by the map as a near duplicate and the removals repeat harmlessly, so
//! the HUD sends them once. The client applies the requests to its map notes
//! ([`crate::HudRequest::Marker`]).

use ac2_net::protocol::StringInfo;
use ac2_weenie::GameplayNotice;

use crate::radar_state::{MapMarker, MarkerRequest};

/// `CMapNoteRow` type 4: the fellows.
pub const FELLOW_MARKER: u32 = 4;
/// `FellowshipHelper::_fellowMarkerDID`'s enum and group.
pub const FELLOW_MARKER_ENUM: u32 = 0x4100_0010;
pub const FELLOW_MARKER_GROUP: u32 = 0x4000_0003;

/// The fellows' notes: the marker image, whether the options show fellows on the map (`None`
/// until they are read: `InqGameplayOptions` failing sends nothing), and the requests made.
#[derive(Debug, Clone, Default)]
pub struct FellowMarkers {
    image: u32,
    shown: Option<bool>,
    requests: Vec<MarkerRequest>,
}

impl FellowMarkers {
    pub fn new(image: u32) -> Self {
        Self {
            image,
            ..Self::default()
        }
    }

    /// Whether notes are sent: the options are read and do not hide fellows on the map.
    fn on(&self) -> bool {
        self.shown == Some(true)
    }

    /// `AddFellow` / `RecvNotice_UpdateFellowCellID`: the fellow's note, unless it is the player.
    pub(crate) fn add(&mut self, player: u64, fellow: u64, cell: u32, name: &str) {
        if self.on() && fellow != player {
            self.requests.push(MarkerRequest::Add(MapMarker {
                kind: FELLOW_MARKER,
                id: fellow,
                cell,
                image: self.image,
                message: StringInfo::literal(name),
            }));
        }
    }

    /// `RemoveFellow`: the fellow's note goes, unless it is the player.
    pub(crate) fn remove(&mut self, player: u64, fellow: u64) {
        if self.on() && fellow != player {
            self.requests.push(MarkerRequest::Remove {
                kind: FELLOW_MARKER,
                id: fellow,
            });
        }
    }

    /// `RemoveAllFellows`: every fellow note goes.
    pub(crate) fn clear(&mut self) {
        if self.on() {
            self.requests.push(MarkerRequest::Clear(FELLOW_MARKER));
        }
    }

    /// `RecvNotice_ReloadGameplayOptions` with the options as they now are (`hidden`: the
    /// profile hides fellows on the map; `None` when they could not be read, which changes
    /// nothing): shown, a note for each fellow but the player; hidden, every note goes.
    pub(crate) fn reload(
        &mut self,
        hidden: Option<bool>,
        player: u64,
        fellows: &[(u64, String, u32)],
    ) {
        let Some(hidden) = hidden else {
            return;
        };
        self.shown = Some(!hidden);
        if hidden {
            self.requests.push(MarkerRequest::Clear(FELLOW_MARKER));
            return;
        }
        for (fellow, name, cell) in fellows {
            self.add(player, *fellow, *cell, name);
        }
    }

    /// The notes a fellowship notice changes, with the members the helper listed before and after
    /// it (`AddAllFellows` / `AddFellow` / `RemoveFellow` / `RemoveAllFellows` /
    /// `RecvNotice_UpdateFellowCellID`).
    pub(crate) fn notice(
        &mut self,
        player: u64,
        notice: &GameplayNotice,
        before: &[(u64, String, u32)],
        after: &[(u64, String, u32)],
    ) {
        let listed = |members: &[(u64, String, u32)], id: u64| members.iter().any(|m| m.0 == id);
        match notice {
            GameplayNotice::UpdateFellowship(_) => {
                if !before.is_empty() {
                    self.clear();
                }
                for (fellow, name, cell) in after {
                    self.add(player, *fellow, *cell, name);
                }
            }
            GameplayNotice::ClearFellowship if !before.is_empty() => self.clear(),
            GameplayNotice::UpdateFellow { fellow, .. } if !listed(before, *fellow) => {
                if let Some((_, name, cell)) = after.iter().find(|m| m.0 == *fellow) {
                    self.add(player, *fellow, *cell, name);
                }
            }
            GameplayNotice::ClearFellow(fellow) if listed(before, *fellow) => {
                self.remove(player, *fellow);
            }
            GameplayNotice::UpdateFellowCell { fellow, cell } => {
                if let Some((_, name, _)) = after.iter().find(|m| m.0 == *fellow) {
                    self.add(player, *fellow, *cell, name);
                }
            }
            _ => {}
        }
    }

    /// The requests made since the last call, in order.
    pub fn take_requests(&mut self) -> Vec<MarkerRequest> {
        std::mem::take(&mut self.requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: u64 = 1;
    const IMAGE: u32 = 0x0600_0001;

    fn member(id: u64, name: &str, cell: u32) -> (u64, String, u32) {
        (id, name.to_owned(), cell)
    }

    fn add(id: u64, cell: u32, name: &str) -> MarkerRequest {
        MarkerRequest::Add(MapMarker {
            kind: FELLOW_MARKER,
            id,
            cell,
            image: IMAGE,
            message: StringInfo::literal(name),
        })
    }

    /// Until the options are read nothing is sent; a reload that shows fellows notes each one but
    /// the player, and one that hides them clears the notes.
    #[test]
    fn the_options_reload_shows_or_clears_the_fellows_notes() {
        let mut markers = FellowMarkers::new(IMAGE);
        let members = [member(PLAYER, "Me", 0x10), member(2, "Ann", 0x20)];
        markers.add(PLAYER, 2, 0x20, "Ann");
        markers.reload(None, PLAYER, &members);
        assert!(markers.take_requests().is_empty(), "options not read");
        markers.reload(Some(false), PLAYER, &members);
        assert_eq!(markers.take_requests(), [add(2, 0x20, "Ann")]);
        markers.reload(Some(true), PLAYER, &members);
        assert_eq!(
            markers.take_requests(),
            [MarkerRequest::Clear(FELLOW_MARKER)]
        );
        // Hidden: the notices send nothing.
        markers.notice(
            PLAYER,
            &GameplayNotice::UpdateFellowCell {
                fellow: 2,
                cell: 0x30,
            },
            &members,
            &members,
        );
        assert!(markers.take_requests().is_empty());
    }

    /// `AddFellow` for a new member, `RecvNotice_UpdateFellowCellID` at the new cell,
    /// `RemoveFellow`, and `RemoveAllFellows` for a cleared fellowship.
    #[test]
    fn the_notices_add_move_and_remove_the_fellows_notes() {
        let mut markers = FellowMarkers::new(IMAGE);
        markers.reload(Some(false), PLAYER, &[]);
        let me = member(PLAYER, "Me", 0x10);
        let ann = member(2, "Ann", 0x20);
        let update = GameplayNotice::UpdateFellow {
            fellow: 2,
            info: ac2_weenie::FellowInfo::default(),
        };
        markers.notice(
            PLAYER,
            &update,
            std::slice::from_ref(&me),
            &[me.clone(), ann.clone()],
        );
        assert_eq!(
            markers.take_requests(),
            [add(2, 0x20, "Ann")],
            "a new member"
        );
        markers.notice(
            PLAYER,
            &update,
            &[me.clone(), ann.clone()],
            &[me.clone(), ann.clone()],
        );
        assert!(
            markers.take_requests().is_empty(),
            "an update of a listed member"
        );
        let moved = GameplayNotice::UpdateFellowCell {
            fellow: 2,
            cell: 0x30,
        };
        let both = [me.clone(), ann.clone()];
        markers.notice(PLAYER, &moved, &both, &both);
        assert_eq!(markers.take_requests(), [add(2, 0x30, "Ann")]);
        let own = GameplayNotice::UpdateFellowCell {
            fellow: PLAYER,
            cell: 0x40,
        };
        markers.notice(PLAYER, &own, &both, &both);
        assert!(markers.take_requests().is_empty(), "the player has no note");
        markers.notice(
            PLAYER,
            &GameplayNotice::ClearFellow(2),
            &both,
            std::slice::from_ref(&me),
        );
        assert_eq!(
            markers.take_requests(),
            [MarkerRequest::Remove {
                kind: FELLOW_MARKER,
                id: 2
            }]
        );
        markers.notice(PLAYER, &GameplayNotice::ClearFellowship, &[me], &[]);
        assert_eq!(
            markers.take_requests(),
            [MarkerRequest::Clear(FELLOW_MARKER)]
        );
    }
}
