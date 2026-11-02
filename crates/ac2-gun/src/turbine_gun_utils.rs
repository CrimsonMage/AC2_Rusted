//! `TurbineGunUtils` (`ac2/shared/src/gun/turbinegunutils.cpp.c`): the helpers around
//! [`crate::TurbineGun`], and the seam where Rust cannot do what retail did (load `GunDll.dll`).
//!
//! Retail's `Init` calls `LoadLibraryA` and `TurbineGunUtils::GunCreateInstance`, which calls the
//! DLL's exported `DllGetClassObject`. Rust cannot load that DLL (GunDll is [`crate::GunObject`]
//! here, behind [`GunModule`]), so the loading is a [`GunLoader`] the caller supplies and
//! `GunCreateInstance` runs its retail steps over a [`GunModule`].

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::hr_table::GUN_ERRORS;
use crate::interfaces::{Guid, GunObject};
use crate::Hresult;

/// `IID_IClassFactory` {00000001-0000-0000-C000-000000000046} (Legions image 0x009FAA0C).
pub const IID_ICLASS_FACTORY: Guid = Guid {
    data1: 0x0000_0001,
    data2: 0,
    data3: 0,
    data4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
};

/// `IClassFactory`, the part `GunCreateInstance` uses: `CreateInstance(NULL, iid, out)` (vtable
/// `+0xC`). Dropping the factory is its `Release()` (`+8`).
pub trait ClassFactory {
    /// `IClassFactory::CreateInstance(NULL, riid, ppv)`: the `HRESULT` and the object.
    fn create_instance(&mut self, riid: &Guid) -> (Hresult, Option<Box<dyn GunObject>>);
}

/// A loaded GunDll (retail's `HMODULE`): its `DllGetClassObject` export.
pub trait GunModule {
    /// `GetProcAddress(hLib, "DllGetClassObject")` then `DllGetClassObject(rclsid, riid, &factory)`:
    /// `None` when the DLL has no such export; else the call's `HRESULT` and the factory it
    /// returned (`riid` is `IID_IClassFactory`).
    fn dll_get_class_object(
        &mut self,
        rclsid: &Guid,
        riid: &Guid,
    ) -> Option<(Hresult, Option<Box<dyn ClassFactory>>)>;

    /// `GetLastError()`, which `GunCreateInstance` returns when the export is missing.
    fn get_last_error(&self) -> u32;
}

/// `LoadLibraryA(path)`. `Err` carries `GetLastError()` (retail's `(uint)hmod <= 0x20` case).
pub trait GunLoader {
    fn load_library(&mut self, path: &str) -> Result<Box<dyn GunModule>, u32>;
}

/// `TurbineGunUtils::GunCreateInstance` (admin 0x00607780, Legions 0x005C8620): the `HRESULT`
/// and the created object.
///
/// Steps: `fn = GetProcAddress(dll, "DllGetClassObject")`, NULL -> `GetLastError()` (a positive
/// number, which `TurbineGun::Init` takes for success); `hr = fn(clsid, IID_IClassFactory, &factory)`,
/// negative -> `hr`; `hr = factory->CreateInstance(NULL, iid, out)`; on `hr >= 0` with a factory,
/// `factory->Release()` (a failed `CreateInstance` does not release it, so retail leaks the
/// factory there; here it is dropped).
pub fn gun_create_instance(
    dll: &mut dyn GunModule,
    clsid: &Guid,
    iid: &Guid,
) -> (Hresult, Option<Box<dyn GunObject>>) {
    let Some((hr, factory)) = dll.dll_get_class_object(clsid, &IID_ICLASS_FACTORY) else {
        return (Hresult(dll.get_last_error()), None);
    };
    if !hr.succeeded() {
        return (hr, None);
    }
    let Some(mut factory) = factory else {
        // open: retail calls `factory->CreateInstance` through a NULL factory when
        // `DllGetClassObject` succeeds but returns none; the result is not defined by the
        // decompile, so none is invented.
        return (hr, None);
    };
    factory.create_instance(iid)
}

/// `TurbineGunUtils::GetModeText` (admin 0x006075D0, Legions 0x005C8470): the debug text for a
/// presence mode. Starts empty; appends, each only when `dwBufferSize` allows (`cap`), the
/// words for mode 0 ("Logged Out "), bit 1 ("Logged In "), 2 ("Away From Keyboard "), 4
/// ("Do Not Disturb "), 8 ("In Game ") and 0x10 ("In Chat "), reducing `cap` by each word's
/// length plus one. Nothing in the client uses the text.
pub fn get_mode_text(mode: u32, cap: u32) -> String {
    let mut text = String::new();
    let mut cap = cap;
    if mode == 0 {
        if cap < 0xd {
            return text;
        }
        text.push_str("Logged Out ");
        cap -= 0xc;
    }
    if mode & 1 != 0 {
        if cap < 0xc {
            return text;
        }
        text.push_str("Logged In ");
        cap -= 0xb;
    }
    if mode & 2 != 0 {
        if cap < 0x15 {
            return text;
        }
        text.push_str("Away From Keyboard ");
        cap -= 0x14;
    }
    if mode & 4 != 0 {
        if cap < 0x11 {
            return text;
        }
        text.push_str("Do Not Disturb ");
        cap -= 0x10;
    }
    if mode & 8 != 0 {
        if cap < 10 {
            return text;
        }
        text.push_str("In Game ");
        cap -= 9;
    }
    if mode & 0x10 != 0 && 9 < cap {
        text.push_str("In Chat ");
    }
    text
}

