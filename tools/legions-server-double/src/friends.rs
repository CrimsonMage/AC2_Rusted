//! The friends list the GUN service keeps for the player, as a stand-in for the GUN answers the
//! client's script takes (`TurbineGun`'s friends). The GUN wire is not modelled here: a test passes
//! the client's GUN requests to this book and feeds its answers back as results and events. The
//! results are GUN codes (`gun_errors.csv`, the client converts them): a new friend and a listed
//! delete are done (0); a duplicate add is `ZT_E_ALREADY_IN_LIST` and a delete of a non-friend is
//! `ZT_E_NOT_IN_LIST`, both as the names say. An add of a name the directory does not hold is
//! `ZT_E_INVALID_USER`: the message names that code, but the code the friends server sends for an
//! unknown name is not in GunDll, so this is the book's inference.

/// `ZT_S_OK`, the result of a done request.
pub const GUN_DONE: u32 = 0x0000_0000;
/// `ZT_E_ALREADY_IN_LIST`: the add of a name already on the list.
pub const GUN_ALREADY_IN_LIST: u32 = 0xc112_1009;
/// `ZT_E_NOT_IN_LIST`: the delete of a name not on the list.
pub const GUN_NOT_IN_LIST: u32 = 0xc112_1401;
/// `ZT_E_INVALID_USER`: the add of a name the directory does not hold.
pub const GUN_INVALID_USER: u32 = 0xc112_0012;

/// One friend's status: the name, the location, the mode (`TurbineUserStatus`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendRecord {
    pub name: String,
    pub location: String,
    pub mode: u32,
}

/// The player's friends, in the order they were added, and the names the directory holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendsBook {
    friends: Vec<FriendRecord>,
    directory: Vec<String>,
}

impl FriendsBook {
    /// A book whose directory holds `names` (the players a friend can be).
    pub fn with_directory(names: &[&str]) -> Self {
        Self {
            friends: Vec::new(),
            directory: names.iter().map(|name| (*name).to_owned()).collect(),
        }
    }

    /// The friends, in the order they were added.
    pub fn friends(&self) -> &[FriendRecord] {
        &self.friends
    }

    /// The friends' names, in the order they were added.
    pub fn names(&self) -> Vec<String> {
        self.friends.iter().map(|f| f.name.clone()).collect()
    }

    /// Adds `name` (its directory spelling is kept): the GUN result, `GUN_DONE` or a refusal.
    pub fn add(&mut self, name: &str) -> u32 {
        if self
            .friends
            .iter()
            .any(|f| f.name.eq_ignore_ascii_case(name))
        {
            return GUN_ALREADY_IN_LIST;
        }
        let Some(listed) = self
            .directory
            .iter()
            .find(|n| n.eq_ignore_ascii_case(name))
            .cloned()
        else {
            return GUN_INVALID_USER;
        };
        self.friends.push(FriendRecord {
            name: listed,
            location: String::new(),
            mode: 0,
        });
        GUN_DONE
    }

    /// Removes `name`: the GUN result, `GUN_DONE` or `GUN_NOT_IN_LIST`.
    pub fn remove(&mut self, name: &str) -> u32 {
        match self
            .friends
            .iter()
            .position(|f| f.name.eq_ignore_ascii_case(name))
        {
            Some(index) => {
                self.friends.remove(index);
                GUN_DONE
            }
            None => GUN_NOT_IN_LIST,
        }
    }

    /// Sets a friend's location and mode (the GUN status change). `false` when `name` is not a
    /// friend.
    pub fn set_status(&mut self, name: &str, location: &str, mode: u32) -> bool {
        match self
            .friends
            .iter_mut()
            .find(|f| f.name.eq_ignore_ascii_case(name))
        {
            Some(friend) => {
                friend.location = location.to_owned();
                friend.mode = mode;
                true
            }
            None => false,
        }
    }
}
