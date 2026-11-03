//! The portal space drawn into the scene target (`PortalSpace::Effect::Render`), seen from its own
//! camera over a cleared depth, in retail's order: the full-screen fader (`DrawScreenQuad` of
//! `m_cFaderColor`, SRCALPHA over INVSRCALPHA) first when it covers only the world
//! (`m_bFadeOnlyWorld`), the background mesh at the eye (`CBaseMesh::DrawMesh` with the
//! background opacity as its alpha), the tunnel rings (the outer, then the inner, each through its
//! DAT material instance as `RenderIndexedUsingMaterial` draws it), the clouds' particles
//! (`ParticleSystem::Render`), and the fader last otherwise. The portal camera rolls with the
//! tunnel (`SceneTool::SetupView` takes its frame's three axes), which the renderer's look-at
//! cameras cannot do, so the geometry is turned about the eye by the camera's inverse rotation
//! ([`PortalCamera::upright`]) and drawn from an upright camera there: the same view. The mesh,
//! the tunnels and the particles draw through the object renderer with no lights,
//! sun or fog: `Render` calls `UnlinkAllLights` (no scene lights), `DisableSunlight` (the sun's
//! direction zeroed and the device ambient set to black, `RenderDevice` vtable `+0x4c`, the same
//! slot `SetAmbientLight` calls) and `SetFogProperties(FOGMODE_NONE)`.
//!
//! The models and their images are uploaded by [`PortalDraw::preload`] at setup, never on the
//! frame thread: a frame only uses what was uploaded (and the tunnels' rewritten vertices).

use ac2_math::Mat4;
use ac2_render::object::{ObjectEnvironment, ObjectLights};
use ac2_render::particles::ParticleGeometry;
use ac2_render::portal::{
    camera_fov_y, tunnel_object_vertices, Draw, PortalCamera, PortalModels, Tunnel, TunnelFrame,
    CAMERA_FAR, CAMERA_NEAR,
};
use ac2_render::Image;

use crate::{
    DrawGroup, Gpu, GpuObjectModel, ObjectInstance, ObjectRenderer, ScreenQuad, TextureKey,
    HDR_FORMAT,
};

/// The portal space's draws: the fader, the background mesh, the tunnels and the clouds.
pub struct PortalDraw {
    fader: ScreenQuad,
    fader_drawn: bool,
    /// `m_bFadeOnlyWorld`: the fader is drawn before the tunnels (the startup fade), else after.
    fader_first: bool,
    tunnel_drawn: bool,
    /// The tunnels' shared index list (`Initialize`'s index buffer), widened for the renderer.
    tunnel_indices: Vec<u32>,
    /// The background mesh, the outer and inner tunnels' and the clouds' material models, drawn
    /// by `objects`.
    objects: ObjectRenderer,
    background: Option<GpuObjectModel>,
    outer_tunnel: Option<GpuObjectModel>,
    inner_tunnel: Option<GpuObjectModel>,
    clouds: Option<GpuObjectModel>,
    /// The mesh's place (translated to the eye, then turned upright) and colour (white at the
    /// background opacity).
    background_world: Mat4,
    /// The tunnels' and the clouds' place: their world coordinates turned upright about the eye
    /// ([`PortalCamera::upright`]; retail drew them at the identity).
    portal_world: Mat4,
    background_tint: [f32; 4],
    background_drawn: bool,
    clouds_drawn: bool,
}

/// No lights, no sun, black ambient and no fog (`UnlinkAllLights`, `DisableSunlight`,
/// `SetFogProperties(FOGMODE_NONE)`): what draws is what the materials add themselves.
fn unlit() -> ObjectEnvironment {
    ObjectEnvironment {
        lights: ObjectLights {
            ambient: [0.0; 3],
            sun: None,
        },
        fog: None,
    }
}

/// `instance`'s group of at most one placed model.
fn placed(
    model: &Option<GpuObjectModel>,
    world: Mat4,
    tint: [f32; 4],
    drawn: bool,
) -> Vec<ObjectInstance<'_>> {
    model
        .as_ref()
        .filter(|_| drawn)
        .map(|model| ObjectInstance {
            tint,
            ..ObjectInstance::new(model, world)
        })
        .into_iter()
        .collect()
}

