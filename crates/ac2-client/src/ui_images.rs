//! UI images set after the HUD was built (skill icons, shortcut icons, ...): `Graphic::Graphic`
//! asks for the image and the element draws nothing until it is in. A worker reads and decodes
//! each image once; the frame takes the finished ones and composes again.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_ui::{UiPixels, UiTree};

pub(crate) struct UiImageLoader {
    send: Sender<u32>,
    receive: Receiver<(u32, Option<UiPixels>)>,
    asked: HashSet<u32>,
}

impl UiImageLoader {
    pub(crate) fn new(store: Arc<ResourceStore>) -> Self {
        let (send, requests) = mpsc::channel::<u32>();
        let (done, receive) = mpsc::channel();
        std::thread::Builder::new()
            .name("ac2 ui images".into())
            .spawn(move || {
                for file in requests {
                    let pixels = ac2_assets::load_ac2_ui_surface(&store, DataId::new(file))
                        .ok()
                        .flatten()
                        .map(|surface| UiPixels {
                            width: surface.width,
                            height: surface.height,
                            rgba: surface.rgba,
                        });
                    if done.send((file, pixels)).is_err() {
                        return;
                    }
                }
            })
            .expect("spawn the UI image loader");
        Self {
            send,
            receive,
            asked: HashSet::new(),
        }
    }

    /// Asks for every image the tree shows that is not loaded or asked for yet.
    pub(crate) fn request_missing(&mut self, tree: &UiTree, images: &HashMap<u32, UiPixels>) {
        for file in tree.image_files() {
            if file != 0 && !images.contains_key(&file) && self.asked.insert(file) {
                let _ = self.send.send(file);
            }
        }
    }

    /// The images loaded since the last call into `images`; whether any arrived.
    pub(crate) fn take(&mut self, images: &mut HashMap<u32, UiPixels>) -> bool {
        let mut any = false;
        for (file, pixels) in self.receive.try_iter() {
            if let Some(pixels) = pixels {
                images.insert(file, pixels);
                any = true;
            }
        }
        any
    }
}
