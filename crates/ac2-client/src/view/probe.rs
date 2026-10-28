//! Where the HUD's elements are on the window, for drivers that feed the pointer the way the
//! window does (`UI_POINTER_X` / `UI_POINTER_Y` and the mouse buttons).

use ac2_ui::{tokenize, AttributeValue, UiTree};

use super::ClientView;

/// The first element below `index` (itself and its ancestors shown) whose id is `id` and for which `keep` holds.
fn find(
    tree: &UiTree,
    index: usize,
    id: u32,
    keep: &dyn Fn(&UiTree, usize) -> bool,
) -> Option<usize> {
    let element = tree.get(index)?;
    if !element.visible() {
        return None;
    }
    if element.id() == id && keep(tree, index) {
        return Some(index);
    }
    tree.children(index)
        .iter()
        .find_map(|&child| find(tree, child, id, keep))
}

impl ClientView {
    /// [`ClientView::render`] with no UI viewport scenes (the paper doll left out).
    pub fn render_without_scenes(
        &self,
        gpu: &ac2_gpu::Gpu,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let (overlay, roots) = self.hud.layers_for_scenes();
        let draw_world: &dyn Fn(&mut wgpu::RenderPass<'_>) = &|pass| self.renderer.draw(pass);
        let draw_portal: &dyn Fn(&mut wgpu::RenderPass<'_>) = &|pass| self.portal.draw(pass);
        crate::ui_scene::render_with_scenes(
            gpu,
            &self.presenter,
            color,
            depth,
            &self.hud.scale,
            [0.0; 3],
            overlay,
            roots,
            &[],
            self.world_drawn.then_some(draw_world),
            self.portal.drawn().then_some(draw_portal),
        );
    }

    /// The window pixel at the centre of the HUD element at `index` of the tree.
    fn window_center(&self, tree: &UiTree, index: usize) -> (i32, i32) {
        let b = tree.screen_box(index);
        let (x, y) = self
            .hud
            .scale
            .to_window(f64::from(b.x0 + b.x1) / 2.0, f64::from(b.y0 + b.y1) / 2.0);
        (x.round() as i32, y.round() as i32)
    }

    /// The window pixel at the centre of the first shown HUD element called `name`.
    pub fn ui_element_center(&self, name: &str) -> Option<(i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let id = tokenize(name);
        let index = tree
            .roots()
            .iter()
            .find_map(|&root| find(tree, root, id, &|_, _| true))?;
        Some(self.window_center(tree, index))
    }

    /// The HUD's element tree, read-only.
    pub fn ui_tree(&self) -> Option<&UiTree> {
        self.hud.hud.as_ref().map(|(tree, _)| tree)
    }

    /// The window rectangle (left, top, right, bottom) of the first shown HUD element `name`.
    pub fn ui_element_rect(&self, name: &str) -> Option<(i32, i32, i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let id = tokenize(name);
        let index = tree
            .roots()
            .iter()
            .find_map(|&root| find(tree, root, id, &|_, _| true))?;
        let b = tree.screen_box(index);
        let (x0, y0) = self.hud.scale.to_window(f64::from(b.x0), f64::from(b.y0));
        let (x1, y1) = self.hud.scale.to_window(f64::from(b.x1), f64::from(b.y1));
        Some((
            x0.round() as i32,
            y0.round() as i32,
            x1.round() as i32,
            y1.round() as i32,
        ))
    }

    /// The skills of the skill panel's nodes, in grid order.
    pub fn skill_nodes(&self) -> Vec<u32> {
        let Some((tree, _)) = self.hud.hud.as_ref() else {
            return Vec::new();
        };
        let Some(skill_tree) = tree
            .roots()
            .iter()
            .find_map(|&root| find(tree, root, tokenize("SkillTree"), &|_, _| true))
        else {
            return Vec::new();
        };
        let icon = tokenize("SkillNodeIcon");
        let data = tokenize("Token_ShortcutData");
        (0..16)
            .flat_map(|y| (0..16).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let node = tree.tree_item(skill_tree, x, y)?;
                let icon = tree.child_recursive(node, icon)?;
                match tree.get(icon)?.attribute(data) {
                    Some(AttributeValue::Id(skill)) => Some(*skill),
                    _ => None,
                }
            })
            .collect()
    }

    /// The window pixel at the centre of the skill panel's node for `skill` (the
    /// `SkillNodeIcon` of a `SkillTree` item carrying the skill as its `Token_ShortcutData`).
    pub fn skill_node_center(&self, skill: u32) -> Option<(i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let skill_tree = tree
            .roots()
            .iter()
            .find_map(|&root| find(tree, root, tokenize("SkillTree"), &|_, _| true))?;
        let icon = tokenize("SkillNodeIcon");
        let data = tokenize("Token_ShortcutData");
        let index = (0..16)
            .flat_map(|y| (0..16).map(move |x| (x, y)))
            .find_map(|(x, y)| {
                let node = tree.tree_item(skill_tree, x, y)?;
                let icon = tree.child_recursive(node, icon)?;
                match tree.get(icon)?.attribute(data) {
                    Some(AttributeValue::Id(found)) if *found == skill => Some(icon),
                    _ => None,
                }
            })?;
        Some(self.window_center(tree, index))
    }

    /// The window pixel at the centre of the `ItemIcon` showing `item` inside the shown element
    /// called `within` (a pack page, a paper doll slot; empty: anywhere).
    pub fn item_icon_center(&self, within: &str, item: u64) -> Option<(i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let keep = |tree: &UiTree, index: usize| {
            matches!(
                tree.get(index).and_then(|e| e.attribute(tokenize("Token_ShortcutData"))),
                Some(AttributeValue::InstanceId(data)) if *data == item
            )
        };
        let icon = tokenize("ItemIcon");
        let index = tree.roots().iter().find_map(|&root| {
            if within.is_empty() {
                return find(tree, root, icon, &keep);
            }
            let outer = find(tree, root, tokenize(within), &|_, _| true)?;
            find(tree, outer, icon, &keep)
        })?;
        Some(self.window_center(tree, index))
    }

    /// The window pixel at the centre of slot `index` of the `ContainerListBox` inside the shown
    /// element called `within` (`PackMainContainer`, `ExternalContainer`).
    pub fn container_slot_center(&self, within: &str, index: usize) -> Option<(i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let list = tree.roots().iter().find_map(|&root| {
            let outer = find(tree, root, tokenize(within), &|_, _| true)?;
            find(tree, outer, tokenize("ContainerListBox"), &|_, _| true)
        })?;
        let slot = *tree.list_items(list).get(index)?;
        Some(self.window_center(tree, slot))
    }

    /// The window pixel at the centre of the shortcut bar's slot `index`.
    pub fn shortcut_slot_center(&self, index: usize) -> Option<(i32, i32)> {
        let (tree, _) = self.hud.hud.as_ref()?;
        let (_, list) = self.hud.panels.shortcuts.as_ref()?.elements();
        let slot = *tree.list_items(list?).get(index)?;
        Some(self.window_center(tree, slot))
    }

    /// Whether a dragged copy follows the pointer (`UIElementManager::m_dragElement`).
    pub fn is_dragging(&self) -> bool {
        self.hud
            .hud
            .as_ref()
            .is_some_and(|(tree, _)| tree.dragged().is_some())
    }
}
