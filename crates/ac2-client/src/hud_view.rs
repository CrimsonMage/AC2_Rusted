//! The gameplay HUD over the world (`window` feature): the element tree
//! `gmGamePlayUI::SetupChildren`'s frameworks create, built and its images read on a worker
//! thread, then composed (`UIRegion::DrawHere`, software, as retail) whenever it arrives or the
//! window changes size, and drawn as one layer per root surface.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use ac2_db::{ResourceResult, ResourceStore};
use ac2_gpu::{Gpu, UiLayer, UiOverlay};
use ac2_ui::{UiPixels, UiTree};

type Built = ResourceResult<(UiTree, HashMap<u32, UiPixels>)>;

pub(crate) struct HudView {
    pending: Option<Receiver<Built>>,
    hud: Option<(UiTree, HashMap<u32, UiPixels>)>,
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
                let built = ac2_ui::build_gameplay_hud(&store, width, height).and_then(|tree| {
                    let images = ac2_ui::load_ui_images(&store, &tree)?;
                    Ok((tree, images))
                });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            overlay: UiOverlay::new(gpu, format),
            size,
            dirty: false,
        }
    }

    /// The render device's new size: every root placed again (`UIElement::UpdateSizeAndPosition`).
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        self.dirty = true;
    }

    fn arrived(&mut self, built: Built) {
        match built {
            Ok(hud) => {
                self.hud = Some(hud);
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

    fn compose(&mut self, gpu: &Gpu) {
        if !self.dirty {
            return;
        }
        let Some((tree, images)) = &mut self.hud else {
            return;
        };
        self.dirty = false;
        tree.set_display(self.size.0 as i32, self.size.1 as i32);
        let surfaces = tree.compose(images);
        self.overlay.set_layers(
            gpu,
            surfaces.iter().map(|surface| UiLayer {
                x: surface.x,
                y: surface.y,
                width: surface.width,
                height: surface.height,
                rgba: &surface.rgba,
            }),
        );
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