impl PortalDraw {
    /// The draws into the scene target; the tunnel's index buffer is the effect's, built from a
    /// fresh [`Tunnel`].
    pub fn new(gpu: &Gpu) -> Self {
        let tunnel_indices = Tunnel::new()
            .index_buffer()
            .into_iter()
            .map(u32::from)
            .collect();
        Self {
            fader: ScreenQuad::new(gpu, HDR_FORMAT),
            fader_drawn: false,
            fader_first: true,
            tunnel_drawn: false,
            tunnel_indices,
            objects: ObjectRenderer::new(gpu, HDR_FORMAT),
            background: None,
            outer_tunnel: None,
            inner_tunnel: None,
            clouds: None,
            background_world: Mat4::IDENTITY,
            portal_world: Mat4::IDENTITY,
            background_tint: [1.0; 4],
            background_drawn: false,
            clouds_drawn: false,
        }
    }

    /// Uploads the background mesh's, the tunnels' (`m_pTunnelMaterial`,
    /// `m_pOuterTunnelMaterial`) and the clouds' models with the images `load` decodes, and builds
    /// their pipelines. Called at setup, not per frame.
    pub fn preload(
        &mut self,
        gpu: &Gpu,
        models: &PortalModels,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) {
        self.outer_tunnel = models
            .outer_tunnel
            .as_ref()
            .map(|model| self.objects.upload(gpu, model, load));
        self.inner_tunnel = models
            .inner_tunnel
            .as_ref()
            .map(|model| self.objects.upload(gpu, model, load));
        self.background = models
            .background
            .as_ref()
            .map(|model| self.objects.upload(gpu, model, load));
        self.clouds = models
            .clouds
            .as_ref()
            .map(|model| self.objects.upload(gpu, model, load));
        // Build the pipelines now (the first prepare waits for them) rather than on the first
        // frame of a teleport.
        self.background_drawn = true;
        self.clouds_drawn = true;
        self.tunnel_drawn = true;
        let camera = ac2_render::Camera {
            eye: ac2_math::Vec3::ZERO,
            target: ac2_math::Vec3::Y,
            fov_y: camera_fov_y(1.0),
            near: CAMERA_NEAR,
            far: CAMERA_FAR,
        };
        self.prepare_objects(gpu, &camera, 1.0, 0.0);
        self.background_drawn = false;
        self.clouds_drawn = false;
        self.tunnel_drawn = false;
    }

