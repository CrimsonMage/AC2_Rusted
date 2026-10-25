//! A friend as the friends list shows it: the `TurbineUserStatus` the GUN service sends for
//! each friend (`gun/helper/turbineuserstatus.cpp.c`: the name, the location, the mode). The
//! GUN wire is not modelled here; the client delivers these values decoded.

/// `TurbineUserStatus`: a friend's name, where they are (`m_Location`) and their mode
/// (`m_Mode`, the GUN status bits, not traced).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendInfo {
    /// `m_Name`.
    pub name: String,
    /// `m_Location`: the place the friend is in (empty when offline or unknown).
    pub location: String,
    /// `m_Mode`.
    pub mode: u32,
}
