//! The far landscape (`LScapeTerrainMesh::InitFarMesh`, `BuildFarTerrainMesh`): a coarse terrain
//! and water mesh over the blocks past the near ring, built around the centre block.
//!
//! `InitFarMesh` lays out six chunks of blocks relative to the centre: chunk 0 the centre block,
//! chunk 1 the near square `[-n, n]^2`, and when the far radius `f` is positive four chunks that
//! tile the ring out to `n + f` (north, east, south, west). Blocks of chunks 0 and 1 keep every
//! vertex; a ring block at Chebyshev distance `d` samples every `step`-th landscape vertex, step 1
//! at `d - n == 1`, else `2^(((d - n - 2) >> 1) + 1)`, capped at the block's polygon count. Every
//! block also gets one water quad. Adjacent blocks of a chunk are linked by an edge
//! (`LScapeFarBlockEdge`), and `BuildFarTerrainMesh` fills the crack along an edge whose blocks
//! differ in detail with one triangle per extra segment: two coarse vertices and the fine
//! vertex between them. Each terrain quad takes a random diagonal (`Random::RollDice(0, 1)`).
//!
//! `LScapeTerrainMesh::DrawTerrain` draws chunks 2 onward after the near mesh (chunk 1 too when
//! the near mesh is not drawn), and `DrawWaterMesh` the same chunks of the far water.

use std::ops::Range;
use std::sync::Arc;

use ac2_assets::load_ac2_material;
use ac2_core::{CellId, Random};
use ac2_db::{ResourceResult, ResourceStore};
use ac2_math::{Mat4, Vec3};
use ac2_world::{Ac2FarTerrain, Ac2WorldSource};

use crate::object::{
    layer_drawn, ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectVertex, RenderSettings,
};

/// `LScapeManager::Init`'s near radius in blocks.
pub const NEAR_RADIUS: i32 = 1;
/// The first chunk `DrawTerrain` draws when the near mesh is drawn.
pub const FIRST_FAR_CHUNK: usize = 2;
/// `BuildFarTerrainMesh`'s height for the water quad of a block without water.
const DRY_WATER_HEIGHT: f32 = -100.0;

/// `SmartBox::InitPreferences`: the far radius for `LandscapeDrawDistance` 0-4 (4 otherwise).
pub fn far_radius_for_draw_distance(preference: u32) -> i32 {
    match preference {
        0 => 0,
        1 => 2,
        3 => 6,
        4 => 8,
        _ => 4,
    }
}

/// `LScapeFarBlock`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FarBlock {
    /// Block offset from the centre block.
    x: i32,
    y: i32,
    /// Quads per side (`m_nNumEdgeQuads`).
    quads: u32,
    /// Vertices per side (`m_nNumEdgeVertices`).
    side: u32,
    /// Landscape vertices per far vertex (`m_nLSVertexStep`).
    step: u32,
    edge_length: f32,
    quad_length: f32,
    base: u32,
    /// Edge indices east, south, west, north (`m_nEastEdgeIndex` ...).
    edges: [Option<usize>; 4],
}

impl FarBlock {
    fn new(x: i32, y: i32, step: u32, polygons: u32, block_length: f32) -> Self {
        let quads = polygons / step;
        Self {
            x,
            y,
            quads,
            side: quads + 1,
            step,
            edge_length: block_length,
            quad_length: block_length / quads as f32,
            base: 0,
            edges: [None; 4],
        }
    }

    fn vertex(&self, x: u32, y: u32) -> u32 {
        self.side * y + self.base + x
    }
}

/// `LScapeFarChunk`.
#[derive(Debug, Clone, PartialEq)]
struct FarChunk {
    blocks: Vec<FarBlock>,
    water: Vec<FarBlock>,
    /// `LScapeFarBlockEdge` block index pairs.
    edges: Vec<[usize; 2]>,
}

/// The far mesh layout and its index lists, built once (`InitFarMesh` and the first
/// `BuildFarTerrainMesh`); the vertices follow the centre block.
#[derive(Debug, Clone, PartialEq)]
pub struct FarTerrain {
    chunks: Vec<FarChunk>,
    vertex_count: u32,
    water_vertex_count: u32,
    indices: Vec<u32>,
    chunk_ranges: Vec<Range<u32>>,
    water_indices: Vec<u32>,
    water_ranges: Vec<Range<u32>>,
}

/// One far terrain vertex in world AC axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FarVertex {
    pub position: [f32; 3],
    /// `CLandBlockData::m_vNormals`.
    pub normal: [f32; 3],
    /// `SurfaceInfo::far_vertex_color` as the `D3DCOLOR` the vertex's specular slot holds;
    /// `IlluminateVertices` lights the diffuse from it like the near mesh.
    pub color: [f32; 4],
    /// Centre-relative position times the far texture scale (`m_fFarTextureScale`).
    pub uv: [f32; 2],
}

