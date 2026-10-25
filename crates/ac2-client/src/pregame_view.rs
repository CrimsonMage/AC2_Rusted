//! The pregame screens in a window (`window` feature): the pregame UI's data read on a worker
//! thread (`ac2_ui::pregame::load_pregame`), then the UI (`PregameUi`) fed the session's login
//! events and the frame's UI actions, composed in software whenever it changes and drawn as one
//! layer per root surface, like the gameplay HUD. Login events that arrive before the data are
//! kept and replayed.
//!
//! With a world source ([`PregameView::with_scenes`]) the viewports' 3D scenes
//! ([`PregameScenes`]) answer the UI's picks and its wait for the chosen avatar's animation, and
//! each is drawn into its viewport's rectangle in its root's place among the surfaces
//! (`UIViewportObject::Render`), its depth cleared first (`UIScene::Render`).

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use ac2_db::{ResourceResult, ResourceStore};
use ac2_gpu::{Gpu, UiLayer, UiOverlay};
use ac2_ui::pregame::{CharGenSystem, PregameAssets, PregameUi};
use ac2_ui::UiMessage;
use ac2_weenie::{OnlineSession, SessionEvent};
use ac2_world::Ac2WorldSource;

use crate::client::Notice;
use crate::pregame::{pregame_event, pregame_notice, pregame_requests, PregameOutcome};
use crate::pregame_scene::PregameScenes;

/// The pregame screens over the window.
pub struct PregameView {
    pending: Option<Receiver<ResourceResult<PregameAssets>>>,
    ui: Option<PregameUi>,
    /// Login events and notices that came before the UI.
    early: Vec<Result<SessionEvent, Notice>>,
    /// The `CharacterGenSystem` for the UI once it exists.
    system: Option<Box<dyn CharGenSystem>>,
    overlay: UiOverlay,
    /// The root of each uploaded layer, in drawing order.
    layer_roots: Vec<usize>,
    scenes: Option<PregameScenes>,
    store: Arc<ResourceStore>,
    format: wgpu::TextureFormat,
    size: (u32, u32),
    dirty: bool,
}

