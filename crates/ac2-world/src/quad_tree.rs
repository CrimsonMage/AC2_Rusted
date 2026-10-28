//! `CLandBlockQuadTree` (`QuadTree<ICPhysicsObj*>`): the 2D quad tree a landblock builds over its
//! static objects' bounding boxes, with per-node height ranges for frustum culling and the draw
//! gather of the nodes the last cull stamped.

use ac2_math::BBox;

/// `BoundingType`: how a box lies against the view frustum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundingType {
    Outside,
    Intersecting,
    EntirelyInside,
}

/// `LandBlockQuadNode`: objects straddling the node's centre lines (or all of a leaf's), up to
/// four children, the z range of everything below and the timestamp of the cull that last kept
/// the node.
#[derive(Debug, Clone)]
struct Node<T> {
    objects: Vec<(T, BBox)>,
    children: [Option<Box<Node<T>>>; 4],
    min_height: f32,
    max_height: f32,
    stamp: u32,
}

impl<T> Node<T> {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            children: [None, None, None, None],
            min_height: 0.0,
            max_height: 0.0,
            stamp: 0,
        }
    }
}

/// One entry of the draw gather (`PhysObjBrush`).
#[derive(Debug, Clone, PartialEq)]
pub struct PhysObjBrush<T> {
    pub cell_id: u32,
    pub object: T,
    pub distance: f32,
    pub opacity: f32,
}

/// `CLandBlockQuadTree`.
#[derive(Debug, Clone)]
pub struct LandBlockQuadTree<T> {
    cell_id: u32,
    min_size: f32,
    objects: Vec<(T, BBox)>,
    root_bbox: BBox,
    root_center: [f32; 2],
    root_size: f32,
    root: Option<Box<Node<T>>>,
}

/// `QuadTree::GetChildQuad`: the quadrant of `bbox` about `center` (bit 0 for +x, bit 1 for +y),
/// or `None` when the box crosses a centre line and stays in the node.
fn child_quad(center: [f32; 2], bbox: &BBox) -> Option<u32> {
    let mut quad = 0;
    let mut straddles = false;
    if center[0] <= bbox.min.x || bbox.max.x <= center[0] {
        if center[0] < bbox.min.x && center[0] < bbox.max.x {
            quad = 1;
        }
    } else {
        straddles = true;
    }
    if center[1] <= bbox.min.y || bbox.max.y <= center[1] {
        if center[1] < bbox.min.y && center[1] < bbox.max.y {
            quad |= 2;
        }
        if !straddles {
            return Some(quad);
        }
    }
    None
}

/// `QuadTree::GetChildCenter`: the centre of quadrant `child` of a square of `size` about
/// `center`.
fn child_center(center: [f32; 2], size: f32, child: u32) -> [f32; 2] {
    let half = size * 0.5;
    let mut corner = [center[0] - half, center[1] - half];
    if child & 1 != 0 {
        corner[0] += half;
    }
    if child & 2 != 0 {
        corner[1] += half;
    }
    [half * 0.5 + corner[0], half * 0.5 + corner[1]]
}

impl<T: Clone> LandBlockQuadTree<T> {
    /// `QuadTree::Initialize`: empties the tree and sets the smallest node size that still splits.
    pub fn new(cell_id: u32, min_size: f32) -> Self {
        Self {
            cell_id,
            min_size,
            objects: Vec::new(),
            root_bbox: BBox::for_adjustment(),
            root_center: [0.0; 2],
            root_size: 0.0,
            root: None,
        }
    }

    /// `QuadTree::Reset`: drops the tree and the added objects.
    pub fn reset(&mut self) {
        self.root = None;
        self.root_bbox = BBox::for_adjustment();
        self.objects.clear();
    }

    /// `QuadTree::AddObject`: queues an object and grows the root box over its box.
    pub fn add_object(&mut self, object: T, bbox: BBox) {
        self.root_bbox.expand_to_include_bbox(bbox);
        self.objects.push((object, bbox));
    }

    /// `CLandBlockQuadTree::Generate`: builds the tree over the queued objects, then the nodes'
    /// z ranges (`UpdateMinMax`). With no objects the root is an empty node and every extent 0.
    pub fn generate(&mut self) {
        if self.objects.is_empty() {
            self.root_bbox = BBox::default();
            self.root_center = [0.0; 2];
            self.root_size = 0.0;
        } else {
            let size = (self.root_bbox.max.x - self.root_bbox.min.x)
                .max(self.root_bbox.max.y - self.root_bbox.min.y);
            self.root_size = size;
            self.root_bbox.max.x = size + self.root_bbox.min.x;
            self.root_bbox.max.y = size + self.root_bbox.min.y;
            let center = self.root_bbox.center();
            self.root_center = [center.x, center.y];
        }
        let mut root = Node::new();
        self.do_generate(&mut root, self.root_center, self.root_size, &self.objects);
        update_min_max(&mut root);
        self.root = Some(Box::new(root));
    }

