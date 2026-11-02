//! `gmClient`'s GUN command-line options and members (`gmClient::BuildCommandLineArgs` admin
//! 0x00403650 / Legions 0x004035C0, the constructor admin 0x004034A0 / Legions 0x00403410,
//! `EvaluateCommandLineArg` admin 0x00403AE0 / Legions 0x00403A50, `OnCommandLineEvaluationDone`
//! admin 0x00403B60 / Legions 0x00403AD0). Legions and admin are instruction-identical here.

/// The default `-migrationurl` (the constructor's `m_szMigrationURL`).
pub const DEFAULT_MIGRATION_URL: &str =
    "https://acbm.turbinegames.com/IISAcBillingMigration/IISAcBillingMigration.dll?ac2";

/// `-gunport`'s default, the constructor's `m_gunPort` = 0x0906.
pub const DEFAULT_GUN_PORT: u16 = 2310;

/// `-migration1Timeout` / `-migration2Timeout` defaults: the globals' initial value 0x1F40 ms
/// (Legions 0x00A30B3C / 0x00A30B40, admin 0x00AC9B3C / 0x00AC9B40).
pub const DEFAULT_MIGRATION_TIMEOUT_MS: i32 = 8000;

/// The GUN fields of `gmClient` and the options that set them. GUN is off exactly when
/// [`GunOptions::gun_server`] is empty: there is no default server and no switch to turn GUN
/// off (`m_len == 1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GunOptions {
    /// `-gunserver <hostname/IP Addr>` (`m_gunServer`, +0xC4): "Connect to the gun server
    /// specified."
    pub gun_server: String,
    /// `-gunport <port>` (`m_gunPort`, +0xCC): type 0x62, a port.
    pub gun_port: u16,
    /// `-gunclientdll <file>` (`m_gundll`, +0xC8): "Name of alternate gundll"; empty means
    /// `"GunDll.dll"` in `TurbineGun::Init`. Ignored by this client (it has its own `CGun`).
    pub gun_client_dll: String,
    /// `-gunticket` (`m_fUseGunTicket`, +0xD4): a flag, "Login with gun ticketing. You must
    /// specify the gunserver argument as well."
    pub use_gun_ticket: bool,
    /// `-gunticketoverride <ticket>` (`m_GunTicketOverride`, +0xDC): "The string of the gun auth
    /// ticket to use. If ommitted but -gunticket is used, then the client will get its gun
    /// ticket from GunDll."
    pub gun_ticket_override: String,
    /// `-migrationurl <url>` (`m_szMigrationURL`, +0xE0).
    pub migration_url: String,
    /// `-migration1Timeout <ms>` (type 0x22, `strtol(text, 0, 0)`, the global at Legions
    /// 0x00A30B3C).
    pub migration1_timeout: i32,
    /// `-migration2Timeout <ms>` (global Legions 0x00A30B40).
    pub migration2_timeout: i32,
}

impl Default for GunOptions {
    /// The constructor's values.
    fn default() -> Self {
        Self {
            gun_server: String::new(),
            gun_port: DEFAULT_GUN_PORT,
            gun_client_dll: String::new(),
            use_gun_ticket: false,
            gun_ticket_override: String::new(),
            migration_url: DEFAULT_MIGRATION_URL.to_owned(),
            migration1_timeout: DEFAULT_MIGRATION_TIMEOUT_MS,
            migration2_timeout: DEFAULT_MIGRATION_TIMEOUT_MS,
        }
    }
}

/// The option names `BuildCommandLineArgs` registers for GUN (long names only, short character
/// `'\0'`), with whether they take a value.
pub const OPTIONS: [(&str, bool); 8] = [
    ("gunserver", true),
    ("gunport", true),
    ("gunclientdll", true),
    ("gunticket", false),
    ("gunticketoverride", true),
    ("migrationurl", true),
    ("migration1Timeout", true),
    ("migration2Timeout", true),
];

impl GunOptions {
    /// Whether `name` is one of the GUN options ([`OPTIONS`]) and whether it takes a value.
    pub fn takes_value(name: &str) -> Option<bool> {
        OPTIONS.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
    }

    /// Whether GUN is on: `m_gunServer` is not empty.
    pub fn gun_enabled(&self) -> bool {
        !self.gun_server.is_empty()
    }

