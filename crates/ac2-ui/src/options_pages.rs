//! The client options page's panel logic (`gmClientUI` in the option panel): the page's widgets
//! are made once the HUD's tree, its toolbar layout and the preferences are there, its messages
//! and the confirmations' answers reach it, and its requests run on the preferences.
//!
//! The applied values the client reads live (sound, camera, scene) are the host's
//! ([`OptionsHost::apply_live`]). What the applied keys change in the world goes out as
//! [`OptionsEffect`]s: the scene's options for the view, and the display section's change, which
//! the window's owner answers by reading the display values again (only a verified display change
//! sets them).
//!
//! Apply saves the file on a worker thread. A render (display section) change asks the player to
//! verify it first (a timed dialog); nothing is saved or applied to the window until that answer is
//! yes, and a timeout answers no (`TimedDialog::ListenToMessage`).

use std::io;
use std::sync::Arc;
use std::thread::JoinHandle;

use ac2_render::prefs::SceneOptions;

use crate::{
    pref_label, ClientOptionsUi, ClientRequest, DialogFactory, DialogOutcome, LayoutDesc,
    OptionsPrefs, UiMessage, UiTree,
};

/// The display keys whose change the window and the surface take: FullScreen, TripleBuffering,
/// SyncToDisplayRefresh, OptimalRefreshRate and Antialiasing (`Device::ChangePresentation`).
pub const DISPLAY_KEYS: [u32; 5] = [
    ac2_render::prefs::FULL_SCREEN,
    0x4000_0005,
    0x4000_0006,
    0x4000_0007,
    0x4000_0060,
];

/// A change the applied options make in the world, for the frame's owner to make.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionsEffect {
    /// The scene's graphics options (`WorldScene::set_options`), when they changed.
    Scene(SceneOptions),
    /// A display key was applied: the window and surface take the display section's values
    /// (read them from the bank, `ac2_client`'s `WindowPresentation::from_bank`).
    Display,
}

/// The client's side of the options page: the preferences the page reads and writes, and the live
/// values the applies change. The client implements it over its own borrow for each call.
pub trait OptionsHost {
    /// Runs `f` on the preferences, when the client has them (`None` before they are loaded).
    fn with_prefs(&mut self, f: &mut dyn FnMut(&mut dyn OptionsPrefs)) -> Option<()>;

    /// The applied preferences as the live client reads them (the sound manager's and the camera's
    /// values). Returns the scene's graphics options for the view.
    fn apply_live(&mut self) -> Option<SceneOptions>;
}

/// The client options page and the effects its applies left.
#[derive(Debug, Default)]
pub struct OptionsPages {
    page: Option<ClientOptionsUi>,
    /// The UI string table's DID, which the page's labels come from.
    ui_table: Option<u32>,
    /// The preferences' label table (enum 0x40000008), for the page's rows.
    pref_table: Option<u32>,
    /// The scene options last applied, so a change is only sent when it differs.
    scene: Option<SceneOptions>,
    effects: Vec<OptionsEffect>,
    /// The saves started on worker threads; reaped (and their errors reported) as they finish.
    saves: Vec<JoinHandle<io::Result<()>>>,
}

impl OptionsPages {
    pub fn set_tables(&mut self, ui: Option<u32>, prefs: Option<u32>) {
        self.ui_table = ui;
        self.pref_table = prefs;
    }

    /// `gmClientUI::gmClientUI`: the page's widgets, made once the HUD's tree, its toolbar layout
    /// and the preferences are there.
    pub fn make(
        &mut self,
        host: &mut dyn OptionsHost,
        tree: &mut UiTree,
        toolbar: Option<&Arc<LayoutDesc>>,
    ) {
        if self.page.is_some() {
            return;
        }
        let (Some(toolbar), Some(ui), Some(pref_table), Some(text)) =
            (toolbar, self.ui_table, self.pref_table, tree.text_data())
        else {
            return;
        };
        let page = &mut self.page;
        host.with_prefs(&mut |prefs| {
            *page = ClientOptionsUi::create(
                tree,
                toolbar,
                &*text,
                (ui, pref_table),
                prefs.bank(),
                prefs.name_table(),
                pref_label,
            );
        });
    }

    /// The page's share of a UI message (its tab, widgets and buttons).
    pub fn message(
        &mut self,
        host: &mut dyn OptionsHost,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        message: &UiMessage,
    ) {
        let Some(page) = self.page.as_mut() else {
            return;
        };
        host.with_prefs(&mut |prefs| page.message(tree, dialogs, prefs.bank_mut(), message));
    }

    /// `gmClientUI::Changed` over the preferences (false before they are loaded).
    pub fn changed(&self, host: &mut dyn OptionsHost) -> bool {
        let mut changed = false;
        host.with_prefs(&mut |prefs| changed = prefs.options_changed());
        changed
    }

    /// `gmOptionUI::DoConfirm` on the client page.
    pub fn confirm_close(&mut self, yes: bool) {
        if let Some(page) = self.page.as_mut() {
            page.confirm_close(yes);
        }
    }

