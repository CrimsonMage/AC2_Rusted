//! Recording the draws laid out by `prepare`, and shader validation.

use super::*;

impl ObjectRenderer {
    /// Records group `group` of the draws laid out by the last `prepare`, which must have been
    /// given the same `instances` for it.
    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        group: usize,
        instances: &[ObjectInstance<'_>],
    ) {
        let Some(range) = self.groups.get(group) else {
            return;
        };
        pass.set_bind_group(
            0,
            &self.frame_group,
            &[(group as u64 * FRAME_STRIDE) as u32],
        );
        pass.set_bind_group(3, &self.bone_group, &[]);
        let mut current: Option<PipelineKey> = None;
        // The part whose buffers and the texture group last bound: consecutive draws of one part
        // (its batches and layers, or the same model placed again) keep them bound.
        let mut bound_part: Option<*const GpuPart> = None;
        let mut bound_textures: Option<*const wgpu::BindGroup> = None;
        for draw in &self.queue[range.clone()] {
            let Some(part) = instances
                .get(draw.instance)
                .and_then(|instance| instance.model.parts.get(draw.part))
            else {
                continue;
            };
            if current != Some(draw.key) {
                pass.set_pipeline(self.pipelines.pipeline(&draw.key));
                current = Some(draw.key);
            }
            let batch = &part.batches[draw.batch];
            pass.set_bind_group(1, &self.draws.group, &[draw.offset]);
            let textures = &batch.passes[draw.layer].textures;
            if bound_textures != Some(std::ptr::from_ref(textures)) {
                pass.set_bind_group(2, textures, &[]);
                bound_textures = Some(std::ptr::from_ref(textures));
            }
            if bound_part != Some(std::ptr::from_ref(part)) {
                pass.set_vertex_buffer(0, part.vertices.slice(..));
                pass.set_index_buffer(part.indices.slice(..), wgpu::IndexFormat::Uint32);
                bound_part = Some(std::ptr::from_ref(part));
            }
            pass.draw_indexed(
                batch.range.clone(),
                0,
                draw.first_instance..draw.first_instance + draw.instances,
            );
        }
    }

    /// Translates and compiles `program`, returning the device's validation error if any.
    pub fn validate_program(
        gpu: &Gpu,
        program: &ac2_render::object::PixelProgram,
    ) -> Option<String> {
        let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _module = object_module(&gpu.device, &shade_function(program));
        pollster::block_on(scope.pop()).map(|error| error.to_string())
    }
}
