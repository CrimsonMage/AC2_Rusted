//! The physics `BSPTREE` of a static mesh (`CStaticMesh::physics_bsp`) in its mesh space, with
//! the collision polygons its leaves name, built from the decoded `Ac2PhysicsMesh`.

mod node;
pub(crate) mod polygon;
mod tree;

pub(crate) use tree::ContactKind;

use ac2_assets::{Ac2PhysicsBspNode, Ac2PhysicsMesh};

use crate::vector::{Plane, Sphere, Vector};
pub use polygon::Polygon;

/// One `BSPNODE` (an internal node with its splitting plane) or `BSPLEAF`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Node {
    Internal {
        sphere: Sphere,
        plane: Plane,
        pos: Option<usize>,
        neg: Option<usize>,
    },
    Leaf {
        sphere: Sphere,
        solid: bool,
        polygons: Vec<usize>,
    },
}

/// A mesh's physics BSP: nodes (root first) and the polygons its leaves index.
#[derive(Debug, Clone, PartialEq)]
pub struct BspTree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) polygons: Vec<Polygon>,
}

impl BspTree {
    /// The physics tree of a decoded mesh; `None` when the mesh does not collide.
    pub fn from_mesh(mesh: &Ac2PhysicsMesh) -> Option<Self> {
        let root = mesh.bsp()?;
        let positions = mesh.positions();
        let polygons = mesh
            .polygons()
            .iter()
            .map(|polygon| {
                Polygon::new(
                    polygon
                        .vertex_ids()
                        .iter()
                        .map(|&id| Vector::from(positions[usize::from(id)]))
                        .collect(),
                )
            })
            .collect();
        let mut tree = Self {
            nodes: Vec::new(),
            polygons,
        };
        tree.push(root);
        Some(tree)
    }

    fn push(&mut self, node: &Ac2PhysicsBspNode) -> usize {
        let index = self.nodes.len();
        let bounds = node.bounding_sphere();
        let sphere = Sphere::new(Vector::from(bounds.origin), bounds.radius);
        if node.is_leaf() {
            self.nodes.push(Node::Leaf {
                sphere,
                solid: node.solid() != 0,
                polygons: node
                    .polygon_slots()
                    .iter()
                    .map(|&slot| usize::from(slot))
                    .collect(),
            });
            return index;
        }
        let split = node.splitting_plane();
        self.nodes.push(Node::Internal {
            sphere,
            plane: Plane::new(Vector::from(split.normal), split.d),
            pos: None,
            neg: None,
        });
        let pos = node.pos_node().map(|child| self.push(child));
        let neg = node.neg_node().map(|child| self.push(child));
        if let Node::Internal { pos: p, neg: n, .. } = &mut self.nodes[index] {
            *p = pos;
            *n = neg;
        }
        index
    }

    /// A tree over explicit polygons and nodes.
    #[cfg(test)]
    pub(crate) fn new(nodes: Vec<Node>, polygons: Vec<Polygon>) -> Self {
        Self { nodes, polygons }
    }
}