    /// The confirmations' answers (the page's questions among them).
    pub fn outcomes(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        outcomes: &[DialogOutcome],
    ) {
        if let Some(page) = self.page.as_mut() {
            page.dialog_outcomes(tree, dialogs, outcomes);
        }
    }

    /// Runs the page's requests on the preferences, then shows the values again. The applied
    /// values go live (sound, camera, scene, and the window for a verified display change).
    pub fn run(
        &mut self,
        host: &mut dyn OptionsHost,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
    ) {
        reap_saves(&mut self.saves);
        // `UIElementManager::Init` / `HandleApply`: the tooltip keys as the bank holds them, taken
        // when they change (at startup and after an apply).
        let mut tooltip = None;
        host.with_prefs(&mut |prefs| tooltip = Some(crate::tooltip_prefs::values(prefs.bank())));
        if let Some((delay, enabled)) = tooltip {
            tree.apply_tooltip_prefs(delay, enabled);
        }
        let Some(page) = self.page.as_mut() else {
            return;
        };
        for request in page.take_requests() {
            let mut answer = None;
            let ran = host.with_prefs(&mut |prefs| {
                answer = Some(run_request(prefs, &request));
            });
            let (Some(()), Some((keys, verify, saves))) = (ran, answer) else {
                return;
            };
            self.saves.extend(saves);
            if !keys.is_empty() {
                let scene = host.apply_live();
                if let Some(scene) = scene {
                    self.effects
                        .extend(live_effects(&keys, scene, &mut self.scene));
                }
            }
            let page = &mut *page;
            let shown = host.with_prefs(&mut |prefs| {
                if verify {
                    page.after_apply(tree, dialogs, prefs.bank(), true);
                }
                page.refresh(tree, prefs.bank());
            });
            if shown.is_none() {
                return;
            }
        }
        host.with_prefs(&mut |prefs| page.refresh(tree, prefs.bank()));
    }

    /// The effects the applies left since the last call.
    pub fn take_effects(&mut self) -> Vec<OptionsEffect> {
        std::mem::take(&mut self.effects)
    }
}

/// Runs one page request on the preferences: the applied keys, whether the render verification
/// is asked, and the saves it started.
fn run_request(
    prefs: &mut dyn OptionsPrefs,
    request: &ClientRequest,
) -> (Vec<u32>, bool, Vec<JoinHandle<io::Result<()>>>) {
    match request {
        ClientRequest::Apply => {
            let outcome = prefs.options_apply();
            let verify = outcome.verify_render;
            (outcome.keys, verify, outcome.save.into_iter().collect())
        }
        ClientRequest::Answer(actions) => {
            let outcome = prefs.options_page_answer(actions);
            (outcome.keys, outcome.verify_render, outcome.saves)
        }
    }
}

/// Saves that finished: an error is reported, a running save waits for the next frame.
fn reap_saves(saves: &mut Vec<JoinHandle<io::Result<()>>>) {
    let (done, running): (Vec<_>, Vec<_>) = std::mem::take(saves)
        .into_iter()
        .partition(JoinHandle::is_finished);
    *saves = running;
    for handle in done {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("ac2: the preferences were not saved: {error}"),
            Err(_) => eprintln!("ac2: the preferences save thread panicked"),
        }
    }
}

/// What the applied keys change in the world: the scene's options when they differ from the last
/// ones sent, and the display change, only when a display key itself was applied (a display
/// change the player verified).
pub fn live_effects(
    keys: &[u32],
    scene_now: SceneOptions,
    last_scene: &mut Option<SceneOptions>,
) -> Vec<OptionsEffect> {
    let mut effects = Vec::new();
    if *last_scene != Some(scene_now) {
        *last_scene = Some(scene_now);
        effects.push(OptionsEffect::Scene(scene_now));
    }
    if keys.iter().any(|k| DISPLAY_KEYS.contains(k)) {
        effects.push(OptionsEffect::Display);
    }
    effects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(far_radius: i32) -> SceneOptions {
        SceneOptions {
            far_radius,
            ..SceneOptions::default()
        }
    }

    #[test]
    fn a_graphics_change_sends_the_scene_once_and_leaves_the_window_alone() {
        let mut last = None;
        let first = live_effects(&[0x4000_0043], scene(6), &mut last);
        assert_eq!(first, vec![OptionsEffect::Scene(scene(6))]);
        let again = live_effects(&[0x4000_0043], scene(6), &mut last);
        assert!(again.is_empty(), "the same scene is not sent twice");
    }

    #[test]
    fn the_display_takes_effect_only_when_a_display_key_is_applied() {
        let mut last = Some(scene(4));
        // Not applied (a display change still waiting for its verification): nothing for the window.
        assert!(live_effects(&[0x4000_0044], scene(4), &mut last).is_empty());
        // Applied after the player verified it: the window and surface follow the values.
        assert_eq!(
            live_effects(&[ac2_render::prefs::FULL_SCREEN], scene(4), &mut last),
            vec![OptionsEffect::Display]
        );
        // The other display keys (sync, triple buffering) reach the surface the same way.
        assert_eq!(
            live_effects(&[0x4000_0005], scene(4), &mut last),
            vec![OptionsEffect::Display]
        );
    }
}
