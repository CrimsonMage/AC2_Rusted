//! The remaining interfaces of `CGun`: `IGunAccount`, `IGunNews`, `IGunHost`. The account calls
//! send `CreateAccount` / `GetAccountStatus` / `GetAccountName` requests (0x2102-0x2104) and keep
//! `SetAccountValue`'s `Email` registry write; the news call is a WinINet thread and the hosted-game
//! calls need GunDll's DP8 *server* side, which is outside the client port.

use super::cgun::CGun;
use super::registry::EMAIL;
use super::requests::wcslen;
use super::tickets::E_NOTIMPL;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

const E_INVALIDARG: Hresult = Hresult(0x8007_0057);
const E_FAIL: Hresult = Hresult(0x8000_FFFF);

impl IGunNews for CGun {
    /// `CGun::GetNews` (0x1000f8c0). **`E_NOTIMPL` is NOT retail's answer; it is a recorded
    /// deviation.** Retail: a NULL host is `E_POINTER` (0x80004003); one worker only (a busy flag at
    /// `+0xb0`; a second call while it runs is `E_FAIL` 0x80004005); otherwise a thread
    /// (`FUN_1000f540`) does WinINet `InternetOpenA`, `InternetConnectA(host, port, HTTP)`,
    /// `HttpOpenRequestA("GET", path)`, `HttpSendRequestA`, reads the body, requires HTTP status 200
    /// (else `E_FAIL`), converts UTF-8 to wide text and posts a `GetNewsRes` (`Cx` = the context, the
    /// result and the text) to the queue; the call itself returns `S_OK`. No client caller exists.
    // open: not ported (WinINet HTTP thread).
    fn get_news(&self, _a: &str, _b: &str, _c: u16, _context: Context) -> Hresult {
        E_NOTIMPL
    }
}

impl IGunAccount for CGun {
    /// `CGun::CreateAccount` (0x1000f170): the connection (via `this+0x44`), then the password
    /// (`p2`, NULL -> `L""`) is encrypted with the session key (`FUN_10001a40`) into the
    /// `<CreateAccount User><Password/>` request (0x2102). `p3` and `p4` are not used.
    fn create_account(&self, a: &str, b: &str, _c: &str, _d: &str, context: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        let pw = match self.encrypt_text(b) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let mut m = self.new_request(0x2102, context);
        m.set_str("User", Some(a));
        m.set_blob("Password", Some(&pw));
        self.send_request(&m)
    }
    /// `CGun::ChangePassword` (0x1000f250): `E_NOTIMPL`.
    fn change_password(&self, _a: &str, _b: &str, _context: Context) -> Hresult {
        E_NOTIMPL
    }
    /// `CGun::ChangePasswordFromHint` (0x1000f260): `E_NOTIMPL`.
    fn change_password_from_hint(
        &self,
        _a: &str,
        _b: &str,
        _c: &str,
        _context: Context,
    ) -> Hresult {
        E_NOTIMPL
    }
    /// `CGun::GetAccountStatus` (0x1000f270): `<GetAccountStatus User>` (0x2103).
    fn get_account_status(&self, a: &str, context: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        let mut m = self.new_request(0x2103, context);
        m.set_str("User", Some(a));
        self.send_request(&m)
    }
    /// `CGun::GetPasswordHint` (0x1000f310): `E_NOTIMPL`.
    fn get_password_hint(&self, _a: &str, _context: Context) -> Hresult {
        E_NOTIMPL
    }
    /// `CGun::GetAccountName` (0x1000f320): an empty name is `0x80070057`, one over 50 characters
    /// `ZT_E_STR_TOO_LONG`; `<GetAccountName PersonaName>` (0x2104).
    fn get_account_name(&self, a: &str, context: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        if wcslen(a) == 0 {
            return E_INVALIDARG;
        }
        if wcslen(a) >= 0x33 {
            return ZT_E_STR_TOO_LONG;
        }
        let mut m = self.new_request(0x2104, context);
        m.set_str("PersonaName", Some(a));
        self.send_request(&m)
    }
    /// `CGun::SetAccountValue` (0x1000f3e0): the type must be 0 (else `0x80070057`); the value is a
    /// UTF-16 string of at most 50 characters (`ZT_E_STR_TOO_LONG`), stored as the `Email` value
    /// (REG_BINARY, `2 * length` bytes without the NUL); a registry failure is `E_FAIL`.
    fn set_account_value(&self, a: AccountValueType, b: &[u8]) -> Hresult {
        if a.0 != 0 {
            return E_INVALIDARG;
        }
        let units: Vec<u16> = b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        if units.len() >= 0x33 {
            return ZT_E_STR_TOO_LONG;
        }
        let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
        let reg = self.env.borrow().registry.clone();
        if reg.set_binary(EMAIL, &bytes) {
            Hresult::S_OK
        } else {
            E_FAIL
        }
    }
}

impl IGunHost for CGun {
    /// `CGun::CreateGame` (0x1000d920): not initialised -> `ZT_E_NOT_INITED`; an empty name
    /// `0x80070057`; a game already hosted `ZT_E_ALREADY_INITED`. **A valid call answering
    /// `E_NOTIMPL` is NOT retail's answer; it is a recorded deviation.** Retail builds the two
    /// 0x3c-byte game objects at `+0x90`/`+0x94` (`FUN_1000d6c0`, `FUN_1000d2d0`), sets the "GameV" and
    /// "Locale" attributes and creates a DP8 server; an AC2 client never hosts.
    // open: the hosted game is not ported.
    fn create_game(&self, a: &str) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        if a.is_empty() {
            return E_INVALIDARG;
        }
        E_NOTIMPL
    }
    /// `CGun::EndGame` (0x1000d760): with no hosted game `ZT_E_INVALID_OP`.
    fn end_game(&self) -> Hresult {
        ZT_E_INVALID_OP
    }
    /// `CGun::AddPlayer` (0x1000d330): initialised and a game, else `ZT_E_NOT_INITED`.
    fn add_player(&self, _a: u32, _b: &str) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::DeletePlayer` (0x1000d440): `ZT_E_NOT_INITED` without a game.
    fn delete_player(&self, _a: u32) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::GetNumPlayers` (0x1000cc20): `ZT_E_NOT_INITED` without a game.
    fn get_num_players(&self, _a: &mut u32) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::EnumPlayers` (0x1000cc60): `ZT_E_NOT_INITED` without a game.
    fn enum_players(&self, _sink: &mut dyn IGunHostEnum) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::SetItemValue` (0x1000ccb0): `ZT_E_NOT_INITED` without a game.
    fn set_item_value(&self, _a: u32, _b: &str, _c: &str) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::GetItemValue` (0x1000cda0): `ZT_E_NOT_INITED` without a game.
    fn get_item_value(&self, _a: u32, _b: &str, _c: &mut [u16]) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::DeleteItemValue` (0x1000ce50): `ZT_E_NOT_INITED` without a game.
    fn delete_item_value(&self, _a: u32, _b: &str) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::GetNumItemValues` (0x1000cf20): `ZT_E_NOT_INITED` without a game.
    fn get_num_item_values(&self, _a: u32, _b: &mut u32) -> Hresult {
        ZT_E_NOT_INITED
    }
    /// `CGun::EnumItemValues` (0x1000cfa0): `ZT_E_NOT_INITED` without a game.
    fn enum_item_values(&self, _a: u32, _sink: &mut dyn IGunHostEnum) -> Hresult {
        ZT_E_NOT_INITED
    }
}
