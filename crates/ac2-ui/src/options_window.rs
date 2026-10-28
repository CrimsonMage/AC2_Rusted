//! The options page's model: `gmClientUI` (the client's preference sections and their Apply,
//! Recall, Default and Changed) and `gmOptionUI`'s close and confirmation (`Show`,
//! `MakeConfirmDialog`, `DoConfirm`). The page's widgets are not modelled: its OK button runs
//! [`OptionsPrefs::options_apply`], its Cancel runs [`OptionsPrefs::options_recall`], its Default
//! runs [`OptionsPrefs::options_default`], and closing with changes asks [`OptionsWindow`].
//!
//! Apply saves the file on a worker thread ([`OptionsPrefs::save`]), except when the render section
//! changed: retail then asks the player to verify the render options (`VerifyRenderOptions`) and
//! does not save. That question is not modelled, so such an apply leaves the file as it was.
//!
//! The preferences are `ac2_prefs::PrefsFile`; the sound manager's and the view's reactions to the
//! applied keys are the client's (it reads [`ApplyOutcome::keys`] and [`PageOutcome::keys`]).

use std::io;
use std::thread::JoinHandle;

use ac2_prefs::{NameTable, PreferenceBank, PrefsFile};

use crate::client_page::PageAction;

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
    pub fn request_close(&mut self, prefs: &impl OptionsPrefs) -> Close {
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
    pub fn answer(&mut self, prefs: &mut impl OptionsPrefs, yes: bool) -> Option<ApplyOutcome> {
        self.confirming = false;
        if yes {
            Some(prefs.options_apply())
        } else {
            prefs.options_recall();
            None
        }
    }
}

pub trait OptionsPrefs {
    /// The preference bank the page's values are read from and written to.
    fn bank(&self) -> &PreferenceBank;

    /// The bank, for the page's changes and the applies.
    fn bank_mut(&mut self) -> &mut PreferenceBank;

    /// Saves the file on a worker thread (`PreferenceBank::Save`).
    fn save(&mut self) -> JoinHandle<io::Result<()>>;

    /// `gmClientUI::Changed`: a value differs from the one last applied, in one of the sections.
    fn options_changed(&self) -> bool {
        std::iter::once(DISPLAY_SECTION)
            .chain(SECTIONS)
            .any(|section| !self.bank().submit(section).is_empty())
    }

    /// `gmClientUI::Apply`: the display section's change is noted (it is submitted to the device,
    /// which this port has no handler for), then every other section is applied in [`SECTIONS`]
    /// order. The file is saved on a worker thread unless the display section changed.
    fn options_apply(&mut self) -> ApplyOutcome {
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
    fn options_recall(&mut self) {
        for section in SECTIONS {
            self.bank_mut().recall(section);
        }
    }

    /// `gmClientUI::Default`: every section's values become its defaults, then the page is applied.
    fn options_default(&mut self) -> ApplyOutcome {
        for section in SECTIONS {
            self.bank_mut().reset_section(section);
        }
        self.options_apply()
    }

    /// The options page's answers, in order (`gmClientUI::RecvNotice_CloseConfirmationDialog`, see
    /// [`ac2_ui::ClientPage::recv_close_confirmation`]). A display recall or synchronize applies
    /// the display section (`PreferenceBank::Recall` / `Synchronize` of `0x4000000B`) and saves;
    /// Recall and Default run as the page's own. Ask the page to verify when `verify_render` is set.
    fn options_page_answer(&mut self, actions: &[PageAction]) -> PageOutcome {
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

impl<N: NameTable> OptionsPrefs for PrefsFile<N> {
    fn bank(&self) -> &PreferenceBank {
        PrefsFile::bank(self)
    }

    fn bank_mut(&mut self) -> &mut PreferenceBank {
        PrefsFile::bank_mut(self)
    }

    fn save(&mut self) -> JoinHandle<io::Result<()>> {
        PrefsFile::save(self)
    }
}