    /// Objects appear once their pipelines are compiled rather than stalling the frame that first
    /// shows them (the running client; headless renders wait instead).
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.objects.set_stream_pipelines(stream);
    }

    /// Whether the effect draws anything this frame.
    pub fn drawn(&self) -> bool {
        self.fader_drawn || self.tunnel_drawn || self.background_drawn
    }

    /// Lays out the background mesh (group 0), the outer and inner tunnels (groups 1 and 2) and
    /// the clouds (group 3) for `camera`.
    fn prepare_objects(&mut self, gpu: &Gpu, camera: &ac2_render::Camera, aspect: f32, time: f64) {
        let background = placed(
            &self.background,
            self.background_world,
            self.background_tint,
            self.background_drawn,
        );
        // `RenderIndexedUsingMaterial` with an identity world transform and `RGBAColor_White`.
        let outer = placed(
            &self.outer_tunnel,
            self.portal_world,
            [1.0; 4],
            self.tunnel_drawn,
        );
        let inner = placed(
            &self.inner_tunnel,
            self.portal_world,
            [1.0; 4],
            self.tunnel_drawn,
        );
        let clouds = placed(&self.clouds, self.portal_world, [1.0; 4], self.clouds_drawn);
        let group = |instances| DrawGroup {
            instances,
            ordered: true,
            environment: None,
            sky: false,
        };
        self.objects.prepare(
            gpu,
            camera,
            &unlit(),
            aspect,
            time,
            &[
                group(&background),
                group(&outer),
                group(&inner),
                group(&clouds),
            ],
        );
    }

    /// The frame's portal state at client time `time`: the fader, and from `camera` (while the
    /// effect is drawn) the background mesh, the `tunnel` rings (while it draws the tunnels, once
    /// the materials are uploaded) and the `clouds` (`aspect` for the projection).
    pub fn update(
        &mut self,
        gpu: &Gpu,
        draw: &Draw,
        camera: Option<&PortalCamera>,
        tunnel: Option<&TunnelFrame>,
        clouds: &ParticleGeometry,
        aspect: f32,
        time: f64,
    ) {
        self.fader_drawn = draw.fader;
        self.fader_first = draw.fade_only_world;
        if draw.fader {
            // The authored colour, decoded for the linear scene; its alpha is the fade.
            let [r, g, b, a] = draw.fader_color;
            let [r, g, b] = [r, g, b].map(|c| crate::srgb_to_linear(c.clamp(0.0, 1.0)));
            self.fader.set_color(&gpu.queue, [r, g, b, a]);
        }
        self.tunnel_drawn = false;
        self.background_drawn = false;
        self.clouds_drawn = false;
        let Some(camera) = camera else {
            return;
        };
        // `SetFOV(mFOV * m_fCameraFOVScale)` on `SceneCamera`'s defaults; the rolled view as an
        // upright camera over geometry turned about the eye.
        let upright = camera.upright();
        let camera =
            camera.to_render_camera(camera_fov_y(draw.camera_fov_scale), CAMERA_NEAR, CAMERA_FAR);
        self.portal_world = upright;
        // `UpdateGeometry` rewrote both tunnels' vertex buffers; they draw through their
        // materials over the shared index list.
        if let (Some(tunnel), Some(outer), Some(inner)) = (
            tunnel,
            self.outer_tunnel.as_mut(),
            self.inner_tunnel.as_mut(),
        ) {
            let outer_vertices = tunnel_object_vertices(&tunnel.outer);
            let inner_vertices = tunnel_object_vertices(&tunnel.inner);
            self.objects
                .set_geometry(gpu, outer, &outer_vertices, &self.tunnel_indices);
            self.objects
                .set_geometry(gpu, inner, &inner_vertices, &self.tunnel_indices);
            self.tunnel_drawn = true;
        }
        // `DrawMesh` at the camera's position, its colour white with the background opacity.
        self.background_drawn = draw.background && self.background.is_some();
        self.background_world = upright * Mat4::from_translation(camera.eye);
        self.background_tint = [1.0, 1.0, 1.0, draw.background_opacity];
        // `ParticleSystem::Render`: the frame's particles go into the clouds' material model.
        if let (Some(model), false) = (self.clouds.as_mut(), clouds.is_empty()) {
            self.objects
                .set_geometry(gpu, model, &clouds.vertices, &clouds.indices);
            self.clouds_drawn = true;
        }
        self.prepare_objects(gpu, &camera, aspect, time);
    }

    /// Draws the portal space, in `Render`'s order.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.fader_drawn && self.fader_first {
            self.fader.draw(pass);
        }
        let background = placed(
            &self.background,
            self.background_world,
            self.background_tint,
            self.background_drawn,
        );
        self.objects.draw(pass, 0, &background);
        let outer = placed(
            &self.outer_tunnel,
            self.portal_world,
            [1.0; 4],
            self.tunnel_drawn,
        );
        self.objects.draw(pass, 1, &outer);
        let inner = placed(
            &self.inner_tunnel,
            self.portal_world,
            [1.0; 4],
            self.tunnel_drawn,
        );
        self.objects.draw(pass, 2, &inner);
        let clouds = placed(&self.clouds, self.portal_world, [1.0; 4], self.clouds_drawn);
        self.objects.draw(pass, 3, &clouds);
        if self.fader_drawn && !self.fader_first {
            self.fader.draw(pass);
        }
    }
}