/// The far mesh vertices around one centre block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FarVertices {
    pub terrain: Vec<FarVertex>,
    water: Vec<ObjectVertex>,
    /// Whether each water block holds water (its quad is collapsed at -100 otherwise).
    wet: Vec<bool>,
}

impl FarTerrain {
    /// Lays out the far mesh for `near_radius` and `far_radius` blocks; `polygons` is the
    /// landscape polygons per block side (`vertex_per_cell * lblock_side`), `block_length` in
    /// metres. `random` supplies the quad diagonals.
    pub fn new(
        near_radius: i32,
        far_radius: i32,
        polygons: u32,
        block_length: f32,
        random: &mut Random,
    ) -> Self {
        let n = near_radius;
        let mut bounds = vec![(0, 0, 0, 0), (-n, -n, n, n)];
        if far_radius > 0 {
            let f = far_radius;
            bounds.extend([
                (-(n + f), n + 1, n, n + f),
                (n + 1, -n, n + f, n + f),
                (-n, -(n + f), n + f, -1 - n),
                (-(n + f), -(n + f), -1 - n, n),
            ]);
        }
        let (mut vertex_count, mut water_vertex_count) = (0, 0);
        let mut chunks = Vec::with_capacity(bounds.len());
        for (index, &(min_x, min_y, max_x, max_y)) in bounds.iter().enumerate() {
            let width = (max_x - min_x + 1) as usize;
            let mut chunk = FarChunk {
                blocks: Vec::new(),
                water: Vec::new(),
                edges: Vec::new(),
            };
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let step = if index >= FIRST_FAR_CHUNK {
                        lod_step(x.abs().max(y.abs()) - n).min(polygons)
                    } else {
                        1
                    };
                    let mut block = FarBlock::new(x, y, step, polygons, block_length);
                    block.base = vertex_count;
                    vertex_count += block.side * block.side;
                    chunk.blocks.push(block);
                    let mut water = FarBlock::new(x, y, polygons, polygons, block_length);
                    water.base = water_vertex_count;
                    water_vertex_count += water.side * water.side;
                    chunk.water.push(water);
                }
            }
            // East, south, west, north neighbours within the chunk, each pair recorded once.
            for i in 0..chunk.blocks.len() {
                let FarBlock { x, y, .. } = chunk.blocks[i];
                let neighbours = [
                    (x < max_x).then(|| i + 1),
                    (y > min_y).then(|| i - width),
                    (x > min_x).then(|| i - 1),
                    (y < max_y).then(|| i + width),
                ];
                for (side, other) in neighbours.into_iter().enumerate() {
                    let Some(other) = other else { continue };
                    let edge = match chunk
                        .edges
                        .iter()
                        .position(|e| *e == [i, other] || *e == [other, i])
                    {
                        Some(edge) => edge,
                        None => {
                            chunk.edges.push([i, other]);
                            chunk.edges.len() - 1
                        }
                    };
                    chunk.blocks[i].edges[side] = Some(edge);
                }
            }
            chunks.push(chunk);
        }
        let mut far = Self {
            chunks,
            vertex_count,
            water_vertex_count,
            indices: Vec::new(),
            chunk_ranges: Vec::new(),
            water_indices: Vec::new(),
            water_ranges: Vec::new(),
        };
        far.build_indices(random);
        far
    }

    /// `BuildFarTerrainMesh`'s index lists, one fragment per chunk.
    fn build_indices(&mut self, random: &mut Random) {
        for chunk in &self.chunks {
            let start = self.indices.len() as u32;
            for block in &chunk.blocks {
                for row in 0..block.quads {
                    for col in 0..block.quads {
                        let a = block.vertex(col, row);
                        let b = block.vertex(col, row + 1);
                        let diagonal = random.roll_dice(0, 1);
                        let quad = if diagonal == 0 {
                            [b, a, b + 1, a, a + 1, b + 1]
                        } else {
                            [b, a, a + 1, b, a + 1, b + 1]
                        };
                        self.indices.extend(quad);
                    }
                }
            }
            for (index, pair) in chunk.edges.iter().enumerate() {
                let [first, second] = pair.map(|block| chunk.blocks[block]);
                let count = first.quads.abs_diff(second.quads);
                if count == 0 {
                    continue;
                }
                let (fine, coarse) = if first.side < second.side {
                    (second, first)
                } else {
                    (first, second)
                };
                edge_triangles(&fine, &coarse, index, count, &mut self.indices);
            }
            self.chunk_ranges.push(start..self.indices.len() as u32);
        }
        for chunk in &self.chunks {
            let start = self.water_indices.len() as u32;
            for block in &chunk.water {
                let (a, b) = (block.vertex(0, 0), block.vertex(0, 1));
                self.water_indices.extend([b, a, a + 1, b, a + 1, b + 1]);
            }
            self.water_ranges
                .push(start..self.water_indices.len() as u32);
        }
    }

    /// The terrain triangle list; chunk `c` is `indices()[chunk_ranges()[c]]`.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn chunk_ranges(&self) -> &[Range<u32>] {
        &self.chunk_ranges
    }

    /// Each chunk's `m_BoundingBox` (`BuildFarTerrainMesh`'s `BBox::AdjustBBox` over the chunk's
    /// terrain vertices) as (min, max); a chunk without vertices is an empty box at the origin.
    pub fn chunk_bounds(&self, vertices: &FarVertices) -> Vec<(Vec3, Vec3)> {
        self.chunk_ranges
            .iter()
            .map(|range| {
                let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for &index in &self.indices[range.start as usize..range.end as usize] {
                    if let Some(vertex) = vertices.terrain.get(index as usize) {
                        let p = Vec3::from_array(vertex.position);
                        min = min.min(p);
                        max = max.max(p);
                    }
                }
                if min.x > max.x {
                    (Vec3::ZERO, Vec3::ZERO)
                } else {
                    (min, max)
                }
            })
            .collect()
    }

    /// `BuildFarTerrainMesh`'s vertex fill around `center` (block x, y): far-detail heights,
    /// normals and far colours of each block's landblock, flat white at height 0 where the block
    /// lies off the map or has no data; one water quad per block at its water height.
    pub fn vertices(
        &self,
        world: &Ac2WorldSource,
        center: (i32, i32),
        far: &Ac2FarTerrain,
    ) -> FarVertices {
        let mut out = FarVertices {
            terrain: Vec::with_capacity(self.vertex_count as usize),
            water: Vec::with_capacity(self.water_vertex_count as usize),
            wet: Vec::new(),
        };
        let block_length = world.block_size();
        let origin = [
            f64::from(center.0) * f64::from(block_length),
            f64::from(center.1) * f64::from(block_length),
        ];
        // Centre-relative f32 coordinate, as the vertex buffer stores it, then world space.
        let relative = |index: u32, length: f32, block: i32, edge: f32| {
            (f64::from(index) * f64::from(length) + f64::from(block) * f64::from(edge)) as f32
        };
        let absolute = |rel: f32, axis: usize| (f64::from(rel) + origin[axis]) as f32;
        for chunk in &self.chunks {
            for (block, water) in chunk.blocks.iter().zip(&chunk.water) {
                let (bx, by) = (center.0 + block.x, center.1 + block.y);
                // `LandDefs::land_width` / `land_length`: blocks 0-254.
                let on_map = (0..255).contains(&bx) && (0..255).contains(&by);
                let cell = CellId::new(((bx as u32) << 24) | ((by as u32) << 16) | 0xFFFF);
                let landblock = on_map
                    .then(|| world.load_landblock(cell).ok().flatten())
                    .flatten();
                let heights = landblock.as_ref().map(|l| l.height_grid());
                for y in 0..block.side {
                    for x in 0..block.side {
                        let rel = [
                            relative(x, block.quad_length, block.x, block.edge_length),
                            relative(y, block.quad_length, block.y, block.edge_length),
                        ];
                        let (sx, sy) = ((x * block.step) as usize, (y * block.step) as usize);
                        let sample = landblock.as_ref().zip(heights.as_ref()).and_then(
                            |(landblock, heights)| {
                                let height = heights[sx * landblock.vertex_side() + sy];
                                let normal = landblock.normal(sx, sy)?;
                                let [r, g, b, _] = landblock.surface_info(sx, sy)?.far_vertex_color;
                                Some((height, normal, d3dcolor([r, g, b, 1.0])))
                            },
                        );
                        let (height, normal, color) =
                            sample.unwrap_or((0.0, [0.0, 0.0, 1.0], [1.0; 4]));
                        out.terrain.push(FarVertex {
                            position: [absolute(rel[0], 0), absolute(rel[1], 1), height],
                            normal,
                            color,
                            uv: [far.texture_scale * rel[0], rel[1] * far.texture_scale],
                        });
                    }
                }
                let water_desc = landblock
                    .as_ref()
                    .and_then(|landblock| world.block_water(cell, landblock));
                // `m_fNearWaterSize / m_fWaterTiling`.
                let uv_scale = water_desc
                    .as_ref()
                    .map_or(1.0, |w| block_length * 0.25 / w.tiling);
                out.wet.push(water_desc.is_some());
                for y in 0..water.side {
                    for x in 0..water.side {
                        let (rel, z) = match &water_desc {
                            Some(w) => (
                                [
                                    relative(x, water.quad_length, water.x, water.edge_length),
                                    relative(y, water.quad_length, water.y, water.edge_length),
                                ],
                                w.height,
                            ),
                            None => (
                                [
                                    relative(0, 0.0, water.x, water.edge_length),
                                    relative(0, 0.0, water.y, water.edge_length),
                                ],
                                DRY_WATER_HEIGHT,
                            ),
                        };
                        let mut uvs = [[0.0; 2]; 4];
                        uvs[0] = [x as f32 * uv_scale, y as f32 * uv_scale];
                        out.water.push(ObjectVertex {
                            position: [absolute(rel[0], 0), absolute(rel[1], 1), z],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0; 4],
                            uvs,
                            tangents: [[0.0; 3]; 2],
                        });
                    }
                }
            }
        }
        out
    }

    /// The far water of chunks `FIRST_FAR_CHUNK` onward with the far water material
    /// (`InitFarMesh`'s `MaterialInstance::Get` of the surface info's water material), leaving
    /// out the collapsed quads of dry blocks. `None` when no drawn block holds water or the
    /// material has no layer this device draws.
    pub fn water_model(
        &self,
        store: &ResourceStore,
        far: &Ac2FarTerrain,
        vertices: &FarVertices,
        settings: &RenderSettings,
    ) -> ResourceResult<Option<ObjectModel>> {
        let mut indices = Vec::new();
        let mut block = 0;
        for (index, chunk) in self.chunks.iter().enumerate() {
            let range = self.water_ranges[index].clone();
            let quads = self.water_indices[range.start as usize..range.end as usize].chunks(6);
            for (quad, _) in quads.zip(&chunk.water) {
                if index >= FIRST_FAR_CHUNK && vertices.wet.get(block).copied().unwrap_or(false) {
                    indices.extend_from_slice(quad);
                }
                block += 1;
            }
        }
        if indices.is_empty() {
            return Ok(None);
        }
        let Some(material) = load_ac2_material(store, far.water_material)? else {
            return Ok(None);
        };
        let layers: Vec<_> = material
            .layers
            .into_iter()
            .filter(|layer| layer_drawn(layer, settings))
            .collect();
        if layers.is_empty() {
            return Ok(None);
        }
        Ok(Some(ObjectModel {
            visual: far.water_material,
            parts: vec![ObjectPart {
                mesh: far.water_material,
                transform: Mat4::IDENTITY,
                vertices: Arc::from(vertices.water.as_slice()),
                batches: vec![ObjectBatch {
                    indices: Arc::from(indices),
                    material: Arc::new(ObjectMaterial::new(layers)),
                }],
                slot: (0, 0),
                skin: None,
                mesh_type: 0,
            }],
            skeleton: None,
        }))
    }
}