    /// Applies one option the way `BuildCommandLineArgs`' table stores it. `-gunport` is type
    /// 0x62 (`sscanf("%hu")`); `-migration1Timeout` / `-migration2Timeout` are `strtol(text, 0, 0)`; a flag stores its
    /// value 1. Returns an error text for a bad value.
    pub fn apply(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        let text = value.unwrap_or("");
        match name {
            "gunserver" => self.gun_server = text.to_owned(),
            // Type word 0x62: `sscanf(text, "%hu", &gunPort)`; a text with no number is an
            // option error (`ArgumentParser::EvaluateCommandLineArg`, Legions 0x0060A0B0,
            // argparser.cpp.c:331-337).
            "gunport" => {
                self.gun_port =
                    scan_hu(text).ok_or_else(|| format!("--gunport: {text:?} is not a number"))?;
            }
            "gunclientdll" => self.gun_client_dll = text.to_owned(),
            "gunticket" => self.use_gun_ticket = true,
            "gunticketoverride" => self.gun_ticket_override = text.to_owned(),
            "migrationurl" => self.migration_url = text.to_owned(),
            "migration1Timeout" => self.migration1_timeout = c_long(text),
            "migration2Timeout" => self.migration2_timeout = c_long(text),
            _ => return Err(format!("unknown GUN option {name}")),
        }
        Ok(())
    }

    /// `gmClient::OnCommandLineEvaluationDone`'s GUN rule (admin 0x00403B60, Legions
    /// 0x00403AD0): `-gunticket` without `-gunserver` is refused.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.gun_server.is_empty() && self.use_gun_ticket {
            return Err("You must specify a gun server in order to use GUN ticketing");
        }
        Ok(())
    }
}

/// `strtol(text, 0, 0)`: base 0 (a `0x` prefix is hex, a leading `0` octal), leading white
/// space and a sign skipped, parsing up to the first character that is not a digit; no digits
/// give 0.
pub fn c_long(text: &str) -> i32 {
    let t = text.trim_start();
    let (negative, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let (radix, digits) = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        (16, h)
    } else if t.starts_with('0') && t.len() > 1 {
        (8, &t[1..])
    } else {
        (10, t)
    };
    let end = digits
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or(digits.len());
    let value = i64::from_str_radix(&digits[..end], radix).unwrap_or(0);
    let value = if negative { -value } else { value };
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// `sscanf(text, "%hu", &value)`: leading white space, an optional sign, decimal digits only
/// (`0x910` reads as 0, `012` as 12), anything after them ignored; the value is stored in 16
/// bits (a longer number wraps, a minus sign negates). `None` when no digit is found: the match
/// count is 0 and the argument parser reports an error.
pub fn scan_hu(text: &str) -> Option<u16> {
    let t = text.trim_start();
    let (negative, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let digits: &str = &t[..t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len())];
    if digits.is_empty() {
        return None;
    }
    let value = digits.bytes().fold(0u32, |v, d| {
        v.wrapping_mul(10).wrapping_add(u32::from(d - b'0'))
    });
    let value = if negative {
        value.wrapping_neg()
    } else {
        value
    };
    Some(value as u16)
}

/// The options `Turbine::Client::BuildCommandLineArgs` registers that GUN's chat selection
/// reads (Legions client.cpp.c:1472-1490): `-tchat` ("Use the Turbine chat room system.", a flag
/// storing 1 into `Client+0xB0` = `m_fUseTurbineChat`) and `-tchatdll` ("<dll path> : Specifies
/// the dll to load for chat client services", a string into `Client+0xB4`).
/// `gmClient::EvaluateCommandLineArg` (Legions 0x00403A50) then sets
/// `S_fUseTurbineChatInsteadOfGUN = 1` and `S_fStripWorldNameFromRoomChatEvents = false` for
/// `tchat`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurbineClientOptions {
    /// `Client+0xB0`.
    pub tchat: bool,
    /// `Client+0xB4`. Stored; this client has no chat DLL to load (its Turbine chat is built in).
    pub tchat_dll: String,
}

impl TurbineClientOptions {
    /// Whether `name` is one of these options and whether it takes a value.
    pub fn takes_value(name: &str) -> Option<bool> {
        match name {
            "tchat" => Some(false),
            "tchatdll" => Some(true),
            _ => None,
        }
    }

    pub fn apply(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        match name {
            "tchat" => self.tchat = true,
            "tchatdll" => self.tchat_dll = value.unwrap_or("").to_owned(),
            _ => return Err(format!("unknown option {name}")),
        }
        Ok(())
    }
}
