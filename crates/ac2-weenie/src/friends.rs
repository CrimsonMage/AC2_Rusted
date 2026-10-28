//! A friend as the friends list shows it: the `TurbineUserStatus` the GUN service sends for
//! each friend (`gun/helper/turbineuserstatus.cpp.c`: the name, the location, the mode). The
//! GUN wire is not modelled here; the client delivers these values decoded.

/// `TurbineUserStatus::m_Mode` bit for an away friend (`gmFriendsUI::StatusTable`'s AFK entry).
pub const MODE_AWAY: u32 = 2;
/// `TurbineUserStatus::m_Mode` bit for a do-not-disturb friend (the DND entry).
pub const MODE_DO_NOT_DISTURB: u32 = 4;

/// `TurbineUserStatus`: a friend's name, where they are (`m_Location`) and their mode
/// (`m_Mode`, the GUN status bits).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendInfo {
    /// `m_Name`.
    pub name: String,
    /// `m_Location`: the place the GUN service gives (empty for a status event; the list's
    /// `FriendStart` carries it). The friends page does not split on it.
    pub location: String,
    /// `m_Mode`: 0 is offline; otherwise online, with [`MODE_AWAY`] and
    /// [`MODE_DO_NOT_DISTURB`] bits (`gmFriendsUI::RecvNotice_GUNResult_FriendStatus` and
    /// `RecvNotice_GUNResult_GetFriendsList` list a friend online when `m_Mode != 0`).
    pub mode: u32,
}

impl FriendInfo {
    /// Whether the friend is listed online (`m_Mode != 0`).
    pub fn is_online(&self) -> bool {
        self.mode != 0
    }
}
