//! The UI scenes' data, read on a loader thread (retail `UISceneManager::GetUIScene`'s `DBObj::Get`
//! and `ICPhysicsObj::makeObject`'s reads): a `UIScene` with its entities set up
//! (`UIScene::AddEntity`, `UISceneEntity::InitFromDesc`, the camera's `SceneCamera::ApplyProperty`
//! and the lights' `LightSource::ApplyProperty`), and an object's chain of `EntityDesc` templates
//! down to its `VisualDesc`, its root Setup's holding locations and its shape for animation.

use ac2_anim::{Ac2Location, Ac2Setup, Ac2Transform};
use ac2_assets::{
    load_ac2_entity_desc, load_ac2_ui_scene, Ac2EntityDesc, UI_SCENE_CAMERA, UI_SCENE_LIGHT,
    UI_SCENE_PHYS_OBJ, UI_SCENE_PHYS_OBJ_ALT,
};
use ac2_core::DataId;
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{read_ac2_property_collection, ResourceStore, TypedResourceRef};
use ac2_math::{Frame, Vec3};
use ac2_smartbox::BodyData;

/// `EntityDesc` templates an object may chain through (0x47 DIDs) before its visual.
const MAX_TEMPLATES: usize = 4;
const ENTITY_TEMPLATE: u32 = 0x4700_0000;

/// `SceneCamera`'s properties (group 0x4000000A).
const CAMERA_ORTHOGRAPHIC: u32 = 0x4000_001E;
const CAMERA_FOV: u32 = 0x4000_001F;
const CAMERA_VERTICAL_SCALE: u32 = 0x4000_0020;
const CAMERA_NEAR: u32 = 0x4000_0021;
const CAMERA_FAR: u32 = 0x4000_0022;

/// `LightSource`'s properties.
const LIGHT_TYPE: u32 = 0x4000_0002;
const LIGHT_DIRECTION: u32 = 0x4000_0003;
const LIGHT_RANGE: u32 = 0x4000_0004;
const LIGHT_DIFFUSE: u32 = 0x4000_0008;
const LIGHT_INTENSITY: u32 = 0x4000_0009;
const LIGHT_SPECULAR: u32 = 0x4000_000A;
/// `LightSource::m_Type`.
pub const LIGHT_POINT: u32 = 0x4000_0001;
pub const LIGHT_DIRECTIONAL: u32 = 0x4000_0002;

/// A camera entity (`UISceneCameraEntity`) with its `SceneCamera` settings.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraData {
    pub name: Option<String>,
    pub frame: Frame,
    /// `mbOrthographic`, `mFOV` (vertical, degrees), `mVerticalScale`, `mNearDistance`,
    /// `mFarDistance` (`SceneCamera::SceneCamera`: perspective, 65, 1, 0.1, 2000).
    pub orthographic: bool,
    pub fov: f32,
    pub vertical_scale: f32,
    pub near: f32,
    pub far: f32,
}

/// A light entity (`UISceneLightEntity`): its `LightSource` at the entity's origin
/// (`UISceneLightEntity::SetFrame`).
#[derive(Debug, Clone, PartialEq)]
pub struct LightData {
    pub name: Option<String>,
    pub origin: Vec3,
    /// `m_Type` (`LightSource::LightSource`: point).
    pub kind: u32,
    pub direction: Vec3,
    /// `m_wRange` and `m_wDiffuseIntensity` base values (5 and 0.5 by default).
    pub range: f32,
    pub intensity: f32,
    /// `m_cDiffuse`, `m_cSpecular` (white by default) as RGB in 0..1.
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

/// A physics object entity (`UIScenePhysObjEntity`): its `EntityDesc`'s object, frame and name.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityData {
    pub name: Option<String>,
    pub frame: Frame,
    pub scale: Vec3,
    /// What `ICPhysicsObj::makeObject(EntityDesc)` makes the object from (its `data`).
    pub object: u32,
}

/// A loaded `UIScene`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneData {
    pub entities: Vec<EntityData>,
    pub cameras: Vec<CameraData>,
    pub lights: Vec<LightData>,
    /// `SceneTool::SetFogProperties(m_bWantFog, m_FogColor, m_fFogNear, m_fFogFar)`.
    pub fog: Option<([f32; 3], (f32, f32))>,
}

/// A physics object as `makeObject` builds it.
#[derive(Clone, Default)]
pub struct ObjectData {
    /// The `VisualDesc` (or Setup) the template chain ends at.
    pub visual: Option<DataId>,
    /// The templates' scales, multiplied.
    pub scale: Vec3,
    /// The root Setup's holding locations, each in model space at the rest pose
    /// (`CPhysicsObj::add_child`'s `LocationType` frame on its part).
    pub holding: Vec<HoldingPoint>,
    /// The root Setup's bounds (`CSetup` render box).
    pub bounds: Option<(Vec3, Vec3)>,
    /// The animated body, when the visual has a motion table.
    pub body: Option<BodyData>,
}

impl ObjectData {
    /// Holding location `id` in model space, on its bone posed at `pose` (model-space bones)
    /// when the object is animated, else at rest.
    pub fn location(&self, id: u32, pose: Option<&[Ac2Transform]>) -> Option<Ac2Transform> {
        let point = self.holding.iter().find(|p| p.id == id)?;
        Some(match pose.and_then(|p| p.get(point.bone)) {
            Some(bone) => bone.compose(&point.local),
            None => point.rest,
        })
    }
}