    /// `QuadTree::DoGenerate`.
    fn do_generate(&self, node: &mut Node<T>, center: [f32; 2], size: f32, objects: &[(T, BBox)]) {
        if objects.is_empty() {
            return;
        }
        if size >= self.min_size && objects.len() > 1 {
            let mut quads: [Vec<(T, BBox)>; 4] = Default::default();
            for entry in objects {
                match child_quad(center, &entry.1) {
                    None => node.objects.push(entry.clone()),
                    Some(quad) => quads[quad as usize].push(entry.clone()),
                }
            }
            for (index, sub) in quads.iter().enumerate() {
                if sub.is_empty() {
                    continue;
                }
                let mut child = Node::new();
                self.do_generate(
                    &mut child,
                    child_center(center, size, index as u32),
                    size * 0.5,
                    sub,
                );
                node.children[index] = Some(Box::new(child));
            }
        } else {
            node.objects = objects.to_vec();
        }
    }

    /// `CLandBlockQuadTree::Cull`: stamps the nodes inside the view. `relationship` is the
    /// frustum test of a box in the viewer's block frame; `block_offset` is
    /// `LandDefs::get_block_offset(viewer cell, this block)`.
    pub fn cull(
        &mut self,
        block_offset: [f32; 2],
        timestamp: u32,
        relationship: &mut dyn FnMut(&BBox) -> BoundingType,
    ) {
        let (center, size) = (self.root_center, self.root_size);
        if let Some(root) = self.root.as_deref_mut() {
            do_cull(
                root,
                block_offset,
                timestamp,
                center,
                size,
                true,
                relationship,
            );
        }
    }

    /// `CLandBlockQuadTree::Draw`: the objects of the nodes stamped `timestamp` for which
    /// `visible` holds, each with its `distance`, in tree order (a node's objects, then its
    /// children).
    pub fn draw(
        &self,
        timestamp: u32,
        opacity: f32,
        visible: &dyn Fn(&T) -> bool,
        distance: &dyn Fn(&T) -> f32,
    ) -> Vec<PhysObjBrush<T>> {
        let mut brushes = Vec::new();
        if let Some(root) = self.root.as_deref() {
            self.do_draw(root, timestamp, opacity, visible, distance, &mut brushes);
        }
        brushes
    }

    fn do_draw(
        &self,
        node: &Node<T>,
        timestamp: u32,
        opacity: f32,
        visible: &dyn Fn(&T) -> bool,
        distance: &dyn Fn(&T) -> f32,
        brushes: &mut Vec<PhysObjBrush<T>>,
    ) {
        if node.stamp != timestamp {
            return;
        }
        for (object, _) in &node.objects {
            if visible(object) {
                brushes.push(PhysObjBrush {
                    cell_id: self.cell_id,
                    object: object.clone(),
                    distance: distance(object),
                    opacity,
                });
            }
        }
        for child in node.children.iter().flatten() {
            self.do_draw(child, timestamp, opacity, visible, distance, brushes);
        }
    }

    /// The z range of the whole tree (`CLandBlockQuadTree`'s min and max heights).
    pub fn height_range(&self) -> (f32, f32) {
        self.root
            .as_deref()
            .map_or((0.0, 0.0), |root| (root.min_height, root.max_height))
    }

    /// The root box and its square size, as `Generate` left them.
    pub fn root_extent(&self) -> (BBox, f32) {
        (self.root_bbox, self.root_size)
    }
}

