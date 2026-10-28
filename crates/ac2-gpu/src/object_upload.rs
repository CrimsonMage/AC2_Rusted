//! Uploads of object models (`upload`, `set_geometry`) and the per-pass pixel program and stage bind groups they need.

use super::*;

impl ObjectRenderer {
    pub fn upload(
        &mut self,
        gpu: &Gpu,
        model: &ObjectModel,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> GpuObjectModel {
        let device = &gpu.device;
        let mut parts = Vec::with_capacity(model.parts.len());
        let (mut sum, mut count) = (Vec3::ZERO, 0.0_f32);
        let mut radius = 0.0_f32;
        for part in &model.parts {
            let vertices: Vec<GpuVertex> = part
                .vertices
                .iter()
                .enumerate()
                .map(|(index, v)| {
                    let influences = part
                        .skin
                        .as_ref()
                        .and_then(|skin| skin.get(index))
                        .copied()
                        .unwrap_or_default();
                    GpuVertex {
                        position: v.position,
                        normal: v.normal,
                        color: v.color,
                        uvs: v.uvs,
                        tangents: v.tangents,
                        bones: influences.map(|(bone, _)| bone),
                        weights: influences.map(|(_, weight)| weight),
                    }
                })
                .collect();
            for v in part.vertices.iter() {
                let position = part
                    .transform
                    .transform_point3(Vec3::from_array(v.position));
                sum += position;
                radius = radius.max(position.length());
                count += 1.0;
            }
            let mut indices = Vec::new();
            let mut batches = Vec::with_capacity(part.batches.len());
            for batch in &part.batches {
                let start = indices.len() as u32;
                indices.extend_from_slice(&batch.indices);
                let passes = batch
                    .material
                    .passes([1.0; 4], self.time)
                    .into_iter()
                    .map(|pass| GpuPass {
                        textures: self.stage_group(gpu, &pass, load),
                        program: self.program_id(&pass),
                        pass,
                    })
                    .collect();
                batches.push(GpuBatch {
                    range: start..indices.len() as u32,
                    material: Arc::clone(&batch.material),
                    passes,
                });
            }
            parts.push(GpuPart {
                vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ac2 object vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ac2 object indices"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                transform: part.transform,
                batches,
                weighted: part.skin.is_some(),
                streamed: false,
            });
        }
        let first_list = model.parts.first().map_or((0, 0), |part| {
            let indices = part.batches.first().map_or(0, |b| b.indices.len());
            (indices as u32, part.vertices.len() as u32)
        });
        GpuObjectModel {
            parts,
            center: if count > 0.0 { sum / count } else { Vec3::ZERO },
            first_list,
            radius,
        }
    }

    /// Replaces the geometry of `model`'s first part and batch: a particle material's model,
    /// whose triangles are laid out again every frame (`ParticleSystem::FillVertexBuffer`).
    pub fn set_geometry(
        &mut self,
        gpu: &Gpu,
        model: &mut GpuObjectModel,
        vertices: &[ObjectVertex],
        indices: &[u32],
    ) {
        let Some(part) = model.parts.first_mut() else {
            return;
        };
        self.scratch_vertices.clear();
        self.scratch_vertices
            .extend(vertices.iter().map(|v| GpuVertex {
                position: v.position,
                normal: v.normal,
                color: v.color,
                uvs: v.uvs,
                tangents: v.tangents,
                bones: [0; 4],
                weights: [0.0; 4],
            }));
        let vertex_bytes: &[u8] = bytemuck::cast_slice(&self.scratch_vertices);
        let index_bytes: &[u8] = bytemuck::cast_slice(indices);
        // The first call replaces the upload's buffers, which lack `COPY_DST`; later calls refill
        // them until the live geometry outgrows their capacity.
        let first = !part.streamed;
        if first || vertex_bytes.len() as u64 > part.vertices.size() {
            part.vertices = stream_buffer(
                &gpu.device,
                "ac2 particle vertices",
                wgpu::BufferUsages::VERTEX,
                vertex_bytes.len() as u64,
            );
        }
        if first || index_bytes.len() as u64 > part.indices.size() {
            part.indices = stream_buffer(
                &gpu.device,
                "ac2 particle indices",
                wgpu::BufferUsages::INDEX,
                index_bytes.len() as u64,
            );
        }
        part.streamed = true;
        // The draw range below bounds the indices drawn, so a larger buffer's tail is never read.
        if !vertex_bytes.is_empty() {
            gpu.queue.write_buffer(&part.vertices, 0, vertex_bytes);
        }
        if !index_bytes.is_empty() {
            gpu.queue.write_buffer(&part.indices, 0, index_bytes);
        }
        if let Some(batch) = part.batches.first_mut() {
            batch.range = 0..indices.len() as u32;
        }
        let sum: Vec3 = vertices.iter().map(|v| Vec3::from_array(v.position)).sum();
        model.center = sum / vertices.len().max(1) as f32;
        model.radius = vertices
            .iter()
            .map(|v| Vec3::from_array(v.position).length())
            .fold(0.0, f32::max);
    }

    /// The module index for a pass's pixel shader, translating and compiling it on first use.
    fn program_id(&mut self, pass: &LayerPass) -> u32 {
        let Some(program) = &pass.pixel_program else {
            return 0;
        };
        let source = shade_function(program);
        if let Some(id) = self.program_ids.get(&source) {
            return *id;
        }
        let id = self.program_ids.len() as u32 + 1;
        self.pipelines.add_module(id, source.clone());
        self.program_ids.insert(source, id);
        id
    }

    fn stage_group(
        &mut self,
        gpu: &Gpu,
        pass: &LayerPass,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> wgpu::BindGroup {
        let mut views = Vec::with_capacity(4);
        let mut samplers = Vec::with_capacity(4);
        for index in 0..4 {
            let stage = pass.stages.get(index);
            // An untextured stage reads `D3DTA_TEXTURE` as white in the fixed-function cascade.
            // Retail binds no texture for an authored pixel shader's empty stage either
            // (`RenderDeviceD3D::SetStageTexture(stage, NULL)`), and the DX8 documentation leaves
            // that `tex` read undefined. The human body and clothing shaders add their stage 2 as
            // a centred detail (`add t1.rgb, t1, t2_bias`) and leave stage 2 empty, so it reads
            // the detail's neutral value: mid grey keeps their authored albedo (white washed the
            // skin out to chalk, black crushed it to brown).
            let view = match stage.and_then(|s| s.texture) {
                Some(did) => {
                    self.textures
                        .get(&gpu.device, &gpu.queue, TextureKey::Texture(did), load)
                }
                None if pass.pixel_program.is_some() => self.textures.neutral().clone(),
                None => self.textures.white().clone(),
            };
            views.push(view);
            let key = sampler_key(stage);
            let sampler = self
                .samplers
                .entry(key)
                .or_insert_with(|| key.create(&gpu.device))
                .clone();
            samplers.push(sampler);
        }
        let mut entries = Vec::with_capacity(8);
        for (binding, view) in views.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        for (index, sampler) in samplers.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: 4 + index as u32,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
        }
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 object stages"),
            layout: &self.texture_layout,
            entries: &entries,
        })
    }
}
