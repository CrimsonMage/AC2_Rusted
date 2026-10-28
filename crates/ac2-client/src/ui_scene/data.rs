//! The scenes' loader thread: each `UIScene` and object is read on it (the data's reads live in
//! `ac2_render::ui_scene::data`), with an object's body for animation (`load_body_data`) added
//! here, and the frame thread takes what finished.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use ac2_db::ResourceStore;

use ac2_render::ui_scene::data::{load_object_shape, load_scene, ObjectData, SceneData};

use crate::world_data::load_body_data;

/// Work for the scene loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Request {
    Scene(u32),
    Object(u32),
}

/// Finished work: `None` where the DATs lack it.
pub(crate) enum Loaded {
    Scene(u32, Option<Arc<SceneData>>),
    Object(u32, Option<Arc<ObjectData>>),
}

/// The scenes' loader thread.
pub(crate) struct SceneLoader {
    requests: Sender<Request>,
    finished: Receiver<Loaded>,
    pub scenes: HashMap<u32, Option<Arc<SceneData>>>,
    pub objects: HashMap<u32, Option<Arc<ObjectData>>>,
    pending: std::collections::HashSet<Request>,
}

impl SceneLoader {
    pub fn new(store: Arc<ResourceStore>) -> Self {
        let (requests, work) = channel::<Request>();
        let (done, finished) = channel();
        std::thread::Builder::new()
            .name("ac2 pregame scenes".into())
            .spawn(move || {
                for request in work {
                    let loaded = match request {
                        Request::Scene(did) => {
                            Loaded::Scene(did, load_scene(&store, did).map(Arc::new))
                        }
                        Request::Object(did) => {
                            Loaded::Object(did, load_object(&store, did).map(Arc::new))
                        }
                    };
                    if done.send(loaded).is_err() {
                        return;
                    }
                }
            })
            .expect("spawn the pregame scene loader");
        Self {
            requests,
            finished,
            scenes: HashMap::new(),
            objects: HashMap::new(),
            pending: Default::default(),
        }
    }

    /// Asks for `request` unless it is loaded or under way.
    pub fn request(&mut self, request: Request) {
        let known = match request {
            Request::Scene(did) => self.scenes.contains_key(&did),
            Request::Object(did) => self.objects.contains_key(&did),
        };
        if !known && self.pending.insert(request) {
            let _ = self.requests.send(request);
        }
    }

    /// Takes what finished; waits for everything asked when `wait` (headless snapshots).
    pub fn poll(&mut self, wait: bool) {
        loop {
            let loaded = if wait && !self.pending.is_empty() {
                match self.finished.recv() {
                    Ok(loaded) => loaded,
                    Err(_) => return,
                }
            } else {
                match self.finished.try_recv() {
                    Ok(loaded) => loaded,
                    Err(_) => return,
                }
            };
            match loaded {
                Loaded::Scene(did, scene) => {
                    self.pending.remove(&Request::Scene(did));
                    self.scenes.insert(did, scene);
                }
                Loaded::Object(did, object) => {
                    self.pending.remove(&Request::Object(did));
                    self.objects.insert(did, object);
                }
            }
        }
    }

    /// Whether loads are under way.
    pub fn busy(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn scene(&self, did: u32) -> Option<&Arc<SceneData>> {
        self.scenes.get(&did)?.as_ref()
    }

    pub fn object(&self, did: u32) -> Option<&Arc<ObjectData>> {
        self.objects.get(&did)?.as_ref()
    }
}

/// `ICPhysicsObj::makeObject`'s reads: the object's shape (`load_object_shape`) and its body.
fn load_object(store: &Arc<ResourceStore>, did: u32) -> Option<ObjectData> {
    let mut object = load_object_shape(store, did)?;
    object.body = object
        .visual
        .and_then(|visual| load_body_data(store, visual).ok())
        .map(|body| body.body);
    Some(object)
}
