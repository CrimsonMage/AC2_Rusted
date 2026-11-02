//! The world name and the `@World` suffix: `SetWorldName`, `LocalToGlobalName`,
//! `GlobalToLocalName`.

use super::TurbineGun;

/// The `GlobalToLocalName` rule over a suffix (admin 0x00606590, Legions 0x005C7430):
/// `PStringBaseIter(name).FindChar(L'@', reverse = false)` finds the FIRST `@` (none -> the name
/// is returned whole); `Split` then gives `left = name.substring(0, curr - 1)` (the characters
/// before it; for `@` at index 0 `curr - 1` is 0xFFFFFFFF, which `substring` clamps to the whole
/// string) and `right = name.substring(curr, strlen)` (from the `@` to the end, the `@`
/// included; `PStringBase<ushort>::substring` Legions 0x00418DA0, `Split` Legions 0x0041F050).
/// `wcscmp(right, suffix) == 0` (exact, case-sensitive) returns `left`, else the name whole.
pub(crate) fn global_to_local(name: &str, suffix: &str) -> String {
    let Some(curr) = name.find('@') else {
        return name.to_owned();
    };
    let left = if curr == 0 { name } else { &name[..curr] };
    let right = &name[curr..];
    if right == suffix {
        left.to_owned()
    } else {
        name.to_owned()
    }
}

/// The `LocalToGlobalName` rule over a suffix (admin 0x006064B0, Legions 0x005C7350): when
/// `wcsstr(name, L"@")` finds no `@` anywhere in the name AND `m_worldSuffix.m_len != 1` AND the
/// suffix buffer is not the null buffer (an empty suffix here), the suffix's characters are
/// appended; otherwise the name is returned unchanged.
pub(crate) fn local_to_global(name: &str, suffix: &str) -> String {
    if !name.contains('@') && !suffix.is_empty() {
        format!("{name}{suffix}")
    } else {
        name.to_owned()
    }
}

impl TurbineGun {
    /// `NSTurbineGun::TurbineGun::SetWorldName` (admin 0x004033A0, Legions 0x00403310; in
    /// `gmclient.cpp.c`): `int SetWorldName(PStringBase<ushort>* name)`. When `m_worldName` is
    /// not empty (`m_len != 1`: already set) returns 0. Else `m_worldName = *name`,
    /// `m_worldSuffix = L"@" + *name`, returns 1. Set at most once (an empty name leaves it
    /// unset). Called by `gmClient::RecvNotice_WorldName`.
    pub fn set_world_name(&self, name: &str) -> bool {
        let mut state = self.names.lock();
        if !state.world_name.is_empty() {
            return false;
        }
        state.world_name = name.to_owned();
        state.world_suffix = format!("@{name}");
        true
    }

    /// `m_worldName`.
    pub fn world_name(&self) -> String {
        self.names.lock().world_name.clone()
    }

    /// `m_worldSuffix`.
    pub fn world_suffix(&self) -> String {
        self.names.lock().world_suffix.clone()
    }

    /// `NSTurbineGun::TurbineGun::LocalToGlobalName` (admin 0x006064B0, Legions 0x005C7350).
    /// Used by FriendAdd/Delete, IgnoredAdd/Delete, SendToUserEx and CallCreate's target; not by
    /// SetLocation, SendToRoomEx or the names in events.
    pub fn local_to_global_name(&self, name: &str) -> String {
        let state = self.names.lock();
        local_to_global(name, &state.world_suffix)
    }

    /// `NSTurbineGun::TurbineGun::GlobalToLocalName` (admin 0x00606590, Legions 0x005C7430).
    pub fn global_to_local_name(&self, name: &str) -> String {
        let state = self.names.lock();
        global_to_local(name, &state.world_suffix)
    }
}