/// `CLandBlockQuadTree::UpdateMinMax`: each node's z range covers its objects and children.
/// (Retail reads uninitialised floats for a node with neither; this gives 0.)
fn update_min_max<T>(node: &mut Node<T>) {
    let mut range: Option<(f32, f32)> = None;
    let mut widen = |lo: f32, hi: f32| {
        range = Some(range.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
    };
    for child in node.children.iter_mut().flatten() {
        update_min_max(child);
        widen(child.min_height, child.max_height);
    }
    for (_, bbox) in &node.objects {
        widen(bbox.min.z, bbox.max.z);
    }
    let (lo, hi) = range.unwrap_or((0.0, 0.0));
    node.min_height = lo;
    node.max_height = hi;
}

/// `CLandBlockQuadTree::DoCull`.
fn do_cull<T>(
    node: &mut Node<T>,
    offset: [f32; 2],
    timestamp: u32,
    center: [f32; 2],
    size: f32,
    check: bool,
    relationship: &mut dyn FnMut(&BBox) -> BoundingType,
) {
    let half = size * 0.5;
    let mut relation = BoundingType::EntirelyInside;
    if check {
        let bbox = BBox::new(
            ac2_math::Vec3::new(
                offset[0] + (center[0] - half),
                offset[1] + (center[1] - half),
                node.min_height,
            ),
            ac2_math::Vec3::new(
                offset[0] + half + center[0],
                offset[1] + half + center[1],
                node.max_height,
            ),
        );
        relation = relationship(&bbox);
        if relation == BoundingType::Outside {
            return;
        }
    }
    node.stamp = timestamp;
    for (index, child) in node.children.iter_mut().enumerate() {
        if let Some(child) = child.as_deref_mut() {
            do_cull(
                child,
                offset,
                timestamp,
                child_center(center, size, index as u32),
                half,
                relation != BoundingType::EntirelyInside,
                relationship,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_math::Vec3;

    fn cube(x: f32, y: f32, z0: f32, z1: f32) -> BBox {
        BBox::new(Vec3::new(x, y, z0), Vec3::new(x + 1.0, y + 1.0, z1))
    }

    fn tree() -> LandBlockQuadTree<u32> {
        let mut tree = LandBlockQuadTree::new(0xA9B2_FFFF, 8.0);
        tree.add_object(1, cube(0.0, 0.0, 5.0, 9.0));
        tree.add_object(2, cube(100.0, 0.0, -3.0, 2.0));
        tree.add_object(3, cube(0.0, 100.0, 0.0, 1.0));
        tree.add_object(4, cube(99.5, 99.5, 0.0, 12.0));
        tree.generate();
        tree
    }

    #[test]
    fn root_is_a_square_over_the_objects_with_their_height_range() {
        let tree = tree();
        let (bbox, size) = tree.root_extent();
        assert_eq!(size, 101.0);
        assert_eq!(bbox.max.x - bbox.min.x, bbox.max.y - bbox.min.y);
        assert_eq!(tree.height_range(), (-3.0, 12.0));
    }

    #[test]
    fn child_quads_split_about_the_centre_and_straddlers_stay() {
        let c = [50.0, 50.0];
        assert_eq!(child_quad(c, &cube(0.0, 0.0, 0.0, 1.0)), Some(0));
        assert_eq!(child_quad(c, &cube(60.0, 0.0, 0.0, 1.0)), Some(1));
        assert_eq!(child_quad(c, &cube(0.0, 60.0, 0.0, 1.0)), Some(2));
        assert_eq!(child_quad(c, &cube(60.0, 60.0, 0.0, 1.0)), Some(3));
        assert_eq!(child_quad(c, &cube(49.5, 0.0, 0.0, 1.0)), None);
        assert_eq!(child_quad(c, &cube(0.0, 49.5, 0.0, 1.0)), None);
        assert_eq!(child_center([50.0, 50.0], 100.0, 3), [75.0, 75.0]);
        assert_eq!(child_center([50.0, 50.0], 100.0, 0), [25.0, 25.0]);
    }

    #[test]
    fn cull_stamps_only_the_nodes_the_view_keeps() {
        let mut tree = tree();
        // A view that only holds the south-west corner of the block.
        tree.cull([0.0, 0.0], 7, &mut |bbox| {
            if bbox.max.x < 50.0 && bbox.max.y < 50.0 {
                BoundingType::EntirelyInside
            } else if bbox.min.x > 50.0 || bbox.min.y > 50.0 {
                BoundingType::Outside
            } else {
                BoundingType::Intersecting
            }
        });
        let seen = tree.draw(7, 1.0, &|_| true, &|object| *object as f32);
        let ids: Vec<u32> = seen.iter().map(|b| b.object).collect();
        assert_eq!(ids, [1]);
        assert_eq!(seen[0].cell_id, 0xA9B2_FFFF);
        // A later timestamp finds nothing until the next cull.
        assert!(tree.draw(8, 1.0, &|_| true, &|_| 0.0).is_empty());
        // The visibility test filters the gather.
        assert!(tree.draw(7, 1.0, &|_| false, &|_| 0.0).is_empty());
    }

    #[test]
    fn an_empty_tree_has_zero_extents_and_nothing_to_draw() {
        let mut tree = LandBlockQuadTree::<u32>::new(0, 4.0);
        tree.generate();
        assert_eq!(tree.height_range(), (0.0, 0.0));
        tree.cull([0.0; 2], 1, &mut |_| BoundingType::EntirelyInside);
        assert!(tree.draw(1, 1.0, &|_| true, &|_| 0.0).is_empty());
        tree.add_object(1, cube(0.0, 0.0, 0.0, 1.0));
        tree.reset();
        tree.generate();
        assert!(tree.draw(1, 1.0, &|_| true, &|_| 0.0).is_empty());
    }
}
