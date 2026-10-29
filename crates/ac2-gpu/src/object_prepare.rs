//! The per-frame prepare: camera, lights and fog uniforms, the draws laid out for each group.

use super::*;

impl ObjectRenderer {
    /// Writes the frame's camera, lights and fog, re-evaluates animated or tinted layers at
    /// client time `time`, and lays out each group's draws. A group in draw order keeps its
    /// instances' order (the sky); otherwise layers that write depth go first and the rest back to
    /// front. Consecutive layers of one model's part with the same uniform draw as one instanced
    /// draw; to make them consecutive, a group's depth-writing layers are grouped by model, part,
    /// batch and layer. Each instance keeps its own layer order; only the order between objects
    /// changes, which the depth test hides except where partly transparent texels of two objects
    /// overlap (one level of 255 on a few dozen pixels of a 400-figure crowd). That order was the
    /// object list's, not a retail one: retail sorts only translucent layers, as here.
    pub fn prepare(
        &mut self,
        gpu: &Gpu,
        camera: &Camera,
        environment: &ObjectEnvironment,
        aspect: f32,
        time: f64,
        groups: &[DrawGroup<'_>],
    ) {
        self.time = time;
        assert!(
            groups.len() <= MAX_GROUPS,
            "at most {MAX_GROUPS} draw groups"
        );
        // The sky groups draw camera-relative: their objects ride the viewer (`LScapeSky::
        // SetViewerPosition`), and taking the eye off in f32 at AC2's world coordinates (tens of
        // kilometres) moved each vertex by millimetres that changed with every step of the eye.
        let sky_camera = crate::sky::camera_at_origin(camera);
        for (index, group) in groups.iter().enumerate() {
            let environment = group.environment.unwrap_or(environment);
            let camera = if group.sky { &sky_camera } else { camera };
            self.write_frame(&gpu.queue, index, camera, environment, aspect);
        }
        self.scratch_entries.clear();
        self.scratch_bones.clear();
        self.groups.clear();
        for group in groups {
            let start = self.scratch_entries.len();
            // A sky group's instances are laid out with the eye taken off their origins.
            let shift = if group.sky { camera.eye } else { Vec3::ZERO };
            self.lay_out_group(group, camera.eye - shift, shift, time);
            self.groups.push(start..self.scratch_entries.len());
        }
        for entry in &self.scratch_entries {
            self.pipelines.request(entry.draw.key);
        }
        self.pipelines.collect();
        self.scratch_waiting.clear();
        if self.stream_pipelines {
            self.find_unready_instances();
        } else {
            self.pipelines.wait_all();
        }
        self.merge_draws();
        self.upload_draws(gpu);
    }

    /// Lays out `group`'s layers into `scratch_entries`: depth-writing layers (every layer of a
    /// group in draw order) first, the rest back to front from `eye`.
    /// `shift` is taken off every instance's origin (the sky's camera-relative draw; zero
    /// otherwise), and `eye` is in the same shifted space.
    fn lay_out_group(&mut self, group: &DrawGroup<'_>, eye: Vec3, shift: Vec3, time: f64) {
        self.scratch_opaque.clear();
        self.scratch_frills.clear();
        self.scratch_translucent.clear();
        let backdrop = group.sky.then(|| sky_backdrop(group.instances)).flatten();
        for (instance_index, instance) in group.instances.iter().enumerate() {
            let mut world = instance.world;
            world.w_axis -= shift.extend(0.0);
            let center = world.transform_point3(instance.model.center);
            let distance = center.distance_squared(eye);
            let base = self.scratch_bones.len() as u32;
            if let Some(pose) = instance.pose {
                self.scratch_bones
                    .extend(pose.palette.iter().map(Mat4::to_cols_array_2d));
            }
            let model_address = std::ptr::from_ref(instance.model).addr();
            for (part_index, part) in instance.model.parts.iter().enumerate() {
                let (transform, skin) = match instance.pose {
                    Some(pose) => (
                        pose.parts
                            .get(part_index)
                            .copied()
                            .unwrap_or(part.transform),
                        pose.skins
                            .get(part_index)
                            .copied()
                            .flatten()
                            .filter(|_| part.weighted),
                    ),
                    None => (part.transform, None),
                };
                let mut data = instance_data(world * transform);
                if let Some(normal) = instance.frill_normal {
                    data.frill = normal.extend(1.0).to_array();
                    let [near, far, hash] = instance.grass;
                    data.frill_shade = [instance.frill_shade, near, far, hash];
                }
                if let Some(first) = skin {
                    data.skin = [base + first, 1, 0, 0];
                }
                // `CBaseMesh::DrawMesh`: the degrade's batches only. Batch indices are the part's,
                // so instances on different degrades never merge into one draw.
                let drawn = PartDegrades::drawn(
                    part.degrades.as_deref(),
                    &instance.detail,
                    part.batches.len(),
                );
                for (batch_index, batch) in part
                    .batches
                    .iter()
                    .enumerate()
                    .take(drawn.end)
                    .skip(drawn.start)
                {
                    let recomputed =
                        (batch.material.animated() || instance.tint != [1.0; 4]).then(|| {
                            batch.material.passes_forced(
                                instance.tint,
                                time,
                                instance.detail.force_disable_alpha_test,
                            )
                        });
                    for (pass_index, gpu_pass) in batch.passes.iter().enumerate() {
                        let pass = recomputed
                            .as_ref()
                            .map_or(&gpu_pass.pass, |p| &p[pass_index]);
                        let mut key = pipeline_key(pass, gpu_pass.program);
                        let role = SkyRole::of(
                            pass,
                            backdrop == Some(instance_index),
                            instance.model.radius,
                            instance.model.center,
                        );
                        if group.sky {
                            key = sky_depth(key);
                            if role == SkyRole::Veil {
                                key = sky_veil(key);
                            }
                        }
                        let mut uniform = match &recomputed {
                            Some(_) => draw_uniform(pass),
                            None => gpu_pass.uniform,
                        };
                        if group.sky {
                            mark_sky(&mut uniform, pass, role);
                        }
                        set_point_lights(&mut uniform, instance.point_lights);
                        let entry = DrawEntry {
                            uniform,
                            data,
                            draw: QueuedDraw {
                                key,
                                offset: 0,
                                instance: instance_index,
                                part: part_index,
                                batch: batch_index,
                                layer: pass_index,
                                first_instance: 0,
                                instances: 1,
                            },
                            model: model_address,
                        };
                        if group.ordered || pass.depth_write {
                            if instance.frill_normal.is_some() && !group.ordered {
                                self.scratch_frills.push(entry);
                            } else {
                                self.scratch_opaque.push(entry);
                            }
                        } else {
                            self.scratch_translucent.push((distance, entry));
                        }
                    }
                }
            }
        }
        if !group.ordered && !group.sky {
            self.group_depth_layers();
        } else {
            self.scratch_entries.append(&mut self.scratch_opaque);
        }
        // `WorldRender::Render` draws the frills (`m_bRenderFrills`) after every opaque object
        // and before the translucent ones. The grass blends with depth writes, so a blade drawn
        // first in front of a body left the body failing the depth test under the blade's soft
        // edge: ragged holes in the body where it walked through grass.
        self.scratch_entries.append(&mut self.scratch_frills);
        self.scratch_translucent.sort_by(|a, b| b.0.total_cmp(&a.0));
        self.scratch_entries
            .extend(self.scratch_translucent.drain(..).map(|(_, entry)| entry));
    }

    /// Appends the group's depth-writing layers to `scratch_entries` ordered by the model's first
    /// appearance, part, batch and layer, instance order within each (a stable sort of small keys,
    /// so the entries themselves move once).
    fn group_depth_layers(&mut self) {
        self.scratch_model_ranks.clear();
        self.scratch_order.clear();
        for (index, entry) in self.scratch_opaque.iter().enumerate() {
            let next = self.scratch_model_ranks.len() as u32;
            let rank = *self.scratch_model_ranks.entry(entry.model).or_insert(next);
            let draw = &entry.draw;
            self.scratch_order
                .push(((rank, draw.part, draw.batch, draw.layer), index as u32));
        }
        self.scratch_order.sort_by_key(|(key, _)| *key);
        let opaque = &self.scratch_opaque;
        self.scratch_entries.extend(
            self.scratch_order
                .iter()
                .map(|&(_, index)| opaque[index as usize]),
        );
        self.scratch_opaque.clear();
    }

    /// Marks every (group, instance) with a layer whose pipeline is not compiled yet, so a model
    /// appears whole once its pipelines are in (a frame or two after its upload).
    fn find_unready_instances(&mut self) {
        for (group, range) in self.groups.iter().enumerate() {
            for entry in &self.scratch_entries[range.clone()] {
                if !self.pipelines.is_ready(&entry.draw.key) {
                    self.scratch_waiting.insert((group, entry.draw.instance));
                }
            }
        }
    }

    /// Turns the laid-out entries into the queue: each entry's instance block in order, and a
    /// new draw (with its uniform) unless the entry continues the last draw: same pipeline, the
    /// same part, batch and layer of the same model, and the same uniform.
    fn merge_draws(&mut self) {
        self.queue.clear();
        self.scratch_instances.clear();
        self.scratch_bytes.clear();
        self.scratch_last_uniform = None;
        let mut last_model = 0;
        for (group, range) in self.groups.iter_mut().enumerate() {
            let start = self.queue.len();
            for entry in &self.scratch_entries[range.clone()] {
                if self.scratch_waiting.contains(&(group, entry.draw.instance)) {
                    continue;
                }
                let slot = self.scratch_instances.len() as u32;
                self.scratch_instances.push(entry.data);
                let continues = self.queue.len() > start
                    && self.queue.last().is_some_and(|last| {
                        last.key == entry.draw.key
                            && last_model == entry.model
                            && (last.part, last.batch, last.layer)
                                == (entry.draw.part, entry.draw.batch, entry.draw.layer)
                    })
                    && self.scratch_last_uniform.as_ref().is_some_and(|uniform| {
                        bytemuck::bytes_of(uniform) == bytemuck::bytes_of(&entry.uniform)
                    });
                if continues {
                    if let Some(last) = self.queue.last_mut() {
                        last.instances += 1;
                    }
                    continue;
                }
                // Zeroed, as the padding between draw uniforms must be.
                let offset = self.scratch_bytes.len();
                self.scratch_bytes.resize(offset + DRAW_STRIDE as usize, 0);
                self.scratch_bytes[offset..offset + std::mem::size_of::<DrawUniform>()]
                    .copy_from_slice(bytemuck::bytes_of(&entry.uniform));
                self.queue.push(QueuedDraw {
                    offset: offset as u32,
                    first_instance: slot,
                    instances: 1,
                    ..entry.draw
                });
                self.scratch_last_uniform = Some(entry.uniform);
                last_model = entry.model;
            }
            *range = start..self.queue.len();
        }
    }

    /// Writes the queue's uniforms, instance blocks and the frame's bone matrices, growing their
    /// buffers (to a power of two) when they no longer fit.
    fn upload_draws(&mut self, gpu: &Gpu) {
        let draws = DRAW_STRIDE * self.queue.len().max(1) as u64;
        let instances = INSTANCE_BYTES * self.scratch_instances.len().max(1) as u64;
        if draws > self.draw_capacity || instances > self.instance_capacity {
            self.draw_capacity = self.draw_capacity.max(draws.next_power_of_two());
            self.instance_capacity = self.instance_capacity.max(instances.next_power_of_two());
            self.draws = draw_storage(
                &gpu.device,
                &self.draw_layout,
                self.draw_capacity,
                self.instance_capacity,
            );
        }
        if !self.scratch_bytes.is_empty() {
            gpu.queue
                .write_buffer(&self.draws.draws, 0, &self.scratch_bytes);
        }
        if !self.scratch_instances.is_empty() {
            gpu.queue.write_buffer(
                &self.draws.instances,
                0,
                bytemuck::cast_slice(&self.scratch_instances),
            );
        }
        let needed = BONE_BYTES * self.scratch_bones.len().max(1) as u64;
        if needed > self.bone_capacity {
            let capacity = needed.next_power_of_two();
            (self.bone_buffer, self.bone_group) =
                bone_storage(&gpu.device, &self.bone_layout, capacity);
            self.bone_capacity = capacity;
        }
        if !self.scratch_bones.is_empty() {
            gpu.queue.write_buffer(
                &self.bone_buffer,
                0,
                bytemuck::cast_slice(&self.scratch_bones),
            );
        }
    }

    /// Leave out (instead of waiting for) instances whose pipelines are still compiling: the
    /// running client, which must not stall a frame on a driver compile.
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.stream_pipelines = stream;
    }

