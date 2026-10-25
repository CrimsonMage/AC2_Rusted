//! The friends list the GUN service keeps for the player, as a stand-in for the GUN results the
//! client's script takes (`TurbineGun`'s friends). The GUN wire is not modelled here: a test
//! passes the client's GUN requests to this book and feeds its answers back as results. An add
//! of a new name and a delete of a listed one answer success (weenie 0, `CTBullet::ConvertToWeenieResult`
//! of GUN 0). A duplicate add or an unknown delete answers nothing: the GUN codes for those are not
//! traced, so the book does not guess them.

/// The player's friends, in the order they were added.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendsBook {
    names: Vec<String>,
}

/// The weenie result of a successful friend request.
pub const FRIEND_DONE: u32 = 0;

impl FriendsBook {
    /// The friends, in the order they were added.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Adds `name`: its result, or `None` when the name is already a friend.
    pub fn add(&mut self, name: &str) -> Option<u32> {
        if self.names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            return None;
        }
        self.names.push(name.to_owned());
        Some(FRIEND_DONE)
    }

    /// Removes `name`: its result, or `None` when it is not a friend.
    pub fn remove(&mut self, name: &str) -> Option<u32> {
        let index = self
            .names
            .iter()
            .position(|n| n.eq_ignore_ascii_case(name))?;
        self.names.remove(index);
        Some(FRIEND_DONE)
    }
}
