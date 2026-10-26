//! The pregame screens' 3D scenes (`window` feature) over [`UiScenes`]: what each screen stands
//! in its viewport ([`content`]), avatar selection's mouse-over modes and selected animation,
//! and avatar modification's camera ([`modification`]). Picks answer what the figure under the
//! pointer stands for (a character slot, an avatar).

mod content;
mod modification;

use std::sync::Arc;

use ac2_anim::Ac2BehaviorParams;
use ac2_db::ResourceStore;
use ac2_gpu::Gpu;
use ac2_ui::pregame::PregameUi;
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

use crate::ui_scene::{Figure, UiScenes, ViewSetup};
use content::MODE_AVATAR_SELECTION;

/// `gmAvatarSelectionUI`'s selected animation (`BehaviorParams::mBehaviorID`).
const BEHAVIOR_SELECTED: u32 = 0x4100_0005;
/// The mode an avatar under the pointer goes to (`MouseOnObject`).
const MODE_PRESENTED: u32 = 0x4100_0007;

/// The pregame viewports' scenes.
pub struct PregameScenes {
    scenes: UiScenes,
    /// The avatar whose selected animation runs (`_waitForAnim`), by figure key.
    selected: Option<u64>,
    /// The avatar under the pointer (`_objectUnderMouse`), by figure key.
    hovered: Option<u64>,
}

impl PregameScenes {
    /// Scenes over `store`'s DATs; each viewport's models load through a `WorldScene` over
    /// `world`.
    pub fn new(store: Arc<ResourceStore>, world: Arc<Ac2WorldSource>) -> Self {
        Self {
            scenes: UiScenes::new(store, world),
            selected: None,
            hovered: None,
        }
    }

    /// The scenes underneath.
    pub fn scenes(&self) -> &UiScenes {
        &self.scenes
    }

    /// Follows the UI at `now` seconds: the shown viewports, what stands in their scenes, the
    /// loads that finished (all of them when `wait`), the animation and the frame's layout.
    pub fn update(&mut self, gpu: &Gpu, ui: &PregameUi, now: f64, wait: bool) {
        let mut figures = |viewport: &SceneViewport, scene: &crate::ui_scene::SceneData| {
            content::figures(ui, viewport, scene)
        };
        let mut direct = |setup: &mut ViewSetup<'_>| modification::direct(ui, setup);
        self.scenes
            .update(gpu, ui.viewports(), now, wait, &mut figures, &mut direct);
        self.hover(ui);
    }

    /// The avatar figures of avatar selection.
    fn avatar(&self, ui: &PregameUi, avatar: usize) -> Option<Figure> {
        let selection = ui.avatar_selection()?;
        let root = selection.viewport()?;
        let viewport = ui.viewports().into_iter().find(|v| v.root == root)?;
        let scene = self.scenes.scene(viewport.scene)?;
        content::figures(ui, &viewport, scene)
            .into_iter()
            .find(|f| f.pick == Some(avatar) && f.mode == Some(MODE_AVATAR_SELECTION))
    }

    /// `gmAvatarSelectionUI::MouseOffObject` / `MouseOnObject` when the avatar under the
    /// pointer changes: the one left goes back to the line-up mode, the one under the pointer
    /// to the presented mode 0x41000007 (where its selected animation plays), each unless
    /// already there.
    fn hover(&mut self, ui: &PregameUi) {
        let under = ui.avatar_selection().and_then(|s| s.under_mouse());
        let key = under.and_then(|i| self.avatar(ui, i)).map(|f| f.key);
        if key == self.hovered {
            return;
        }
        let modes = [(self.hovered, MODE_AVATAR_SELECTION), (key, MODE_PRESENTED)];
        for (key, mode) in modes {
            if let Some(body) = key.and_then(|k| self.scenes.body_mut(k)) {
                if body.mode() != mode {
                    body.do_mode(0, mode);
                }
            }
        }
        self.hovered = key;
    }

    /// `UIScene::StartSelection` / `FinishSelection` at screen `point` in the current screen's
    /// viewport: what the nearest figure under it stands for.
    pub fn pick(&self, ui: &PregameUi, point: (i32, i32)) -> Option<usize> {
        let root = ui
            .character_selection()
            .and_then(|s| s.viewport())
            .or_else(|| ui.avatar_selection().and_then(|s| s.viewport()));
        self.scenes.pick(point, Some(root?))?.pick
    }

    /// Avatar selection's wait for the chosen avatar's selected animation
    /// (`gmAvatarSelectionUI::ListenToMessage`: `DoBehavior(0x41000005)`, then
    /// `IsDoingBehavior` each loop): whether it ended.
    pub fn animation_done(&mut self, ui: &PregameUi) -> bool {
        let Some(avatar) = ui.selected_animation() else {
            self.selected = None;
            return false;
        };
        let Some(figure) = self.avatar(ui, avatar) else {
            return true;
        };
        match self.selected {
            Some(key) if key == figure.key => self
                .scenes
                .body_mut(figure.key)
                .is_none_or(|b| !b.is_doing_behavior(BEHAVIOR_SELECTED)),
            _ => {
                if !self.scenes.object_known(figure.object) {
                    // Still loading: the avatar is not made yet.
                    return false;
                }
                self.selected = Some(figure.key);
                match self.scenes.body_mut(figure.key) {
                    Some(body) => {
                        body.do_behavior(&Ac2BehaviorParams::new(BEHAVIOR_SELECTED));
                        false
                    }
                    None => true,
                }
            }
        }
    }

    /// Whether loads are under way (headless snapshots wait them out).
    pub fn busy(&self) -> bool {
        self.scenes.busy()
    }

    /// Writes each view's frame (`time` is client time for animated materials).
    pub fn prepare(&mut self, gpu: &Gpu, time: f64) {
        self.scenes.prepare(gpu, time);
    }

    /// How many objects of the views have their models loaded.
    pub fn drawn_objects(&self) -> usize {
        self.scenes.drawn_objects()
    }
}