/// `InitFarMesh`'s vertex step at `ring = d - n` blocks past the near square.
fn lod_step(ring: i32) -> u32 {
    if ring == 1 {
        1
    } else {
        1 << ((((ring - 2) as u32) >> 1) + 1)
    }
}

/// The crack fill along the edge `index` of `fine`: per coarse segment, its two vertices and the
/// fine vertex at its midpoint, walked by which side of `fine` the edge lies on.
fn edge_triangles(
    fine: &FarBlock,
    coarse: &FarBlock,
    index: usize,
    count: u32,
    out: &mut Vec<u32>,
) {
    let (vc, vf) = (coarse.side as i32, fine.side as i32);
    // Coarse start and step, fine start and step (x, y each).
    let walk = match fine.edges.iter().position(|e| *e == Some(index)) {
        Some(0) => ((0, 0), (0, 1), (vf - 1, 1), (0, 2)),
        Some(1) => ((0, vc - 1), (1, 0), (1, 0), (2, 0)),
        Some(2) => ((vc - 1, vc - 1), (0, -1), (0, vf - 2), (0, -2)),
        Some(3) => ((vc - 1, 0), (-1, 0), (vf - 2, vf - 1), (-2, 0)),
        _ => return,
    };
    let ((mut cx, mut cy), (dcx, dcy), (mut fx, mut fy), (dfx, dfy)) = walk;
    let index = |block: &FarBlock, x: i32, y: i32| block.vertex(x as u32, y as u32);
    for _ in 0..count {
        let first = index(coarse, cx, cy);
        cx += dcx;
        cy += dcy;
        out.extend([first, index(coarse, cx, cy), index(fine, fx, fy)]);
        fx += dfx;
        fy += dfy;
    }
}

/// `RGBAColor::GetColor32` then `SetColor32`: the far colour as its stored `D3DCOLOR`.
fn d3dcolor(color: [f32; 4]) -> [f32; 4] {
    color.map(|c| ((c * 255.0) as i32 as u32 & 0xFF) as f32 * 0.003_921_569)
}

#[cfg(test)]
#[path = "far_tests.rs"]
mod tests;
