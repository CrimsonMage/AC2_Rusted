//! What stands in a UI scene: its figures (physics objects, `UIScenePhysObjEntity`) and where.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ac2_anim::Ac2Transform;
use ac2_assets::Ac2VisualDesc;
use ac2_math::{Frame, Mat4, Vec3};
use ac2_render::object::ObjectPose;

use super::data::SceneData;

/// Where a figure stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    /// At a frame in the scene, scaled.
    Scene(Frame, Vec3),
    /// On figure `parent`'s holding location `location` (`CPhysicsObj::add_child`).
    Held { parent: usize, location: u32 },
}

/// One physics object of a viewport's scene this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    /// Names the figure across frames (its animated body).
    pub key: u64,
    /// What `makeObject` reads: an `EntityDesc` template or a visual. For a figure made from an
    /// appearance, its parent visual (the body it animates with).
    pub object: u32,
    /// `makePartGroupObject`'s appearance, when the figure is made from one.
    pub local: Option<Ac2VisualDesc>,
    pub at: At,
    /// What a pick on it answers, if anything.
    pub pick: Option<usize>,
    /// The mode it goes to once made (`CPhysicsObj::SetMode`).
    pub mode: Option<u32>,
}

/// A stable figure key from a scene, a tag and an index.
pub fn figure_key(scene: u32, tag: &str, index: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    (scene, tag, index).hash(&mut hasher);
    hasher.finish()
}

/// The scene's own physics objects (`UIScene::AddEntity`), in entity order.
pub fn scene_figures(did: u32, scene: &SceneData) -> Vec<Figure> {
    scene
        .entities
        .iter()
        .enumerate()
        .map(|(i, entity)| Figure {
            key: figure_key(did, "entity", i),
            object: entity.object,
            local: None,
            at: At::Scene(entity.frame, entity.scale),
            pick: None,
            mode: None,
        })
        .collect()
}

/// A loaded figure's model as drawn this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawn {
    /// The model's key (`WorldScene::request_object`).
    pub key: u64,
    pub world: Mat4,
    pub pose: Option<ObjectPose>,
    /// The model's own (`VisualDesc`) scale.
    pub rig_scale: Vec3,
    /// The model's box at rest, unscaled (its pick mesh's bounds).
    pub bounds: Option<(Vec3, Vec3)>,
}

/// One figure laid out this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub key: u64,
    pub object: u32,
    pub pick: Option<usize>,
    /// The figure's frame (no scale) and scale, for what it holds.
    pub frame: Frame,
    pub scale: Vec3,
    /// Its model, once loaded.
    pub drawn: Option<Drawn>,
}

impl Placed {
    /// The figure moved to `frame` (`UISceneEntity::SetFrame`).
    pub fn move_to(&mut self, frame: Frame) {
        self.frame = frame;
        if let Some(drawn) = self.drawn.as_mut() {
            drawn.world = Mat4::from_scale_rotation_translation(
                self.scale * drawn.rig_scale,
                frame.orientation,
                frame.origin,
            );
        }
    }

    /// The model's height, scaled (`CPhysicsObj`'s box, `max.z - min.z`).
    pub fn height(&self) -> Option<f32> {
        let drawn = self.drawn.as_ref()?;
        let (min, max) = drawn.bounds?;
        Some((max.z - min.z) * self.scale.z * drawn.rig_scale.z)
    }
}

/// `CPhysicsObj::add_child`: the child's frame on its parent's location (the parent's scale
/// moves the location, not the child's size).
pub(crate) fn held_frame(parent: &Frame, scale: Vec3, location: &Ac2Transform) -> Frame {
    let offset = parent.orientation * (location.offset * scale);
    Frame::new(
        parent.origin + offset,
        (parent.orientation * location.rotation).normalize(),
    )
}
