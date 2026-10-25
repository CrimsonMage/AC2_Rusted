//! `gmExternalContainerUI`: the panel over a container in the world (`ExternalContainerField` of
//! the gameplay layout), its slots a [`ContainerDisplay`] in `ExternalContainer`. It listens to
//! every object (`Init`, `RecvNotice_CreateObject`): a container opening shows the panel over
//! the object and lists the container (`RecvNotice_ContainerStateUpdate`); any container closing,
//! or the shown one going (`RecvNotice_BeingDeleted`), empties and hides it. `Close` asks the
//! container shut (`gmCWInterface::UpdateOpen(player, 0)`) and hides the panel; `Loot All` and
//! `Transmute All` ask for the whole container.
//!
//! `PositionWindow` centres the panel on the object's box centre projected to the window, or on
//! the display when that is behind the camera, kept on the display. The projection needs the
//! camera, so the client answers [`ExternalContainerUi::wants_position`] the same frame
//! ([`ExternalContainerUi::position_window_clip`]).

use std::sync::Arc;

use crate::element::UiTree;
use crate::item_display::{ContainerDisplay, InventorySource, ItemRequest};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// What the panel asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalContainerRequest {
    /// A slot's request (press, drop, split).
    Item(ItemRequest),
    /// `gmCWInterface::UpdateOpen(player, 0)`.
    Close(u64),
    /// `gmCWInterface::LootAllFromContainer`.
    LootAll(u64),
    /// `gmCWInterface::TransmuteAllFromContainer`.
    TransmuteAll(u64),
}

/// `gmExternalContainerUI`'s state.
#[derive(Debug, Clone)]
pub struct ExternalContainerUi {
    /// The toolbar layout, the slots' (`ContainerDisplay::Init`'s `0x41000003`).
    slots: Arc<LayoutDesc>,
    /// `_externalContainerField`, `_container`, `_label`.
    field: Option<usize>,
    container: Option<usize>,
    label: Option<usize>,
    /// `_externalContainer`, while a container is listed.
    display: Option<ContainerDisplay>,
    /// The object `PositionWindow` waits to be placed over.
    position_for: Option<u64>,
}

