//! The gameplay HUD over the world (`window` feature): the element tree
//! `gmGamePlayUI::SetupChildren`'s frameworks create, built and its images read on a worker
//! thread, then composed (`UIRegion::DrawHere`, software, as retail) whenever it arrives or the
//! window changes size, and drawn as one layer per root surface. The frame's UI actions go to
//! the tree (`UIElementManager::StartInput`), and it is composed again when they change it.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use ac2_db::{ResourceResult, ResourceStore};
use ac2_gpu::{Gpu, UiLayer, UiOverlay};
use ac2_ui::{GameplayHud, Reticle, ReticleOutcome, ReticleTarget, UiMessage, UiPixels, UiTree};

type Built = ResourceResult<(GameplayHud, HashMap<u32, UiPixels>)>;

pub(crate) struct HudView {
    pending: Option<Receiver<Built>>,
    hud: Option<(UiTree, HashMap<u32, UiPixels>)>,
    reticle: Option<Reticle>,
    overlay: UiOverlay,
    size: (u32, u32),
    dirty: bool,
}

impl HudView {
    pub(crate) fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let (send, receive) = mpsc::channel();
        let (width, height) = (size.0 as i32, size.1 as i32);
        std::thread::Builder::new()
            .name("ac2 hud".into())
            .spawn(move || {
                let built = ac2_ui::build_gameplay_hud(&store, width, height).and_then(|hud| {
                    let images = ac2_ui::load_ui_images(&store, &hud.tree)?;
                    Ok((hud, images))
                });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            reticle: None,
            overlay: UiOverlay::new(gpu, format),
            size,
            dirty: false,
        }
    }

    /// The render device's new size: every root placed again (`UIElement::UpdateSizeAndPosition`).
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        if let Some((tree, _)) = &mut self.hud {
            tree.set_display(self.size.0 as i32, self.size.1 as i32);
        }
        self.dirty = true;
    }

    /// The frame's UI actions (`IUIManager` start / stop: `UIElementManager::StartInput` /
    /// `StopInput`); returns the messages they broadcast.
    pub(crate) fn input(&mut self, actions: &[(u32, f32, bool)]) -> Vec<UiMessage> {
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        for &(action, extent, start) in actions {
            if start {
                tree.start_input(action, extent);
            } else {
                tree.stop_input(action);
            }
        }
        if tree.take_changed() {
            self.dirty = true;
        }
        tree.take_messages()
    }

    /// The active text element's actions and characters (`UIElement_Text::ActionHandler` /
    /// `CharacterHandler`); returns the messages they broadcast.
    pub(crate) fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        ctrl: bool,
    ) -> Vec<UiMessage> {
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        for &(action, extent) in actions {
            tree.text_action(action, extent, ctrl);
        }
        for &(ch, key) in characters {
            tree.character(ch, key, ctrl);
        }
        if tree.take_changed() {
            self.dirty = true;
        }
        tree.take_messages()
    }

    /// Whether an editable text element is active (the input manager's text mode).
    pub(crate) fn text_mode(&self) -> bool {
        self.hud.as_ref().is_some_and(|(tree, _)| tree.text_mode())
    }

    fn arrived(&mut self, built: Built) {
        match built {
            Ok((
                GameplayHud {
                    mut tree, reticle, ..
                },
                images,
            )) => {
                tree.set_display(self.size.0 as i32, self.size.1 as i32);
                self.hud = Some((tree, images));
                self.reticle = reticle;
                self.dirty = true;
            }
            Err(error) => eprintln!("ac2: the HUD did not load: {error}"),
        }
    }

    /// Takes a finished build, then recomposes if anything changed.
    pub(crate) fn update(&mut self, gpu: &Gpu) {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(built) => {
                    self.pending = None;
                    self.arrived(built);
                }
                Err(TryRecvError::Disconnected) => self.pending = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        self.compose(gpu);
    }

    /// Waits for the build (headless snapshots).
    pub(crate) fn settle(&mut self, gpu: &Gpu) {
        if let Some(pending) = self.pending.take() {
            if let Ok(built) = pending.recv() {
                self.arrived(built);
            }
        }
        self.compose(gpu);
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it
    /// is drawn, its box and projection. Then the HUD is composed again or its surfaces moved.
    pub(crate) fn reticle(
        &mut self,
        gpu: &Gpu,
        selection: Option<(u64, Option<&str>)>,
        target: Option<ReticleTarget<'_>>,
    ) -> ReticleOutcome {
        let (Some((tree, _)), Some(reticle)) = (&mut self.hud, &mut self.reticle) else {
            return ReticleOutcome::default();
        };
        let id = selection.map(|(id, _)| id);
        if reticle.current() != id {
            reticle.selection_changed(tree, id, selection.and_then(|(_, name)| name));
        }
        let outcome = reticle.update(tree, target, false);
        if tree.take_changed() {
            self.dirty = true;
        }
        self.compose(gpu);
        outcome
    }

    fn compose(&mut self, gpu: &Gpu) {
        let Some((tree, images)) = &mut self.hud else {
            return;
        };
        if !self.dirty {
            if tree.take_moved() {
                let placements = tree.placements();
                self.overlay
                    .set_placements(placements.into_iter().map(|(_, x, y, r, s)| (x, y, r, s)));
            }
            return;
        }
        tree.take_moved();
        self.dirty = false;
        tree.take_changed();
        let surfaces = tree.compose(images);
        self.overlay.set_layers(
            gpu,
            surfaces.iter().map(|surface| UiLayer {
                x: surface.x,
                y: surface.y,
                width: surface.width,
                height: surface.height,
                rotation: surface.rotation,
                shown: surface.shown,
                rgba: &surface.rgba,
            }),
        );
    }

    /// The pointer as the UI last saw it.
    pub(crate) fn pointer(&self) -> (i32, i32) {
        self.hud.as_ref().map_or((0, 0), |(tree, _)| tree.pointer())
    }

    pub(crate) fn shown_layers(&self) -> usize {
        self.overlay.shown_layers()
    }

    pub(crate) fn layers(&self) -> usize {
        self.overlay.layers()
    }

    pub(crate) fn prepare(&self, queue: &wgpu::Queue) {
        self.overlay.prepare(queue, self.size.0, self.size.1);
    }

    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.overlay.draw(pass);
    }
}