/// A Setup holding location (`LocationType`): on bone `bone` at `local` in that bone's space,
/// `rest` in model space at the rest pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoldingPoint {
    pub id: u32,
    pub bone: usize,
    pub local: Ac2Transform,
    pub rest: Ac2Transform,
}

pub fn rgb(argb: u32) -> [f32; 3] {
    let channel = |shift: u32| ((argb >> shift) & 0xFF) as f32 / 255.0;
    [channel(16), channel(8), channel(0)]
}

fn frame_of(desc: &Ac2EntityDesc) -> Frame {
    Frame::new(desc.origin, desc.rotation)
}

/// `UIScene::StreamPack` and `AddEntity` for each `EntityDesc` by type.
pub fn load_scene(store: &ResourceStore, did: u32) -> Option<SceneData> {
    let scene = load_ac2_ui_scene(store, DataId::new(did)).ok()??;
    let mut data = SceneData {
        fog: scene.want_fog.then(|| {
            (
                rgb(scene.fog_color),
                (scene.fog_near, scene.fog_far.max(scene.fog_near)),
            )
        }),
        ..SceneData::default()
    };
    for entity in &scene.entities {
        let properties = entity
            .properties
            .as_ref()
            .and_then(|bytes| read_ac2_property_collection(&mut ArchiveCursor::new(bytes)).ok())
            .unwrap_or_default();
        match entity.entity_type {
            UI_SCENE_PHYS_OBJ | UI_SCENE_PHYS_OBJ_ALT => {
                if let Some(object) = entity.data {
                    data.entities.push(EntityData {
                        name: entity.workbench_name.clone(),
                        frame: frame_of(entity),
                        scale: entity.scale,
                        object: object.raw(),
                    });
                }
            }
            UI_SCENE_CAMERA => {
                let mut camera = CameraData {
                    name: entity.workbench_name.clone(),
                    frame: frame_of(entity),
                    orthographic: false,
                    fov: 65.0,
                    vertical_scale: 1.0,
                    near: 0.1,
                    far: 2000.0,
                };
                for p in &properties {
                    match p.name {
                        CAMERA_ORTHOGRAPHIC => {
                            camera.orthographic = p.word().is_some_and(|w| w != 0)
                        }
                        CAMERA_FOV => camera.fov = p.float().unwrap_or(camera.fov),
                        CAMERA_VERTICAL_SCALE => {
                            camera.vertical_scale = p.float().unwrap_or(camera.vertical_scale)
                        }
                        CAMERA_NEAR => camera.near = p.float().unwrap_or(camera.near),
                        CAMERA_FAR => camera.far = p.float().unwrap_or(camera.far),
                        _ => {}
                    }
                }
                data.cameras.push(camera);
            }
            UI_SCENE_LIGHT => {
                let mut light = LightData {
                    name: entity.workbench_name.clone(),
                    origin: entity.origin,
                    kind: LIGHT_POINT,
                    direction: Vec3::ZERO,
                    range: 5.0,
                    intensity: 0.5,
                    diffuse: [1.0; 3],
                    specular: [1.0; 3],
                };
                for p in &properties {
                    match p.name {
                        LIGHT_TYPE => light.kind = p.word().unwrap_or(light.kind),
                        LIGHT_DIRECTION => {
                            if let Some(v) = p.vector() {
                                light.direction = Vec3::from_array(v);
                            }
                        }
                        // `Waveform`: its kind, then the base value.
                        LIGHT_RANGE => light.range = p.float_at(1).unwrap_or(light.range),
                        LIGHT_INTENSITY => {
                            light.intensity = p.float_at(1).unwrap_or(light.intensity)
                        }
                        LIGHT_DIFFUSE => light.diffuse = p.word().map_or(light.diffuse, rgb),
                        LIGHT_SPECULAR => light.specular = p.word().map_or(light.specular, rgb),
                        _ => {}
                    }
                }
                data.lights.push(light);
            }
            _ => {}
        }
    }
    Some(data)
}

/// `ICPhysicsObj::makeObject`'s reads that need no body: the template chain, the root Setup's
/// holding locations and bounds. The caller adds the body (`BodyData`) for the animation.
pub fn load_object_shape(store: &ResourceStore, did: u32) -> Option<ObjectData> {
    let mut current = DataId::new(did);
    let mut scale = Vec3::ONE;
    for _ in 0..MAX_TEMPLATES {
        if current.raw() & 0xFF00_0000 != ENTITY_TEMPLATE {
            break;
        }
        let desc = load_ac2_entity_desc(store, current).ok()??;
        scale *= desc.scale;
        current = desc.data?;
    }
    if current.raw() & 0xFF00_0000 == ENTITY_TEMPLATE {
        return None;
    }
    let mut object = ObjectData {
        visual: Some(current),
        scale,
        ..ObjectData::default()
    };
    let groups = crate::object::resolve_drawn_groups(store, current, None)
        .ok()
        .flatten();
    let root = groups.as_ref().and_then(|g| g.first()).map(|g| g.setup);
    if let Some(setup) = root.and_then(|did| {
        store
            .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
            .ok()
            .flatten()
    }) {
        let rest = setup.rest_pose();
        object.holding = setup
            .holding_points
            .iter()
            .map(|l: &Ac2Location| HoldingPoint {
                id: l.id,
                bone: l.bone,
                local: l.transform,
                rest: rest
                    .get(l.bone)
                    .copied()
                    .unwrap_or(Ac2Transform::IDENTITY)
                    .compose(&l.transform),
            })
            .collect();
        object.bounds = Some((setup.bounds_min, setup.bounds_max));
    }
    Some(object)
}