    /// Draw calls the last `prepare` laid out to record (deferred instances left out).
    pub fn draw_count(&self) -> usize {
        self.queue.len()
    }

    /// Whether pipelines are still compiling (a headless render waits for this).
    pub fn pipelines_busy(&self) -> bool {
        self.pipelines.busy()
    }

    fn write_frame(
        &self,
        queue: &wgpu::Queue,
        slot: usize,
        camera: &Camera,
        environment: &ObjectEnvironment,
        aspect: f32,
    ) {
        let lights = environment.lights;
        let (fog_color, (start, end), fog_on) = match environment.fog {
            Some((color, range)) => (color, range, 1.0),
            None => ([1.0; 3], (0.0, 1.0), 0.0),
        };
        let rgb = |c: [f32; 3]| [c[0], c[1], c[2], 1.0];
        let forward = (camera.target - camera.eye).normalize_or_zero();
        let (inv_dir, diffuse, specular) = match lights.sun {
            Some(sun) => (
                (-sun.direction).extend(1.0).to_array(),
                rgb(sun.diffuse),
                rgb(sun.specular),
            ),
            None => ([0.0; 4], [0.0; 4], [0.0; 4]),
        };
        let uniform = FrameUniform {
            view_proj: camera.view_projection(aspect).to_cols_array_2d(),
            view: ac2_math::camera::look_at_mat4(camera.eye, camera.target, Vec3::Z)
                .to_cols_array_2d(),
            eye: camera.eye.extend(1.0).to_array(),
            // w: the client time for the grass wind, wrapped to keep f32 precision.
            to_viewer: (-forward).extend((self.time % 3600.0) as f32).to_array(),
            sun_inv_dir: inv_dir,
            sun_diffuse: diffuse,
            sun_specular: specular,
            ambient: rgb(lights.ambient),
            fog_color: rgb(fog_color),
            fog: [start, end, 1.0 / (end - start).max(1e-3), fog_on],
            shading: FRAME_SHADING,
            shadow: self.shadow_matrix.to_cols_array_2d(),
        };
        queue.write_buffer(
            &self.frame_buffer,
            slot as u64 * FRAME_STRIDE,
            bytemuck::bytes_of(&uniform),
        );
    }
}
