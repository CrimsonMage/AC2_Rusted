//! Read-only views of the HUD's panels: what the tests and the frame's queries read.

use ac2_ui::{ToolbarChild, UiTree};

use super::HudView;

impl HudView {
    /// The skills panel's panel and selected skill.
    pub(crate) fn skills_current(&self) -> Option<(u32, u32)> {
        self.skills.as_ref().map(ac2_ui::SkillTreeUi::current)
    }

    /// The text of the first element with id `name` (`IUIManager` lookup).
    pub(crate) fn element_text(&self, name: &str) -> Option<String> {
        let (tree, _) = self.hud.as_ref()?;
        let id = ac2_ui::tokenize(name);
        let element = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id))?;
        tree.get(element)?.text()
    }

    /// The net indicator's tooltip and state (`gmNetStatusUI`'s field), once a heartbeat has
    /// set them.
    pub(crate) fn net_indicator(&self) -> Option<(String, u32)> {
        let (tree, _) = self.hud.as_ref()?;
        let net = self.net.as_ref()?;
        let field = net.field()?;
        Some((tree.tooltip_text(field)?, net.state(tree)?))
    }

    /// The radar's `CoordinatesText` (the HUD has another element of that name).
    pub(crate) fn radar_coordinates(&self) -> Option<String> {
        let (tree, _) = self.hud.as_ref()?;
        let field = self.radar.as_ref()?.field()?;
        let text = tree.child_recursive(field, ac2_ui::tokenize("CoordinatesText"))?;
        tree.get(text)?.text()
    }

    /// The backpack's first page: each item and its slot's tooltip.
    pub(crate) fn backpack_items(&self) -> Vec<(u64, String)> {
        let (Some((tree, _)), Some(pack)) = (&self.hud, &self.backpack) else {
            return Vec::new();
        };
        pack.packs.first().map_or(Vec::new(), |page| {
            page.items
                .iter()
                .map(|d| (d.item, tree.tooltip_text(d.icon).unwrap_or_default()))
                .collect()
        })
    }

    /// The external container panel's container and items, while it shows one.
    pub(crate) fn external_container_items(&self) -> Option<(u64, Vec<u64>)> {
        let display = self.external_container.as_ref()?.display()?;
        Some((
            display.container,
            display.items.iter().map(|d| d.item).collect(),
        ))
    }

    pub(crate) fn allegiance_texts(&self) -> Vec<String> {
        let (Some((tree, _)), Some(page)) = (&self.hud, &self.allegiance) else {
            return Vec::new();
        };
        fn walk(tree: &UiTree, index: usize, out: &mut Vec<String>) {
            if let Some(text) = tree.get(index).and_then(|e| e.text()) {
                if !text.is_empty() {
                    out.push(text);
                }
            }
            for &child in tree.children(index) {
                walk(tree, child, out);
            }
        }
        let mut out = Vec::new();
        for row in page.rows(tree) {
            walk(tree, row, &mut out);
        }
        out
    }

    pub(crate) fn fellowship_rows(&self) -> Vec<(u64, String)> {
        let (Some((tree, _)), Some(panel)) = (&self.hud, &self.fellowship) else {
            return Vec::new();
        };
        let name = ac2_ui::tokenize("FellowTemplateNameText");
        panel
            .rows(tree)
            .into_iter()
            .map(|(id, row)| {
                let text = tree
                    .child_recursive(row, name)
                    .and_then(|t| tree.get(t))
                    .and_then(|e| e.text());
                (id, text.unwrap_or_default())
            })
            .collect()
    }

    pub(crate) fn examination_texts(&self) -> Vec<String> {
        let (Some((tree, _)), Some(panel)) = (&self.hud, &self.examination) else {
            return Vec::new();
        };
        fn walk(tree: &UiTree, index: usize, out: &mut Vec<String>) {
            if let Some(text) = tree.get(index).and_then(|e| e.text()) {
                if !text.is_empty() {
                    out.push(text);
                }
            }
            for &child in tree.children(index) {
                walk(tree, child, out);
            }
        }
        let mut out = Vec::new();
        if let Some(list) = panel.list() {
            for &row in tree.list_items(list) {
                walk(tree, row, &mut out);
            }
        }
        out
    }

    /// The gameplay HUD's element tree, once it is built.
    pub(crate) fn tree(&self) -> Option<&ac2_ui::UiTree> {
        self.hud.as_ref().map(|(tree, _)| tree)
    }

    /// The shortcut bar's slots.
    pub(crate) fn shortcut_slots(&self) -> usize {
        let (Some((tree, _)), Some(bar)) = (&self.hud, &self.shortcuts) else {
            return 0;
        };
        bar.elements()
            .1
            .map_or(0, |list| tree.list_items(list).len())
    }

    /// The first chat window's log text.
    pub(crate) fn chat_log(&self) -> Option<String> {
        let (tree, _) = self.hud.as_ref()?;
        let log = self.chat.as_ref()?.logs().first().copied().flatten()?;
        tree.get(log)?.text()
    }

    /// Whether the HUD is shown (`gmGamePlayUI::m_shown`).
    pub(crate) fn gameplay_shown(&self) -> bool {
        self.gameplay_shown
    }

    /// Whether a gameplay child with a key is shown (the toolbar's bar, the status bar, the radar).
    pub(crate) fn child_visible(&self, child: ac2_ui::GameplayChild) -> bool {
        self.child_shown(child)
    }

    /// Whether a toolbar panel is shown.
    pub(crate) fn panel_shown(&self, child: ToolbarChild) -> bool {
        match (&self.hud, &self.toolbar) {
            (Some((tree, _)), Some(toolbar)) => toolbar.shown(tree, child),
            _ => false,
        }
    }

    /// Whether an editable text element is active (the input manager's text mode).
    pub(crate) fn text_mode(&self) -> bool {
        self.hud.as_ref().is_some_and(|(tree, _)| tree.text_mode())
    }

    /// Whether a read-only text holds the copy focus (typed characters reach it, not the game).
    pub(crate) fn text_copy_focus(&self) -> bool {
        self.hud
            .as_ref()
            .is_some_and(|(tree, _)| tree.text_copy_focus())
    }

    /// The objects with a name plate.
    pub(crate) fn named(&self) -> Vec<u64> {
        self.names.as_ref().map_or(Vec::new(), |n| n.named())
    }

    /// The objects with a floating damage text in the air.
    pub(crate) fn damage_objects(&self) -> Vec<u64> {
        self.damage.as_ref().map_or(Vec::new(), |d| d.objects())
    }

    /// The container `gmExternalContainerUI` waits to be placed over.
    pub(crate) fn external_container_position(&self) -> Option<u64> {
        self.external_container.as_ref()?.wants_position()
    }

    /// The pointer as the UI last saw it, in window pixels.
    pub(crate) fn pointer(&self) -> (i32, i32) {
        let (x, y) = self.hud.as_ref().map_or((0, 0), |(tree, _)| tree.pointer());
        let (x, y) = self.scale.to_window(f64::from(x), f64::from(y));
        (x.round() as i32, y.round() as i32)
    }

    pub(crate) fn shown_layers(&self) -> usize {
        self.overlay.shown_layers()
    }

    pub(crate) fn layers(&self) -> usize {
        self.overlay.layers()
    }

    /// The paper doll's viewport while it shows.
    pub(crate) fn doll_viewport(&self) -> Option<ac2_ui::SceneViewport> {
        let (tree, _) = self.hud.as_ref()?;
        let root = self.paper_doll.as_ref()?.viewport()?;
        ac2_ui::scene_viewports(tree)
            .into_iter()
            .find(|v| v.root == root)
    }

    /// Where the paper doll waits for a pick, if it does.
    pub(crate) fn doll_wants_pick(&self) -> Option<(i32, i32)> {
        self.paper_doll.as_ref()?.wants_pick()
    }
}