impl PregameView {
    /// Starts reading the pregame data on a worker thread.
    pub fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let (send, receive) = mpsc::channel();
        let data = Arc::clone(&store);
        std::thread::Builder::new()
            .name("ac2 pregame".into())
            .spawn(move || {
                let _ = send.send(ac2_ui::pregame::load_pregame(&data));
            })
            .expect("spawn the pregame loader");
        Self {
            pending: Some(receive),
            ui: None,
            early: Vec::new(),
            system: None,
            overlay: UiOverlay::new(gpu, format),
            layer_roots: Vec::new(),
            scenes: None,
            store,
            format,
            size,
            dirty: false,
        }
    }

    /// The viewports' 3D scenes, their models read through `world`.
    pub fn with_scenes(mut self, world: Arc<Ac2WorldSource>) -> Self {
        self.scenes = Some(PregameScenes::new(
            Arc::clone(&self.store),
            world,
            self.format,
        ));
        self
    }

    /// The `CharacterGenSystem` character generation asks (`PregameUi::set_char_gen_system`).
    pub fn set_char_gen_system(&mut self, system: Box<dyn CharGenSystem>) {
        match &mut self.ui {
            Some(ui) => ui.set_char_gen_system(system),
            None => self.system = Some(system),
        }
    }

    /// The UI, once its data arrived.
    pub fn ui(&self) -> Option<&PregameUi> {
        self.ui.as_ref()
    }

    /// The UI mode (0 while connecting or loading).
    pub fn mode(&self) -> u32 {
        self.ui.as_ref().map_or(0, PregameUi::mode)
    }

    /// A session event (`Characters`, character errors, creation answers, disconnection).
    pub fn event(&mut self, event: &SessionEvent) {
        match &mut self.ui {
            Some(ui) => pregame_event(ui, event),
            None => self.early.push(Ok(event.clone())),
        }
    }

    /// A client notice (`ConnectionLost`).
    pub fn notice(&mut self, notice: Notice) {
        match &mut self.ui {
            Some(ui) => pregame_notice(ui, notice),
            None => self.early.push(Err(notice)),
        }
    }

    /// The frame's UI actions; the messages they broadcast.
    pub fn input(&mut self, actions: &[(u32, f32, bool)]) -> Vec<UiMessage> {
        let Some(ui) = &mut self.ui else {
            return Vec::new();
        };
        let messages = ui.input(actions);
        self.dirty |= ui.tree_mut().take_changed();
        messages
    }

    /// The active text element's actions and characters; the messages.
    pub fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        ctrl: bool,
    ) -> Vec<UiMessage> {
        let Some(ui) = &mut self.ui else {
            return Vec::new();
        };
        let messages = ui.text_input(actions, characters, ctrl);
        self.dirty |= ui.tree_mut().take_changed();
        messages
    }

    /// Whether an editable text element is active (the input manager's text mode).
    pub fn text_mode(&self) -> bool {
        self.ui.as_ref().is_some_and(|ui| ui.tree().text_mode())
    }

    /// The UI's requests carried out on the session.
    pub fn requests(&mut self, session: &mut OnlineSession) -> PregameOutcome {
        match &mut self.ui {
            Some(ui) => pregame_requests(ui, session),
            None => PregameOutcome::default(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        if let Some(ui) = &mut self.ui {
            ui.tree_mut()
                .set_display(self.size.0 as i32, self.size.1 as i32);
        }
        self.dirty = true;
    }

    /// Takes the data when it arrived, runs the UI loop at `now` seconds, follows the scenes,
    /// answers a waiting pick (nothing without scenes) and the end of the chosen avatar's
    /// animation, and composes what changed.
    pub fn update(&mut self, gpu: &Gpu, now: f64) {
        self.step(gpu, now, false);
    }

    /// [`PregameView::update`], waiting for every load the frame asks for (headless snapshots).
    pub fn settle(&mut self, gpu: &Gpu, now: f64) {
        self.step(gpu, now, true);
        while self.scenes.as_ref().is_some_and(PregameScenes::busy) {
            self.step(gpu, now, true);
        }
        self.step(gpu, now, true);
    }

    fn step(&mut self, gpu: &Gpu, now: f64, wait: bool) {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(Ok(assets)) => {
                    self.pending = None;
                    let mut ui = PregameUi::new(assets, self.size.0 as i32, self.size.1 as i32);
                    if let Some(system) = self.system.take() {
                        ui.set_char_gen_system(system);
                    }
                    for early in self.early.drain(..) {
                        match early {
                            Ok(event) => pregame_event(&mut ui, &event),
                            Err(notice) => pregame_notice(&mut ui, notice),
                        }
                    }
                    self.ui = Some(ui);
                    self.dirty = true;
                }
                Ok(Err(error)) => {
                    self.pending = None;
                    eprintln!("ac2: the pregame UI did not load: {error}");
                }
                Err(TryRecvError::Disconnected) => self.pending = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        let Some(ui) = &mut self.ui else {
            return;
        };
        ui.use_time(now);
        if let Some(scenes) = &mut self.scenes {
            scenes.update(gpu, ui, now, wait);
        }
        if let Some(point) = ui.wants_pick() {
            let picked = self.scenes.as_ref().and_then(|s| s.pick(point));
            ui.finish_pick(picked);
        }
        if self.scenes.as_mut().is_some_and(|s| s.animation_done(ui)) {
            ui.finish_animation();
        }
        self.dirty |= ui.tree_mut().take_changed();
        if !self.dirty {
            // Only surfaces moved, turned or faded: placements, no new pixels.
            if ui.tree_mut().take_moved() {
                let placements = ui.tree().placements();
                self.overlay.set_placements(
                    placements
                        .into_iter()
                        .map(|p| (p.x, p.y, p.rotation, p.shown, p.opacity)),
                );
            }
            return;
        }
        self.dirty = false;
        let surfaces = ui.compose();
        ui.tree_mut().take_moved();
        self.layer_roots = surfaces.iter().map(|surface| surface.root).collect();
        self.overlay.set_layers(
            gpu,
            surfaces.iter().map(|surface| UiLayer {
                x: surface.x,
                y: surface.y,
                width: surface.width,
                height: surface.height,
                rotation: surface.rotation,
                shown: surface.shown,
                opacity: surface.opacity,
                rgba: &surface.rgba,
            }),
        );
    }

    pub fn shown_layers(&self) -> usize {
        self.overlay.shown_layers()
    }

    /// The viewports' scenes, when the view has them.
    pub fn scenes(&self) -> Option<&PregameScenes> {
        self.scenes.as_ref()
    }

    /// How many of the scenes' objects have their models loaded.
    pub fn scene_objects(&self) -> usize {
        self.scenes.as_ref().map_or(0, PregameScenes::drawn_objects)
    }

    /// Writes the frame's UI layout and scenes (before [`PregameView::render`]).
    pub fn prepare(&mut self, gpu: &Gpu, time: f64) {
        self.overlay.prepare(&gpu.queue, self.size.0, self.size.1);
        if let Some(scenes) = &mut self.scenes {
            scenes.prepare(gpu, time);
        }
    }

    /// Draws the frame into `color` cleared to black: the UI's surfaces in order, each scene
    /// in its viewport after its root's surface, in a pass of its own that clears the depth
    /// (`UIScene::Render`'s clear) and sets the device viewport to the root's rectangle
    /// (`UIViewportObject::Render`).
    pub fn render(&self, gpu: &Gpu, color: &wgpu::TextureView, depth: &wgpu::TextureView) {
        let mut views: Vec<(usize, usize, ac2_ui::pregame::SceneViewport)> = self
            .scenes
            .as_ref()
            .map(|s| s.drawn())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(view, viewport)| {
                let layer = self.layer_roots.iter().position(|&r| r == viewport.root)?;
                Some((layer, view, viewport))
            })
            .collect();
        views.sort_by_key(|(layer, ..)| *layer);
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        let mut next = 0;
        let (width, height) = self.size;
        for (i, (layer, view, viewport)) in views.iter().enumerate() {
            let mut pass = begin_pass(&mut encoder, color, depth, i == 0);
            self.overlay.draw_range(&mut pass, next..layer + 1);
            next = layer + 1;
            let x0 = viewport.x.clamp(0, width as i32) as u32;
            let y0 = viewport.y.clamp(0, height as i32) as u32;
            let x1 = (viewport.x + viewport.width as i32).clamp(0, width as i32) as u32;
            let y1 = (viewport.y + viewport.height as i32).clamp(0, height as i32) as u32;
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            pass.set_viewport(
                x0 as f32,
                y0 as f32,
                (x1 - x0) as f32,
                (y1 - y0) as f32,
                0.0,
                1.0,
            );
            if let Some(scenes) = &self.scenes {
                scenes.draw(*view, &mut pass);
            }
        }
        {
            let mut pass = begin_pass(&mut encoder, color, depth, views.is_empty());
            self.overlay
                .draw_range(&mut pass, next..self.layer_roots.len().max(next));
        }
        gpu.queue.submit([encoder.finish()]);
    }

    /// Records the UI layers alone into `pass`.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.overlay.draw(pass);
    }
}

/// A pass over `color` (cleared to black when `clear`, else kept) with its depth cleared.
fn begin_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    clear: bool,
) -> wgpu::RenderPass<'a> {
    let load = if clear {
        wgpu::LoadOp::Clear(wgpu::Color::BLACK)
    } else {
        wgpu::LoadOp::Load
    };
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ac2 pregame"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}
