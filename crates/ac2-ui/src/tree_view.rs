//! `UIElement_Tree`: items laid out as the nodes of a tree. In grid mode (`grid`) every item has
//! a cell (`gridx`, `gridy`) of `node_width` x `node_height` pixels, its parents named by
//! `Parent1x/y` and `Parent2x/y` (`GridLayoutUpdate`) or set directly (`SetItemsParent`); without
//! it the parents name their children (`child1`..`child3`, `StaticLayoutUpdate`). The paper is
//! one cell wider and taller than the largest cell (`CalculatePaperDimen`), never smaller than
//! the element; a vertical tree (`horizontal` unset) without `orient_high` grows upward from the
//! bottom of the paper. `UpdateLayout` moves the items to their cells less the view (`x_view`,
//! `y_view`) and collects a line from each parent to each child (`UIElement_Node::
//! GetChildLines`); `DrawDone` draws them after the items, straight or, with `straight_connect`,
//! as three segments bent `straight_away_pix` away (`SurfaceWindow::DrawLine`).
//!
//! The vertical scroll bar and its wheel are in `tree_scroll.rs`. Not modelled yet: the horizontal
//! bar and arrow buttons (`UpdateControls`) and the static layout's placement (static trees keep
//! the items where the layout put them). The selection (`SetSelectedItem`) is `tree_scroll.rs`'s
//! press.

use crate::element::{UiBox, UiTree};
use crate::layout::AttributeValue;
use crate::token::tokenize;

/// One item (`UIElement_Node`): its element, cell, parents and children (by element).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeNode {
    pub element: usize,
    pub grid: (i32, i32),
    pub parents: Vec<usize>,
    pub children: Vec<usize>,
}

/// A connection (`Lines`): parent point, child point and the parent's `node_away_x/y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeLine {
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub away: (i32, i32),
}

/// `UIElement_Tree`'s state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TreeState {
    /// `m_nodes`, in grid order.
    pub nodes: Vec<TreeNode>,
    pub lines: Vec<TreeLine>,
    pub paper: (i32, i32),
    /// `m_x_view_record`, `m_y_view_record`.
    pub view: (i32, i32),
    pub max_grid: (i32, i32),
    pub multiop: bool,
    /// `m_selected`: the selected item's element.
    pub selected: Option<usize>,
}

impl TreeState {
    /// `GetNodeInGrid`: the node at a cell, stopping at the first node past it on both axes.
    fn node_in_grid(&self, x: i32, y: i32) -> Option<&TreeNode> {
        for node in &self.nodes {
            if node.grid == (x, y) {
                return Some(node);
            }
            if x < node.grid.0 && y < node.grid.1 {
                return None;
            }
        }
        None
    }

    fn node_mut(&mut self, element: usize) -> Option<&mut TreeNode> {
        self.nodes.iter_mut().find(|n| n.element == element)
    }

    /// `UIElement_Node::AddParent` and the parent's `AddChild`, each once.
    fn link(&mut self, child: usize, parent: usize) {
        if let Some(node) = self.node_mut(child) {
            if !node.parents.contains(&parent) {
                node.parents.push(parent);
            }
        }
        if let Some(node) = self.node_mut(parent) {
            if !node.children.contains(&child) {
                node.children.push(child);
            }
        }
    }
}

fn int(tree: &UiTree, index: usize, name: &str) -> Option<i32> {
    match tree.el(index).attributes.get(&tokenize(name)) {
        Some(AttributeValue::Int(v)) => Some(*v),
        _ => None,
    }
}

fn flag(tree: &UiTree, index: usize, name: &str) -> bool {
    matches!(
        tree.el(index).attributes.get(&tokenize(name)),
        Some(AttributeValue::Bool(true))
    )
}

