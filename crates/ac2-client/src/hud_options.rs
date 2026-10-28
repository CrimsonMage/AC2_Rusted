//! The client options page's wiring (`gmClientUI` in the option panel): the page's widgets are made
//! once the HUD and the preferences are there, its messages and the confirmations' answers reach
//! it, and its requests run on the client's preferences ([`crate::ClientPrefs`]). What the applied
//! values change in the world goes out as [`OptionsEffect`]s: the scene's draw and frill distances
//! for the view, and the window's fullscreen, which only a verified display change sets. The sound
//! manager's volumes and the camera's values are applied to the client directly.
//!
//! Apply saves the file on a worker thread. A render (display section) change asks the player to
//! verify it first (a timed dialog); nothing is saved or applied to the window until that answer is
//! yes, and a timeout answers no (`TimedDialog::ListenToMessage`).

use std::io;
use std::sync::Arc;
use std::thread::JoinHandle;

use ac2_render::prefs::SceneOptions;
use ac2_ui::{
    ClientOptionsUi, ClientRequest, DialogFactory, DialogOutcome, LayoutDesc, OptionsPrefs,
    UiMessage, UiTree,
};

use crate::client::Client;
use crate::window_presentation::{WindowPresentation, DISPLAY_KEYS};

/// A change the applied options make in the world, for the frame's owner to make.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionsEffect {
    /// The scene's graphics options (`WorldScene::set_options`), when they changed.
    Scene(SceneOptions),
    /// The window's fullscreen state and the surface's presentation (only a verified display
    /// change asks for it).
    Presentation(WindowPresentation),
}

/// The client options page and the effects its applies left.
#[derive(Debug, Default)]
pub(crate) struct OptionsPages {
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
    pub(crate) fn set_tables(&mut self, ui: Option<u32>, prefs: Option<u32>) {
        self.ui_table = ui;
        self.pref_table = prefs;
    }

    /// `gmClientUI::gmClientUI`: the page's widgets, made once the HUD's tree, its toolbar layout
    /// and the preferences are there.
    pub(crate) fn make(
        &mut self,
        client: &Client,
        tree: &mut UiTree,
        toolbar: Option<&Arc<LayoutDesc>>,
    ) {
        if self.page.is_some() {
            return;
        }
        let (Some(toolbar), Some(ui), Some(pref_table), Some(text), Some(prefs)) = (
            toolbar,
            self.ui_table,
            self.pref_table,
            tree.text_data(),
            client.prefs(),
        ) else {
            return;
        };
        self.page = ClientOptionsUi::create(
            tree,
            toolbar,
            &*text,
            (ui, pref_table),
            prefs.bank(),
            prefs.names(),
            ac2_ui::pref_label,
        );
    }

    /// The page's share of a UI message (its tab, widgets and buttons).
    pub(crate) fn message(
        &mut self,
        client: &mut Client,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        message: &UiMessage,
    ) {
        let (Some(page), Some(prefs)) = (self.page.as_mut(), client.prefs_mut()) else {
            return;
        };
        page.message(tree, dialogs, prefs.bank_mut(), message);
    }

    /// The confirmations' answers (the page's questions among them).
    pub(crate) fn outcomes(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        outcomes: &[DialogOutcome],
    ) {
        if let Some(page) = self.page.as_mut() {
            page.dialog_outcomes(tree, dialogs, outcomes);
        }
    }

    /// Runs the page's requests on the client's preferences, then shows the values again. The
    /// applied values go live (sound, camera, scene, and the window for a verified display change).
    pub(crate) fn run(
        &mut self,
        client: &mut Client,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
    ) {
        self.reap_saves();
        let Some(page) = self.page.as_mut() else {
            return;
        };
        for request in page.take_requests() {
            let Some(prefs) = client.prefs_mut() else {
                return;
            };
            let (keys, verify) = match request {
                ClientRequest::Apply => {
                    let outcome = prefs.options_apply();
                    let verify = outcome.verify_render;
                    self.saves.extend(outcome.save);
                    (outcome.keys, verify)
                }
                ClientRequest::Answer(actions) => {
                    let outcome = prefs.options_page_answer(&actions);
                    self.saves.extend(outcome.saves);
                    (outcome.keys, outcome.verify_render)
                }
            };
            if !keys.is_empty() {
                apply_live(client, &keys, &mut self.scene, &mut self.effects);
            }
            let Some(prefs) = client.prefs() else {
                return;
            };
            if verify {
                page.after_apply(tree, dialogs, prefs.bank(), true);
            }
            page.refresh(tree, prefs.bank());
        }
        if let Some(prefs) = client.prefs() {
            page.refresh(tree, prefs.bank());
        }
    }

