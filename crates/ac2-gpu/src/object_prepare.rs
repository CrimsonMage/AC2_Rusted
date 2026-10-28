//! The per-frame prepare: camera, lights and fog uniforms, the draws laid out for each group.

use super::*;

impl ObjectRenderer {
    /// Writes the frame's camera, lights and fog, re-evaluates animated layers at client time
    /// `time`, and lays out this frame's draws: layers that write depth in instance order, then the
    /// rest back to front.
    /// Writes the frame's camera, lights and fog, re-evaluates animated or tinted layers at
    /// client time `time`, and lays out each group's draws. A group in draw order keeps its
    /// instances' order (the sky); otherwise layers that write depth go first in instance order
    /// and the rest back to front.
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
        for (index, group) in groups.iter().enumerate() {
            let environment = group.environment.unwrap_or(environment);
            self.write_frame(&gpu.queue, index, camera, environment, aspect);
        }
        self.scratch_entries.clear();
        self.scratch_bones.clear();
        self.groups.clear();
        for group in groups {
            let start = self.scratch_entries.len();
            self.scratch_opaque.clear();
            self.scratch_translucent.clear();
            let backdrop = group.sky.then(|| sky_backdrop(group.instances)).flatten();
            for (instance_index, instance) in group.instances.iter().enumerate() {
                let center = instance.world.transform_point3(instance.model.center);
                let distance = center.distance_squared(camera.eye);
                let base = self.scratch_bones.len() as u32;
                if let Some(pose) = instance.pose {
                    self.scratch_bones
                        .extend(pose.palette.iter().map(Mat4::to_cols_array_2d));
                }
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
                    let model = instance.world * transform;
                    for (batch_index, batch) in part.batches.iter().enumerate() {
                        let recomputed = (batch.material.animated() || instance.tint != [1.0; 4])
                            .then(|| batch.material.passes(instance.tint, time));
                        for (pass_index, gpu_pass) in batch.passes.iter().enumerate() {
                            let pass = recomputed
                                .as_ref()
                                .map_or(&gpu_pass.pass, |p| &p[pass_index]);
                            let mut key = pipeline_key(pass, gpu_pass.program);
                            let role = SkyRole::of(
                                pass,
                                backdrop == Some(instance_index),
                                instance.model.radius,
                            );
                            if group.sky {
                                key = sky_depth(key);
                                if role == SkyRole::Veil {
                                    key = sky_veil(key);
                                }
                            }
                            let draw = QueuedDraw {
                                key,
                                deferred: false,
                                offset: 0,
                                instance: instance_index,
                                part: part_index,
                                batch: batch_index,
                                layer: pass_index,
                            };
                            let mut uniform = draw_uniform(model, pass);
                            if group.sky {
                                mark_sky(&mut uniform, pass, role);
                            }
                            if let Some(normal) = instance.frill_normal {
                                uniform.frill = normal.extend(1.0).to_array();
                                uniform.frill_shade = [instance.frill_shade, 0.0, 0.0, 0.0];
                            }
                            set_point_lights(&mut uniform, instance.point_lights);
                            if let Some(first) = skin {
                                uniform.skin = [base + first, 1, 0, 0];
                            }
                            let entry = (uniform, draw);
                            if group.ordered || pass.depth_write {
                                self.scratch_opaque.push(entry);
                            } else {
                                self.scratch_translucent.push((distance, entry));
                            }
                        }
                    }
                }
            }
            self.scratch_translucent.sort_by(|a, b| b.0.total_cmp(&a.0));
            self.scratch_entries.append(&mut self.scratch_opaque);
            self.scratch_entries
                .extend(self.scratch_translucent.drain(..).map(|(_, entry)| entry));
            let end = self.scratch_entries.len();
            self.groups.push(start..end);
        }
        let needed = DRAW_STRIDE * self.scratch_entries.len().max(1) as u64;
        if needed > self.draw_capacity {
            let capacity = needed.next_power_of_two();
            (self.draw_buffer, self.draw_group) =
                draw_storage(&gpu.device, &self.draw_layout, capacity);
            self.draw_capacity = capacity;
        }
        // Zeroed, as the padding between draw uniforms must be.
        self.scratch_bytes.clear();
        self.scratch_bytes.resize(needed as usize, 0);
        self.queue.clear();
        for (index, (uniform, draw)) in self.scratch_entries.iter().enumerate() {
            let offset = index as u64 * DRAW_STRIDE;
            self.scratch_bytes
                [offset as usize..offset as usize + std::mem::size_of::<DrawUniform>()]
                .copy_from_slice(bytemuck::bytes_of(uniform));
            let mut draw = *draw;
            self.pipelines.request(draw.key);
            draw.offset = offset as u32;
            self.queue.push(draw);
        }
        self.pipelines.collect();
        if self.stream_pipelines {
            self.defer_unready_instances();
        } else {
            self.pipelines.wait_all();
        }
        gpu.queue
            .write_buffer(&self.draw_buffer, 0, &self.scratch_bytes);
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

    /// Leaves out every instance of the frame with a draw whose pipeline is not compiled yet, so
    /// a model appears whole once its pipelines are in (a frame or two after its upload).
    fn defer_unready_instances(&mut self) {
        let mut waiting = HashSet::new();
        for (group, range) in self.groups.iter().enumerate() {
            for draw in &self.queue[range.clone()] {
                if !self.pipelines.is_ready(&draw.key) {
                    waiting.insert((group, draw.instance));
                }
            }
        }
        if waiting.is_empty() {
            return;
        }
        for (group, range) in self.groups.iter().enumerate() {
            for draw in &mut self.queue[range.clone()] {
                draw.deferred = waiting.contains(&(group, draw.instance));
            }
        }
    }

    /// Leave out (instead of waiting for) instances whose pipelines are still compiling: the
    /// running client, which must not stall a frame on a driver compile.
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.stream_pipelines = stream;
    }

    /// Draw calls the last `prepare` laid out to record (deferred instances left out).
    pub fn draw_count(&self) -> usize {
        self.queue.iter().filter(|draw| !draw.deferred).count()
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
            to_viewer: (-forward).extend(0.0).to_array(),
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