fn float(tree: &UiTree, index: usize, name: &str) -> Option<f32> {
    match tree.el(index).attributes.get(&tokenize(name)) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

impl UiTree {
    pub(crate) fn tree_state(&mut self, tree: usize) -> &mut TreeState {
        self.el_mut(tree).tree.get_or_insert_with(Default::default)
    }

    /// `UIElement_Tree::IsOccupied`.
    pub fn tree_is_occupied(&self, tree: usize, x: i32, y: i32) -> bool {
        self.tree_item(tree, x, y).is_some()
    }

    /// `UIElement_Tree::GetItem`: the item in a cell.
    pub fn tree_item(&self, tree: usize, x: i32, y: i32) -> Option<usize> {
        self.el(tree)
            .tree
            .as_ref()?
            .node_in_grid(x, y)
            .map(|n| n.element)
    }

    /// `UIElement_Tree::MultiOpStart`: layout waits for [`UiTree::tree_multi_op_complete`].
    pub fn tree_multi_op_start(&mut self, tree: usize) {
        self.tree_state(tree).multiop = true;
    }

    /// `UIElement_Tree::MultiOpComplete`.
    pub fn tree_multi_op_complete(&mut self, tree: usize) {
        self.tree_state(tree).multiop = false;
        self.tree_update_layout(tree);
    }

    /// `UIElement_Tree::CalculatePaperDimen`.
    fn tree_paper(&mut self, tree: usize, num_x: i32, num_y: i32) {
        let (w, h) = (self.el(tree).rect.width(), self.el(tree).rect.height());
        let nw = int(self, tree, "node_width").unwrap_or(0);
        let nh = int(self, tree, "node_height").unwrap_or(0);
        self.tree_state(tree).paper = (((num_x + 1) * nw).max(w), ((num_y + 1) * nh).max(h));
    }

    /// `UIElement_Tree::AddItem`: `item` becomes a child of the tree in cell (`x`, `y`) (grid
    /// mode; -1 keeps the item's own `gridx`/`gridy`), then every node takes its parents.
    pub fn tree_add_item(&mut self, tree: usize, item: usize, x: i32, y: i32) -> bool {
        let grid = flag(self, tree, "grid");
        self.set_parent(item, Some(tree));
        self.update_size_and_position(item);
        if grid && x != -1 && y != -1 {
            let attrs = &mut self.el_mut(item).attributes;
            attrs.insert(tokenize("gridx"), AttributeValue::Int(x));
            attrs.insert(tokenize("gridy"), AttributeValue::Int(y));
        }
        let node = TreeNode {
            element: item,
            grid: (0, 0),
            parents: Vec::new(),
            children: Vec::new(),
        };
        if grid {
            self.tree_add_grid_node(tree, node);
        } else {
            self.tree_state(tree).nodes.insert(0, node);
        }
        let elements: Vec<usize> = self
            .tree_state(tree)
            .nodes
            .iter()
            .map(|n| n.element)
            .collect();
        for element in elements {
            if grid {
                self.tree_grid_links(tree, element);
            } else {
                self.tree_static_links(tree, element);
            }
        }
        if !self.tree_state(tree).multiop {
            self.tree_update_layout(tree);
        }
        true
    }

    /// `UIElement_Tree::AddGridNode`: kept in grid order, unless its cell is taken.
    fn tree_add_grid_node(&mut self, tree: usize, mut node: TreeNode) -> bool {
        let (Some(x), Some(y)) = (
            int(self, node.element, "gridx"),
            int(self, node.element, "gridy"),
        ) else {
            return false;
        };
        let state = self.tree_state(tree);
        if state.node_in_grid(x, y).is_some() {
            return false;
        }
        let mut pos = 0;
        let mut max = state.max_grid;
        for n in &state.nodes {
            max = (max.0.max(n.grid.0), max.1.max(n.grid.1));
            if x < n.grid.0 && y < n.grid.1 {
                break;
            }
            pos += 1;
        }
        max = (max.0.max(x), max.1.max(y));
        state.max_grid = max;
        node.grid = (x, y);
        state.nodes.insert(pos, node);
        self.tree_paper(tree, max.0, max.1);
        true
    }

    /// `UIElement_Tree::GridLayoutUpdate`: the parents a node names by cell.
    fn tree_grid_links(&mut self, tree: usize, element: usize) {
        for (px, py) in [("Parent1x", "Parent1y"), ("Parent2x", "Parent2y")] {
            let (Some(x), Some(y)) = (int(self, element, px), int(self, element, py)) else {
                return;
            };
            let state = self.tree_state(tree);
            let parent = state
                .nodes
                .iter()
                .take_while(|n| !(x < n.grid.0 && y < n.grid.1))
                .find(|n| n.grid == (x, y))
                .map(|n| n.element);
            if let Some(parent) = parent {
                state.link(element, parent);
                return;
            }
        }
    }

    /// `UIElement_Tree::StaticLayoutUpdate`: the children a node names by id.
    fn tree_static_links(&mut self, tree: usize, element: usize) {
        for name in ["child1", "child2", "child3"] {
            let id = match self.el(element).attributes.get(&tokenize(name)) {
                Some(AttributeValue::Id(id)) if *id != 0 => *id,
                _ => return,
            };
            let elements: Vec<usize> = self
                .tree_state(tree)
                .nodes
                .iter()
                .map(|n| n.element)
                .collect();
            let child = elements.into_iter().find(|&e| self.el(e).id() == id);
            if let Some(child) = child {
                self.tree_state(tree).link(child, element);
            }
        }
    }

    /// `UIElement_Tree::SetItemsParent`: the node in cell (`x`, `y`) becomes `item`'s parent.
    pub fn tree_set_items_parent(&mut self, tree: usize, item: usize, x: i32, y: i32) -> bool {
        let state = self.tree_state(tree);
        if !state.nodes.iter().any(|n| n.element == item) {
            return false;
        }
        let Some(parent) = state.node_in_grid(x, y).map(|n| n.element) else {
            return false;
        };
        state.link(item, parent);
        if !state.multiop {
            self.tree_update_layout(tree);
        }
        true
    }

    /// `UIElement_Tree::FlushTree`: every item deleted, the view back to the origin.
    pub fn tree_flush(&mut self, tree: usize) {
        let state = self.tree_state(tree);
        state.nodes.clear();
        state.max_grid = (0, 0);
        state.view = (0, 0);
        for child in self.el(tree).children.clone() {
            self.set_parent(child, None);
            self.delete(child);
        }
        self.tree_update_layout(tree);
    }

    /// `UIElement_Tree::End`: the view at the paper's far corner.
    pub fn tree_end(&mut self, tree: usize) {
        let (w, h) = (self.el(tree).rect.width(), self.el(tree).rect.height());
        let state = self.tree_state(tree);
        state.view = ((state.paper.0 - w).max(0), (state.paper.1 - h).max(0));
        self.tree_update_layout(tree);
    }

    /// `UIElement_Tree::UpdateLayout`.
    pub(crate) fn tree_update_layout(&mut self, tree: usize) {
        let horizontal = flag(self, tree, "horizontal");
        let orient = flag(self, tree, "orient_high");
        if flag(self, tree, "grid") {
            let nw = int(self, tree, "node_width").unwrap_or(0);
            let nh = int(self, tree, "node_height").unwrap_or(0);
            let state = self.tree_state(tree).clone();
            for node in &state.nodes {
                let mut x = node.grid.0 * nw;
                if orient && horizontal {
                    x = state.paper.0 - nw - x;
                }
                let mut y = node.grid.1 * nh;
                if !orient && !horizontal {
                    y = state.paper.1 - nh - y;
                }
                self.move_to(node.element, x - state.view.0, y - state.view.1);
            }
        }
        let nodes = self.tree_state(tree).nodes.clone();
        let mut lines = Vec::new();
        for node in &nodes {
            self.child_lines(node, horizontal, orient, &mut lines);
        }
        self.tree_state(tree).lines = lines;
        self.changed = true;
    }

    /// `UIElement_Node::GetChildLines`: from the parent's connection point (the near edge's
    /// middle, or `node_parent_connect_x/y`) to each child's (`GetConnection`).
    fn child_lines(
        &self,
        node: &TreeNode,
        horizontal: bool,
        orient: bool,
        out: &mut Vec<TreeLine>,
    ) {
        let b = self.el(node.element).rect;
        let x = int(self, node.element, "node_parent_connect_x").unwrap_or(if !horizontal {
            (b.x0 + b.x1) / 2
        } else if orient {
            b.x0
        } else {
            b.x1
        });
        let y = int(self, node.element, "node_parent_connect_y").unwrap_or(if horizontal {
            (b.y0 + b.y1) / 2
        } else if orient {
            b.y1
        } else {
            b.y0
        });
        let away = (
            int(self, node.element, "node_away_x").unwrap_or(0),
            int(self, node.element, "node_away_y").unwrap_or(0),
        );
        for &child in &node.children {
            let c = self.el(child).rect;
            let cx = int(self, child, "node_child_connect_x").unwrap_or(if !horizontal {
                (c.x0 + c.x1) / 2
            } else if orient {
                c.x1
            } else {
                c.x0
            });
            let cy = int(self, child, "node_child_connect_y").unwrap_or(if horizontal {
                (c.y0 + c.y1) / 2
            } else if orient {
                c.y0
            } else {
                c.y1
            });
            out.push(TreeLine {
                from: (x, y),
                to: (cx, cy),
                away,
            });
        }
    }

    /// `UIElement_Tree::DrawDone`'s segments in tree coordinates, and the line colour
    /// (`RGBAColor::GetColor32`, alpha 1); none with `no_lines`.
    pub(crate) fn tree_segments(&self, tree: usize) -> (Vec<((i32, i32), (i32, i32))>, [u8; 4]) {
        let Some(state) = self.el(tree).tree.as_ref() else {
            return (Vec::new(), [0; 4]);
        };
        if state.lines.is_empty() || flag(self, tree, "no_lines") {
            return (Vec::new(), [0; 4]);
        }
        let channel = |name| (f64::from(float(self, tree, name).unwrap_or(0.0)) * 255.0) as u8;
        let color = [
            channel("line_color_red"),
            channel("line_color_green"),
            channel("line_color_blue"),
            255,
        ];
        let straight = flag(self, tree, "straight_connect");
        let away = int(self, tree, "straight_away_pix").unwrap_or(0);
        let horizontal = flag(self, tree, "horizontal");
        let orient = flag(self, tree, "orient_high");
        let mut segments = Vec::new();
        for line in &state.lines {
            if !straight {
                segments.push((line.from, line.to));
                continue;
            }
            let (x0, y0) = line.from;
            let bend = if !horizontal {
                let y = if orient {
                    line.away.1 + y0 + away
                } else {
                    y0 - line.away.1 - away
                };
                (line.away.0 + x0, y)
            } else if orient {
                (x0 - line.away.0 - away, line.away.1 + y0)
            } else {
                (line.away.0 + x0 + away, line.away.1 + y0)
            };
            let corner = if horizontal {
                (bend.0, line.to.1)
            } else {
                (line.to.0, bend.1)
            };
            segments.push(((x0, y0), bend));
            segments.push((bend, corner));
            segments.push((corner, line.to));
        }
        (segments, color)
    }
}

/// `SurfaceWindow::DrawLine` into an RGBA surface `width` wide, clipped to `clip` (surface
/// coordinates); the pixels are written, not blended.
pub(crate) fn draw_line(
    rgba: &mut [u8],
    width: usize,
    clip: UiBox,
    from: (i32, i32),
    to: (i32, i32),
    color: [u8; 4],
) {
    // The window is the clipped bounding box; the endpoints are relative to it.
    let bounds = UiBox::new(
        from.0.min(to.0),
        from.1.min(to.1),
        from.0.max(to.0),
        from.1.max(to.1),
    );
    let Some(window) = clip.intersection(&bounds) else {
        return;
    };
    // `SurfaceWindow::SurfaceWindow(x0, y0, x1, y1)`: the right and bottom edges exclusive.
    let (rect_w, rect_h) = (window.width(), window.height());
    let (mut x, mut y) = (from.0 - window.x0, from.1 - window.y0);
    let (x1, y1) = (to.0 - window.x0, to.1 - window.y0);
    let (dx, dy) = ((x1 - x).abs(), (y1 - y).abs());
    let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
    let mut err = dx - dy;
    let mut plot = |x: i32, y: i32| {
        if x >= 0 && y >= 0 && x < rect_w && y < rect_h {
            let (px, py) = ((window.x0 + x) as usize, (window.y0 + y) as usize);
            let i = (py * width + px) * 4;
            if let Some(p) = rgba.get_mut(i..i + 4) {
                p.copy_from_slice(&color);
            }
        }
    };
    if dx < dy {
        let mut count = dy;
        loop {
            plot(x, y);
            if err >= 0 {
                x += sx;
                err -= dy;
                if err >= 0 {
                    continue;
                }
            }
            if count == 0 {
                return;
            }
            y += sy;
            err += dx;
            count -= 1;
        }
    } else {
        let mut count = dx;
        loop {
            plot(x, y);
            if err >= 0 {
                if count == 0 {
                    return;
                }
                count -= 1;
                x += sx;
                err -= dy;
                if err < 0 {
                    y += sy;
                    err += dx;
                }
            } else {
                y += sy;
                err += dx;
            }
        }
    }
}
