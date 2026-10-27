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

use ac2_ui::PageAction;

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
/// The display (Rendering) section: `gmClientUI::Apply`'s `_indexRendering` expander. Its change
/// asks for the render verification, and the section is submitted to the device's handlers, not
/// applied by the page (only a verified change applies it).
const DISPLAY_SECTION: u32 = 0x4000_000B;

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

/// What the options page's answers did: the keys applied, the saves started (on worker threads),
/// and whether a default that changed the graphics asks for the render verification.
#[derive(Debug, Default)]
pub struct PageOutcome {
    pub keys: Vec<u32>,
    pub saves: Vec<JoinHandle<io::Result<()>>>,
    pub verify_render: bool,
}

impl PageOutcome {
    /// Waits for every save and returns the first error.
    pub fn wait_saved(self) -> io::Result<()> {
        let mut result = Ok(());
        for handle in self.saves {
            let saved = handle
                .join()
                .unwrap_or_else(|_| Err(io::Error::other("the save thread panicked")));
            if result.is_ok() {
                result = saved;
            }
        }
        result
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
        std::iter::once(DISPLAY_SECTION)
            .chain(SECTIONS)
            .any(|section| !self.bank().submit(section).is_empty())
    }

    /// `gmClientUI::Apply`: the display section's change is noted (it is submitted to the device,
    /// which this port has no handler for), then every other section is applied in [`SECTIONS`]
    /// order. The file is saved on a worker thread unless the display section changed.
    pub fn options_apply(&mut self) -> ApplyOutcome {
        let verify_render = !self.bank().submit(DISPLAY_SECTION).is_empty();
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

    /// The options page's answers, in order (`gmClientUI::RecvNotice_CloseConfirmationDialog`, see
    /// [`ac2_ui::ClientPage::recv_close_confirmation`]). A display recall or synchronize applies
    /// the display section (`PreferenceBank::Recall` / `Synchronize` of `0x4000000B`) and saves;
    /// Recall and Default run as the page's own. Ask the page to verify when `verify_render` is set.
    pub fn options_page_answer(&mut self, actions: &[PageAction]) -> PageOutcome {
        let mut out = PageOutcome::default();
        for action in actions {
            match action {
                PageAction::RecallDisplay => {
                    self.bank_mut().recall(DISPLAY_SECTION);
                    out.saves.push(self.save());
                }
                PageAction::SynchronizeDisplay => {
                    out.keys.extend(self.bank_mut().apply(DISPLAY_SECTION));
                    out.saves.push(self.save());
                }
                PageAction::Recall => self.options_recall(),
                PageAction::Default => {
                    let applied = self.options_default();
                    out.keys.extend(applied.keys);
                    out.verify_render |= applied.verify_render;
                    out.saves.extend(applied.save);
                }
            }
        }
        out
    }
}
