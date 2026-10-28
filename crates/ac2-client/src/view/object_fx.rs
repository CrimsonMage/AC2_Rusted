//! The client's view of the objects' FX: the particle, light and streak calls the FX made reach
//! the scene in order (the drawing itself is `ac2_render::object_fx`).

use ac2_render::decal::DecalSpec;
use ac2_render::fx_particles::{ObjectLightStart, ObjectParticleStart};
use ac2_render::fx_streaks::ObjectStreakCall;
use ac2_render::object_fx;

use super::ClientView;

impl ClientView {
    /// The particle systems the objects' FX started and stopped, see
    /// [`ac2_render::object_fx::apply_particles`].
    pub fn set_object_particles(&mut self, starts: &[ObjectParticleStart], stops: &[u64]) {
        object_fx::apply_particles(&mut self.scene, starts, stops);
    }

    /// The lights the objects' FX placed and the objects that left, see
    /// [`ac2_render::object_fx::apply_lights`].
    pub fn set_object_lights(&mut self, starts: &[ObjectLightStart], gone: &[u64]) {
        object_fx::apply_lights(&mut self.scene, starts, gone);
    }

    /// The streaks the objects' FX made, started, stopped and destroyed, see
    /// [`ac2_render::object_fx::apply_streaks`].
    pub fn set_object_streaks(&mut self, calls: &[ObjectStreakCall], visible: bool) {
        object_fx::apply_streaks(&mut self.scene, calls, visible);
    }

    /// The decals the objects' FX projected, see [`ac2_render::object_fx::apply_decals`].
    pub fn set_object_decals(&mut self, calls: &[DecalSpec]) {
        object_fx::apply_decals(&mut self.scene, calls);
    }
}