    /// The effects the applies left since the last call.
    pub(crate) fn take_effects(&mut self) -> Vec<OptionsEffect> {
        std::mem::take(&mut self.effects)
    }

    /// Saves that finished: an error is reported, a running save waits for the next frame.
    fn reap_saves(&mut self) {
        let (done, running): (Vec<_>, Vec<_>) = std::mem::take(&mut self.saves)
            .into_iter()
            .partition(JoinHandle::is_finished);
        self.saves = running;
        for handle in done {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => eprintln!("ac2: the preferences were not saved: {error}"),
                Err(_) => eprintln!("ac2: the preferences save thread panicked"),
            }
        }
    }
}

/// The applied values the client reads live (`apply_live_preferences`), and the effects of the
/// scene and the window for the view and the window's owner.
fn apply_live(
    client: &mut Client,
    keys: &[u32],
    scene: &mut Option<SceneOptions>,
    effects: &mut Vec<OptionsEffect>,
) {
    let Some(options) = client.apply_live_preferences() else {
        return;
    };
    let presentation = client
        .prefs()
        .map(|prefs| WindowPresentation::from_bank(prefs.bank()));
    effects.extend(live_effects(keys, options, presentation, scene));
}

/// What the applied keys change in the world: the scene's options when they differ from the last
/// ones sent, and the window's presentation, only when a display key itself was applied (a display
/// change the player verified). `presentation` is the display section's values.
pub(crate) fn live_effects(
    keys: &[u32],
    scene_now: SceneOptions,
    presentation: Option<WindowPresentation>,
    last_scene: &mut Option<SceneOptions>,
) -> Vec<OptionsEffect> {
    let mut effects = Vec::new();
    if *last_scene != Some(scene_now) {
        *last_scene = Some(scene_now);
        effects.push(OptionsEffect::Scene(scene_now));
    }
    if let (true, Some(now)) = (keys.iter().any(|k| DISPLAY_KEYS.contains(k)), presentation) {
        effects.push(OptionsEffect::Presentation(now));
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
        let first = live_effects(&[0x4000_0043], scene(6), None, &mut last);
        assert_eq!(first, vec![OptionsEffect::Scene(scene(6))]);
        let again = live_effects(&[0x4000_0043], scene(6), None, &mut last);
        assert!(again.is_empty(), "the same scene is not sent twice");
    }

    const FULL_SCREEN: u32 = crate::window_presentation::FULL_SCREEN;

    fn window(fullscreen: bool) -> WindowPresentation {
        WindowPresentation::from_flags(fullscreen, false, true)
    }

    #[test]
    fn the_window_takes_the_display_only_when_a_display_key_is_applied() {
        let mut last = Some(scene(4));
        // Not applied (a display change still waiting for its verification): no window change.
        assert!(live_effects(&[0x4000_0044], scene(4), Some(window(true)), &mut last).is_empty());
        // Applied after the player verified it: the window and surface follow the values.
        assert_eq!(
            live_effects(&[FULL_SCREEN], scene(4), Some(window(true)), &mut last),
            vec![OptionsEffect::Presentation(window(true))]
        );
        assert_eq!(
            live_effects(&[FULL_SCREEN], scene(4), Some(window(false)), &mut last),
            vec![OptionsEffect::Presentation(window(false))]
        );
        // The other display keys (sync, triple buffering) reach the surface the same way.
        let triple = WindowPresentation::from_flags(false, true, false);
        assert_eq!(
            live_effects(&[0x4000_0005], scene(4), Some(triple), &mut last),
            vec![OptionsEffect::Presentation(triple)]
        );
    }

    #[test]
    fn a_missing_display_setting_changes_nothing_in_the_window() {
        let mut last = Some(scene(4));
        assert!(live_effects(&[FULL_SCREEN], scene(4), None, &mut last).is_empty());
    }
}
