//! The options page's model: `gmClientUI` (the client's preference sections and their Apply,
//! Recall, Default and Changed) and `gmOptionUI`'s close and confirmation (`Show`,
//! `MakeConfirmDialog`, `DoConfirm`). The page's widgets are not modelled: its OK button runs
//! [`ClientPrefs::options_apply`], its Cancel runs [`ClientPrefs::options_recall`], its Default
//! runs [`ClientPrefs::options_default`], and closing with changes asks [`OptionsWindow`].
//!
//! Apply saves the file on a worker thread (`ClientPrefs::save`), except when the render section
//! changed: retail then asks the player to verify the render options (`VerifyRenderOptions`) and
//! does not save. That question is not modelled, so such an apply leaves the file as it was.

use std::io;
use std::thread::JoinHandle;

use crate::client_prefs::ClientPrefs;

/// The sections `gmClientUI::Apply` applies, in its order: graphics, sound, FX, camera, then the
/// three avatar and misc sections.
pub const SECTIONS: [u32; 7] = [
    0x4000_0004,
    0x4000_0007,
    0x4000_0009,
    0x4000_0008,
    0x4000_000D,
    0x4000_000E,
    0x4000_000C,
];
/// The graphics section, whose change asks for the render verification.
const RENDER_SECTION: u32 = 0x4000_0004;
/// The display section: submitted to its handlers (the device), not applied by the page.
const DEVICE_SECTION: u32 = 0x4000_000B;

/// What an apply did: the keys it applied, whether the render verification is asked, and the save
/// it started (none while the verification is asked).
#[derive(Debug)]
pub struct ApplyOutcome {
    pub keys: Vec<u32>,
    pub verify_render: bool,
    pub save: Option<JoinHandle<io::Result<()>>>,
}

impl ApplyOutcome {
    /// Whether a sound setting was applied (the caller pushes the sound settings to the manager).
    pub fn applied_sound(&self) -> bool {
        self.keys
            .iter()
            .any(|&key| ac2_audio::prefs::SOUND_KEYS.contains(&key))
    }

    /// Waits for the save (if one was started) and returns its result.
    pub fn wait_saved(self) -> io::Result<()> {
        match self.save {
            Some(handle) => handle
                .join()
                .unwrap_or_else(|_| Err(io::Error::other("the save thread panicked"))),
            None => Ok(()),
        }
    }
}

/// The close request's answer: the page closes, or the player is asked about the changes first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Close {
    Closed,
    Confirm,
}

/// `gmOptionUI`'s confirmation state: `MakeConfirmDialog` while a question is open.
#[derive(Debug, Default)]
pub struct OptionsWindow {
    confirming: bool,
}

impl OptionsWindow {
    /// `gmOptionUI::Show(0)`: closing with changes asks first (`MakeConfirmDialog`).
    pub fn request_close(&mut self, prefs: &ClientPrefs) -> Close {
        if prefs.options_changed() {
            self.confirming = true;
            Close::Confirm
        } else {
            Close::Closed
        }
    }

    /// Whether the confirmation is open.
    pub fn confirming(&self) -> bool {
        self.confirming
    }

    /// `gmOptionUI::DoConfirm(confirm)`: yes applies the page (`gmClientUI::Apply`), no recalls it.
    /// `None` for no (nothing is applied).
    pub fn answer(&mut self, prefs: &mut ClientPrefs, yes: bool) -> Option<ApplyOutcome> {
        self.confirming = false;
        if yes {
            Some(prefs.options_apply())
        } else {
            prefs.options_recall();
            None
        }
    }
}

impl ClientPrefs {
    /// `gmClientUI::Changed`: a value differs from the one last applied, in one of the sections.
    pub fn options_changed(&self) -> bool {
        SECTIONS
            .iter()
            .any(|&section| !self.bank().submit(section).is_empty())
    }

    /// `gmClientUI::Apply`: the render section's change is noted, the device is submitted, then
    /// every section is applied in [`SECTIONS`] order. The file is saved on a worker thread unless
    /// the render section changed.
    pub fn options_apply(&mut self) -> ApplyOutcome {
        let verify_render = !self.bank().submit(RENDER_SECTION).is_empty();
        // `PreferenceBank::Submit(0x4000000B)`: the device's handlers are told of its changes;
        // this port's device has no handler, so the keys are only read.
        let _ = self.bank().submit(DEVICE_SECTION);
        let mut keys = Vec::new();
        for section in SECTIONS {
            keys.extend(self.bank_mut().apply(section));
        }
        let save = if verify_render {
            None
        } else {
            Some(self.save())
        };
        ApplyOutcome {
            keys,
            verify_render,
            save,
        }
    }

    /// `gmClientUI::Recall`: every section goes back to its applied values (nothing is written).
    pub fn options_recall(&mut self) {
        for section in SECTIONS {
            self.bank_mut().recall(section);
        }
    }

    /// `gmClientUI::Default`: every section's values become its defaults, then the page is applied.
    pub fn options_default(&mut self) -> ApplyOutcome {
        for section in SECTIONS {
            self.bank_mut().reset_section(section);
        }
        self.options_apply()
    }
}