impl ExternalContainerUi {
    /// `gmExternalContainerUI::gmExternalContainerUI`: the hidden field of the gameplay layout.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>, slots: &Arc<LayoutDesc>) -> Self {
        let field = tree.create_root(Arc::clone(gameplay), id("ExternalContainerField"));
        let container = field.and_then(|f| tree.child_recursive(f, id("ExternalContainer")));
        let label = field.and_then(|f| tree.child_recursive(f, id("ExternalContainerLabel")));
        if let Some(field) = field {
            tree.set_visible(field, false);
        }
        Self {
            slots: Arc::clone(slots),
            field,
            container,
            label,
            display: None,
            position_for: None,
        }
    }

    /// The panel's root.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// The listed container's display, while the panel shows one.
    pub fn display(&self) -> Option<&ContainerDisplay> {
        self.display.as_ref()
    }

    pub fn display_mut(&mut self) -> Option<&mut ContainerDisplay> {
        self.display.as_mut()
    }

    /// `IsShown`.
    fn shown(&self, tree: &UiTree) -> bool {
        self.field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.visible)
    }

    fn show(&self, tree: &mut UiTree, shown: bool) {
        if let Some(field) = self.field {
            tree.set_visible(field, shown);
        }
    }

    /// `ContainerDisplay::Cleanup` and `Show(0)`: the stack dialog context to close, if a drop
    /// was waiting on it.
    fn close_panel(&mut self, tree: &mut UiTree) -> Vec<u32> {
        let contexts = self.display.take().map(|mut d| d.cleanup(tree));
        self.position_for = None;
        self.show(tree, false);
        contexts.unwrap_or_default()
    }

    /// `gmExternalContainerUI::RecvNotice_ContainerStateUpdate`: an opening shows the panel and
    /// lists the container (`ContainerDisplay::Init`, which cleans up first); a closing empties
    /// and hides it. Returns a stack dialog context to close.
    pub fn container_state_update(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        container: u64,
        open: bool,
    ) -> Vec<u32> {
        if !open {
            return self.close_panel(tree);
        }
        self.show(tree, true);
        self.position_for = Some(container);
        let contexts = self
            .display
            .take()
            .map(|mut d| d.cleanup(tree))
            .unwrap_or_default();
        if let (Some(element), Some(_)) = (self.container, self.label) {
            self.display = Some(ContainerDisplay::init(
                tree,
                &self.slots,
                s,
                container,
                element,
                0,
                0,
            ));
        }
        contexts
    }

    /// `gmExternalContainerUI::RecvNotice_BeingDeleted`: the shown container going hides the
    /// panel. Returns a stack dialog context to close.
    pub fn being_deleted(&mut self, tree: &mut UiTree, object: u64) -> Vec<u32> {
        let listed = self.display.as_ref().map(|d| d.container);
        if self.shown(tree) && listed == Some(object) {
            return self.close_panel(tree);
        }
        Vec::new()
    }

    /// The object the panel waits to be placed over (`PositionWindow` after an opening).
    pub fn wants_position(&self) -> Option<u64> {
        self.position_for
    }

    /// `gmExternalContainerUI::PositionWindow`: centred on `at` (the object's box centre in
    /// window pixels, `SurfaceWindow::ClipToWindow`), or on the display when it is behind the
    /// camera (`None`); kept on the display. The half sizes are stored as floats and each corner
    /// truncated (`_ftol2`), admin 0x4206a0.
    pub fn position_window(&mut self, tree: &mut UiTree, at: Option<(i32, i32)>) {
        self.position_for = None;
        let Some(field) = self.field else {
            return;
        };
        let rect = tree.get(field).map(|e| e.rect).unwrap_or_default();
        let hw = f64::from((f64::from(rect.x1 - rect.x0 + 1) * 0.5) as f32);
        let hh = f64::from((f64::from(rect.y1 - rect.y0 + 1) * 0.5) as f32);
        let (dw, dh) = tree.display;
        let (cx, cy) = match at {
            Some((x, y)) => (f64::from(x), f64::from(y)),
            None => (f64::from(dw) * 0.5, f64::from(dh) * 0.5),
        };
        let clamp = |corner: f64, half: f64, display: i32| {
            let corner = corner as i32;
            if corner < 0 {
                0
            } else if display < corner - (half * -2.0) as i32 {
                (f64::from(display) - (half + half)) as i32
            } else {
                corner
            }
        };
        let x = clamp(cx - hw, hw, dw);
        let y = clamp(cy - hh, hh, dh);
        tree.set_object_position(field, x, y);
    }

    /// [`Self::position_window`] from the object's box centre in clip space (`None` behind
    /// the camera): `SurfaceWindow::ClipToWindow` on the full-screen window.
    pub fn position_window_clip(&mut self, tree: &mut UiTree, clip: Option<[f32; 2]>) {
        let (w, h) = tree.display;
        let at = clip.map(|c| crate::reticle::clip_to_pixel(c, w, h));
        self.position_window(tree, at);
    }

    /// `gmExternalContainerUI::ListenToMessage` (its buttons) and the display's slots and drops.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> (Option<ExternalContainerRequest>, Vec<u32>) {
        let listed = self.display.as_ref().map(|d| d.container);
        if message.message == MSG_BUTTON_RELEASE_ON {
            let button = message.element_id;
            if button == id("ExternalContainerCloseButton") {
                if let (true, Some(container)) = (self.shown(tree), listed) {
                    let contexts = self.close_panel(tree);
                    return (Some(ExternalContainerRequest::Close(container)), contexts);
                }
                return (None, Vec::new());
            }
            if button == id("LootAllButton") {
                return (listed.map(ExternalContainerRequest::LootAll), Vec::new());
            }
            if button == id("TransmuteAllButton") {
                return (
                    listed.map(ExternalContainerRequest::TransmuteAll),
                    Vec::new(),
                );
            }
        }
        let request = self
            .display
            .as_mut()
            .and_then(|d| d.message(tree, s, message))
            .map(ExternalContainerRequest::Item);
        (request, Vec::new())
    }

    /// `ContainerDisplay::RecvNotice_StackUIDone` for the listed container.
    pub fn stack_ui_done(
        &mut self,
        s: &dyn InventorySource,
        context: u32,
        amount: u32,
    ) -> Option<ItemRequest> {
        self.display
            .as_mut()
            .and_then(|d| d.stack_ui_done(s, context, amount))
    }
}