/// `TurbineGunUtils::GetRoomModeText` (admin 0x00607690, Legions 0x005C8530): the debug text for
/// a room mode: 1 "STANDARD" (needs `cap` > 9), 2 "USER" (> 5), 4 "PASSWORD" (> 9), 8 "PRIVATE"
/// (> 8), 0x10 "SECRET" (> 7), any other mode "Unknown Mode " (> 0xe).
pub fn get_room_mode_text(mode: u32, cap: u32) -> String {
    let (text, min) = match mode {
        1 => ("STANDARD", 9),
        2 => ("USER", 5),
        4 => ("PASSWORD", 9),
        8 => ("PRIVATE", 8),
        0x10 => ("SECRET", 7),
        _ => ("Unknown Mode ", 0xe),
    };
    if min < cap {
        text.to_owned()
    } else {
        String::new()
    }
}

/// `TurbineGunUtils::StringFromHR` (admin 0x006078B0, Legions 0x005C8750): the name of `hr` in
/// the 112-entry `aGunErrors` table (first match). A code the table lacks goes through
/// `FormatMessageA(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM |
/// FORMAT_MESSAGE_IGNORE_INSERTS, NULL, hr, 0x400, ...)`, a Windows message table Rust has no
/// access to: a code outside the table gets no text here (an empty string), where retail
/// returned the system message text. The text is debug output only.
pub fn string_from_hr(hr: Hresult) -> String {
    GUN_ERRORS
        .iter()
        .find(|(code, _)| *code == hr.0)
        .map(|(_, name)| (*name).to_owned())
        .unwrap_or_default()
}

/// `TurbineGunUtils::PrintCmdHRESULT` (admin 0x00607930, Legions 0x005C87D0): `true` when
/// `hr >= 0` (a success). On a failure retail prints `GUN: failed %s, hr=0x%x == %s` through
/// `Turbine::Debug::printf` (console output, no effect on the client): not reproduced, so `_cmd`
/// is unused.
pub fn print_cmd_hresult(_cmd: &str, hr: Hresult) -> bool {
    hr.succeeded()
}

/// What `TurbineGun::S_pTurbineGun` reaches: the world name and the `@World` suffix, behind
/// the one `TurbineGun` the constructor made current.
#[derive(Debug, Default)]
pub struct WorldNames {
    inner: Mutex<NameState>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NameState {
    /// `m_worldName` (+0x80), `L""` until `SetWorldName`.
    pub(crate) world_name: String,
    /// `m_worldSuffix` (+0x84), the null string until `SetWorldName`, then `L"@" + name`.
    pub(crate) world_suffix: String,
}

impl WorldNames {
    pub(crate) fn lock(&self) -> MutexGuard<'_, NameState> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// `TurbineGun::S_pTurbineGun` (Legions 0x00AF1B58): set to `this` by the constructor and never
/// cleared (not even by the destructor). The statics that read it only use the world name and
/// suffix, so it holds those. (In this crate's own unit tests it is per thread, so tests that
/// build a `TurbineGun` do not disturb each other.)
#[cfg(not(test))]
static S_P_TURBINE_GUN: Mutex<Option<Arc<WorldNames>>> = Mutex::new(None);
#[cfg(test)]
thread_local! {
    static S_P_TURBINE_GUN: std::cell::RefCell<Option<Arc<WorldNames>>> =
        const { std::cell::RefCell::new(None) };
}

pub(crate) fn set_s_p_turbine_gun(names: Arc<WorldNames>) {
    #[cfg(not(test))]
    {
        *S_P_TURBINE_GUN
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(names);
    }
    #[cfg(test)]
    S_P_TURBINE_GUN.with(|s| *s.borrow_mut() = Some(names));
}

/// `S_pTurbineGun`'s names, `None` while it is NULL (no `TurbineGun` was ever made).
pub fn s_p_turbine_gun() -> Option<Arc<WorldNames>> {
    #[cfg(not(test))]
    {
        S_P_TURBINE_GUN
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    #[cfg(test)]
    S_P_TURBINE_GUN.with(|s| s.borrow().clone())
}

/// `TurbineGunUtils::GetWorldName` (admin 0x00607830, Legions 0x005C86D0): `S_pTurbineGun->m_worldName`,
/// the null string (empty) when `S_pTurbineGun` is NULL. `gTurbineGunUtils::CallEFun_GetWorldName`
/// (EFun 0x200A0001, Legions 0x0046E8C0) returns it to scripts.
pub fn get_world_name() -> String {
    match s_p_turbine_gun() {
        Some(names) => names.lock().world_name.clone(),
        None => String::new(),
    }
}

/// `TurbineGunUtils::GetWorldSuffix` (admin 0x00607870, Legions 0x005C8710): `m_worldSuffix`
/// (`@World`), empty when `S_pTurbineGun` is NULL. `gTurbineGunUtils::CallEFun_GetWorldSuffix`
/// (EFun 0x200A0002, Legions 0x0046E940) returns it to scripts.
pub fn get_world_suffix() -> String {
    match s_p_turbine_gun() {
        Some(names) => names.lock().world_suffix.clone(),
        None => String::new(),
    }
}

/// `TurbineGunUtils::GlobalToLocalName` (admin 0x006079A0, Legions 0x005C8840): the name
/// unchanged when `S_pTurbineGun` is NULL, else `S_pTurbineGun->GlobalToLocalName(name)` (which
/// does not test `S_fStripWorldNameFromRoomChatEvents`). The `CCli*Result::Handle*` list
/// handlers and `CCliFriendsEvent::FriendStart` call this form.
pub fn global_to_local_name(name: &str) -> String {
    match s_p_turbine_gun() {
        None => name.to_owned(),
        Some(names) => {
            let state = names.lock();
            crate::turbine_gun::names::global_to_local(name, &state.world_suffix)
        }
    }
}
