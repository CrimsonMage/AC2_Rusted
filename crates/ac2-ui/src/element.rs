//! UI elements: the live tree `UIElementManager` builds from layouts.
//!
//! `UIElementManager::CreateElement` makes an element for an `ElementDesc` (a reference element
//! stands for the top-level element its `referent` attribute names,
//! `LayoutDesc::DereferenceElementDesc`), sets it to the description's default state
//! (`UIElement::SetState`: the reference's then the description's attributes for the state and
//! the stateless state, first one set winning; `ResetMedia`; `Update_Internal`), updates it and
//! shows it. `Update_Internal` makes or refreshes a child for every element the current and the
//! stateless state list (`UpdateChild`), drops children no state lists any more, and places the
//! element (`UpdateSizeAndPosition`, in `place.rs`). Siblings are kept in ascending z
//! (`UIElement::AddChild`; a z of `0xFFFF` always goes last).
//!
//! Not modelled yet: transitions (`StartTransition`), timed media (an animation shows its first
//! frame, a pause stops the run, a jump never fires), mouse handling and the typed attribute side
//! effects other than `hide`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, StateDesc};
use crate::mouse::UiMessage;
use crate::text::{TextState, UiTextData};
use crate::token::{HIDE, MOUSE_INVISIBLE, REFERENCE, REFERENT, ROOT_MOVER, STATELESS, TEXT};

/// A `Box2D`: inclusive pixel bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiBox {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl UiBox {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub fn width(&self) -> i32 {
        self.x1 - self.x0 + 1
    }

    pub fn height(&self) -> i32 {
        self.y1 - self.y0 + 1
    }

    pub fn offset(&self, x: i32, y: i32) -> Self {
        Self::new(self.x0 + x, self.y0 + y, self.x1 + x, self.y1 + y)
    }

    /// `Box2D::Intersection`: `None` when the overlap is empty.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x0 = self.x0.max(other.x0);
        let y0 = self.y0.max(other.y0);
        let x1 = self.x1.min(other.x1);
        let y1 = self.y1.min(other.y1);
        (x1 >= x0 && y1 >= y0).then_some(Self::new(x0, y0, x1, y1))
    }
}

/// `UIElement::SetImage`'s image and draw mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiImage {
    pub file: u32,
    pub draw_mode: u32,
}

/// A root's surface object on the screen: its position, rotation about its centre in degrees,
/// whether it is shown, and its material opacity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiPlacement {
    pub root: usize,
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub shown: bool,
    pub opacity: f32,
}

/// One element.
#[derive(Debug, Clone)]
pub struct UiElement {
    pub(crate) layout: Arc<LayoutDesc>,
    /// `referenceDesc`: the description the element was made from.
    pub(crate) reference: Arc<ElementDesc>,
    /// `desc`: `reference` dereferenced.
    pub(crate) desc: Arc<ElementDesc>,
    pub(crate) state: u32,
    pub(crate) parent: Option<usize>,
    /// First to last, ascending z.
    pub(crate) children: Vec<usize>,
    /// `m_box`, relative to the parent (a root's is relative to its surface object).
    pub(crate) rect: UiBox,
    /// A root's surface object position (`m_ownsObject`).
    pub(crate) object: Option<(i32, i32)>,
    pub(crate) z: i32,
    pub(crate) visible: bool,
    pub(crate) image: Option<UiImage>,
    pub(crate) alpha_image: Option<u32>,
    /// `m_alphaValue` (-1 until `SetImage` sets it): the constant alpha of value-alpha images.
    pub(crate) alpha_value: f32,
    pub(crate) attributes: HashMap<u32, AttributeValue>,
    /// `UIElement_Text`'s state, for text elements.
    pub(crate) text: Option<Box<TextState>>,
    /// `UIElement_ListBox`'s state, once items are added.
    pub(crate) list: Option<Box<crate::list_box::ListBoxState>>,
    /// `UIElement_Scrollbar`'s drag (`scrollbar.rs`).
    pub(crate) scrollbar: Option<Box<crate::scrollbar::ScrollbarState>>,
    /// `UIElement_PanelPage::page`: the tab's content.
    pub(crate) page: Option<usize>,
    /// `UIElement_Tree`'s state, once items are added.
    pub(crate) tree: Option<Box<crate::tree_view::TreeState>>,
    /// `UIElement_Meter`'s state, for meters.
    pub(crate) meter: Option<Box<crate::meter::MeterState>>,
    /// `UIElement_Menu`'s state, for menus.
    pub(crate) menu: Option<Box<crate::menu::MenuState>>,
    /// `m_mouseOver`, `m_mouseOverTop`, `m_mouseVisible` and the buttons pressed on the element
    /// (`m_mouseDownTable`).
    pub(crate) mouse_over: bool,
    pub(crate) mouse_over_top: bool,
    pub(crate) mouse_visible: bool,
    pub(crate) mouse_down: Vec<u32>,
    /// `m_dragOutCallback` and `m_dragDropCallback`.
    pub(crate) drag_out: Option<crate::drag::DragCheck>,
    pub(crate) drag_drop: Option<crate::drag::DragCheck>,
    /// `UIElement_Button::mousePressedOnButton`.
    pub(crate) button_pressed: bool,
    /// A root's surface object: shown (`UIObject` show, apart from the element's own
    /// visibility) and rotated about its centre in degrees (`UISurfaceObject::SetRotateVals`).
    pub(crate) shown: bool,
    pub(crate) rotation: i32,
    /// `mediaState` (the description and state whose media run) and `m_curMediaIndex`.
    pub(crate) media_state: Option<(Arc<ElementDesc>, usize)>,
    pub(crate) media_index: i32,
    /// `m_transitionArray`, `m_transition_state`, `m_transition_close`.
    pub(crate) transitions: Vec<crate::transition::ActiveTransition>,
    pub(crate) transition_state: u32,
    pub(crate) transition_close: bool,
    /// A root's surface object's material opacity (`UISurfaceObject::SetMaterialOpacity`).
    pub(crate) opacity: f32,
    /// `m_cursorDID`, `m_cursorHotX`, `m_cursorHotY`.
    pub(crate) cursor: (u32, i32, i32),
    /// `m_savedCursorDID`, `m_savedCursorHotX`, `m_savedCursorHotY` (`UIElement::SaveCursor`).
    pub(crate) saved_cursor: (u32, i32, i32),
    /// `m_TTText`, rendered (`SetTooltip`).
    pub(crate) tooltip: Option<String>,
    latest_update: u32,
    created_by_layout: bool,
    /// A root that is not a window the player moves: a world overlay (the targeting reticle, name
    /// plates, damage text), a full-display backdrop, or a part that moves with another window
    /// (the radar's heading ring, the doll's 3D viewport). It does not move by Alt + drag and is not
    /// saved as a window position ([`UiTree::set_overlay_root`]).
    pub(crate) overlay: bool,
}

impl UiElement {
    /// The element id (`referenceDesc->elementID`).
    pub fn id(&self) -> u32 {
        self.reference.id
    }

    /// The element class (`desc->type`).
    pub fn kind(&self) -> u32 {
        self.desc.kind
    }

    pub fn state(&self) -> u32 {
        self.state
    }

    pub fn image(&self) -> Option<UiImage> {
        self.image
    }

    pub fn attribute(&self, id: u32) -> Option<&AttributeValue> {
        self.attributes.get(&id)
    }

    /// `UIRegion::IsVisible` (the element's own flag).
    /// The element's rectangle on the screen (`UIRegion`'s bounds).
    pub fn rect(&self) -> UiBox {
        self.rect
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// `m_mouseOver`.
    pub fn mouse_over(&self) -> bool {
        self.mouse_over
    }

    /// The parent element.
    pub fn parent_index(&self) -> Option<usize> {
        self.parent
    }

    /// A text element's text (`UIElement_Text::GetText`).
    pub fn text(&self) -> Option<String> {
        self.text
            .as_ref()
            .map(|text| String::from_utf16_lossy(&text.text()))
    }
}

/// The element tree (`UIElementManager`'s elements and root list).
#[derive(Debug, Clone)]
pub struct UiTree {
    pub(crate) slots: Vec<Option<UiElement>>,
    free: Vec<usize>,
    /// Ascending z, like siblings.
    pub(crate) roots: Vec<usize>,
    update_count: u32,
    /// The render device's display size.
    pub(crate) display: (i32, i32),
    /// `gmStackUI::m_globalContext`.
    pub(crate) stack_context: u32,
    /// Fonts and strings for text elements.
    pub(crate) text_data: Option<Arc<dyn UiTextData>>,
    /// Messages broadcast and not yet taken.
    pub(crate) messages: Vec<UiMessage>,
    pub(crate) pointer: (i32, i32),
    pub(crate) mouse_top: Option<usize>,
    /// Something drawn changed since the last [`UiTree::take_changed`].
    pub(crate) changed: bool,
    /// A root's surface object moved, turned or showed since the last [`UiTree::take_moved`].
    pub(crate) moved: bool,
    /// `m_activeElement`.
    pub(crate) active: Option<usize>,
    /// `Timer::cur_time` of the last global loop, in seconds.
    pub(crate) time: f64,
    /// `Random::rand`'s state for the media's rolls.
    pub(crate) random: ac2_core::Random,
    /// The media items' timers, shared by every element that runs the same description.
    pub(crate) media_clocks: std::collections::HashMap<usize, crate::media::MediaClock>,
    /// Sounds the media played since the last [`UiTree::take_sounds`].
    pub(crate) sounds: Vec<u32>,
    /// `UIElementManager`'s cursor: the last set and the default.
    pub(crate) cursor: Option<(u32, i32, i32)>,
    pub(crate) default_cursor: Option<(u32, i32, i32)>,
    /// `UIElementManager`'s tooltip, and the owner the current mouse pass found.
    pub(crate) tooltip: crate::tooltip::TooltipState,
    pub(crate) new_tooltip_owner: Option<usize>,
    /// `UIElementManager`'s drag and drop.
    pub(crate) drag: crate::drag::DragState,
    /// The element being moved by its drag bar (`UIElement::m_moving`).
    pub(crate) movement: Option<crate::window_locations::Movement>,
    /// A window that saves its location moved (`location_changed`), until taken.
    pub(crate) locations_changed: bool,
    /// The clipboard the text elements copy, cut and paste through (`text_clip.rs`).
    pub(crate) text_clipboard: std::sync::Arc<dyn crate::text_clipboard::TextClipboard>,
    /// The editable or selectable text element a left press started a selection in
    /// (`UIElement_Text::m_mouseSelecting`); the drag moves its moving end until the release.
    pub(crate) text_drag: Option<usize>,
    /// The Alt key is held (the window's modifier state, [`UiTree::set_alt`]).
    pub(crate) alt: bool,
    /// A window moved and the drag ended since the last [`UiTree::take_positions_changed`].
    pub(crate) positions_changed: bool,
    /// The player's saved positions of the windows (`window_positions`): applied to a window when
    /// it is made and to the windows already up.
    pub(crate) saved_positions: Vec<crate::window_positions::WindowPosition>,
}

/// The index of `desc`'s state `id`.
pub(crate) fn state_of(desc: &ElementDesc, id: u32) -> Option<&StateDesc> {
    desc.states.iter().find(|state| state.id == id)
}

/// `LayoutDesc::DereferenceElementDesc`: follows `referent` attributes to a non-reference
/// top-level element of `layout`.
pub(crate) fn dereference(
    layout: &LayoutDesc,
    desc: &Arc<ElementDesc>,
) -> Option<Arc<ElementDesc>> {
    let mut desc = Arc::clone(desc);
    for _ in 0..64 {
        if desc.kind != REFERENCE {
            return Some(desc);
        }
        let stateless = state_of(&desc, STATELESS)?;
        let referent = stateless.attributes.iter().find_map(|attribute| {
            match (attribute.id, &attribute.value) {
                (REFERENT, AttributeValue::Id(id)) => Some(*id),
                _ => None,
            }
        })?;
        desc = Arc::clone(layout.element(referent)?);
    }
    None
}

impl UiTree {
    /// An empty tree on a `width` x `height` display.
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            roots: Vec::new(),
            update_count: 0,
            display: (width, height),
            stack_context: 0,
            text_data: None,
            messages: Vec::new(),
            pointer: (0, 0),
            mouse_top: None,
            changed: true,
            moved: true,
            active: None,
            time: 0.0,
            random: ac2_core::Random::default(),
            media_clocks: std::collections::HashMap::new(),
            sounds: Vec::new(),
            cursor: None,
            default_cursor: None,
            tooltip: crate::tooltip::TooltipState::default(),
            new_tooltip_owner: None,
            drag: crate::drag::DragState::default(),
            movement: None,
            locations_changed: false,
            text_clipboard: std::sync::Arc::new(crate::text_clipboard::MemoryClipboard::default()),
            text_drag: None,
            alt: false,
            positions_changed: false,
            saved_positions: Vec::new(),
        }
    }

    /// The fonts and strings text elements made from now on draw from.
    /// The text data the tree reads its strings and fonts through (`UIElement::GetTextData`).
    pub fn text_data(&self) -> Option<Arc<dyn UiTextData>> {
        self.text_data.clone()
    }

    pub fn set_text_data(&mut self, data: Arc<dyn UiTextData>) {
        self.text_data = Some(data);
    }

    /// The clipboard text elements copy, cut and paste through (the client's OS clipboard). A
    /// tree starts with its own in-memory one.
    pub fn set_text_clipboard(&mut self, clipboard: Arc<dyn crate::text_clipboard::TextClipboard>) {
        self.text_clipboard = clipboard;
    }

    /// One past the highest element index.
    pub fn slots_len(&self) -> usize {
        self.slots.len()
    }

    pub fn get(&self, index: usize) -> Option<&UiElement> {
        self.slots.get(index)?.as_ref()
    }

    pub(crate) fn el(&self, index: usize) -> &UiElement {
        self.slots[index].as_ref().expect("live element")
    }

    pub(crate) fn el_mut(&mut self, index: usize) -> &mut UiElement {
        self.slots[index].as_mut().expect("live element")
    }

    /// The root elements, ascending z.
    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    pub fn children(&self, index: usize) -> &[usize] {
        &self.el(index).children
    }

    /// `UIElement::GetChildRecursive`: the first descendant with element id `id`.
    pub fn child_recursive(&self, index: usize, id: u32) -> Option<usize> {
        for &child in &self.el(index).children {
            if self.el(child).id() == id {
                return Some(child);
            }
            if let Some(found) = self.child_recursive(child, id) {
                return Some(found);
            }
        }
        None
    }

    /// Every image file the live elements show.
    pub fn image_files(&self) -> BTreeSet<u32> {
        self.slots
            .iter()
            .flatten()
            .flat_map(|el| {
                el.image
                    .map(|image| image.file)
                    .into_iter()
                    .chain(el.alpha_image)
            })
            .collect()
    }

    /// `UIMainFramework::CreateAndAddRootElementByDataID` with the layout loaded: the layout's
    /// top-level element `element_id` made with its own surface object, marked `root_mover` and
    /// placed.
    pub fn create_root(&mut self, layout: Arc<LayoutDesc>, element_id: u32) -> Option<usize> {
        let reference = Arc::clone(layout.element(element_id)?);
        let index = self.create_element(layout, reference, true)?;
        self.initialize_panel_pages(index);
        self.set_bool(index, ROOT_MOVER, true);
        self.update_size_and_position(index);
        self.add_root(index);
        // A window the player left somewhere else opens there (`window_positions`).
        self.apply_saved_position(index);
        Some(index)
    }

    fn alloc(&mut self, element: UiElement) -> usize {
        match self.free.pop() {
            Some(index) => {
                self.slots[index] = Some(element);
                index
            }
            None => {
                self.slots.push(Some(element));
                self.slots.len() - 1
            }
        }
    }

    /// Frees an element and its descendants (`AddElementToDeleteQueue`). A menu's popup goes with
    /// it (`UIElement_Menu::~UIElement_Menu`).
    pub(crate) fn delete(&mut self, index: usize) {
        self.menu_delete(index);
        let children = std::mem::take(&mut self.el_mut(index).children);
        for child in children {
            self.delete(child);
        }
        if self.active == Some(index) {
            self.active = None;
        }
        self.slots[index] = None;
        self.free.push(index);
    }

    /// `UIElementManager::CreateElement`.
    pub(crate) fn create_element(
        &mut self,
        layout: Arc<LayoutDesc>,
        reference: Arc<ElementDesc>,
        owns_object: bool,
    ) -> Option<usize> {
        let desc = dereference(&layout, &reference)?;
        let default_state = reference.default_state;
        let is_meter = desc.kind == crate::meter::METER;
        // `UIElement_Text::UIElement_Text`: the default font, no text.
        let text = (desc.kind == TEXT).then(|| {
            let font = self.text_data.as_ref().and_then(|data| data.default_font());
            Box::new(TextState::new(font))
        });
        let index = self.alloc(UiElement {
            layout,
            reference,
            desc,
            state: 0,
            parent: None,
            children: Vec::new(),
            // `UIRegion::UIRegion(0, 0, 0, 0, 0)`: an empty box.
            rect: UiBox::new(0, 0, -1, -1),
            object: owns_object.then_some((0, 0)),
            z: 0,
            visible: false,
            image: None,
            alpha_image: None,
            alpha_value: -1.0,
            attributes: HashMap::new(),
            text,
            tree: None,
            meter: None,
            menu: None,
            page: None,
            list: None,
            scrollbar: None,
            mouse_over: false,
            mouse_over_top: false,
            mouse_visible: true,
            mouse_down: Vec::new(),
            drag_out: None,
            drag_drop: None,
            button_pressed: false,
            shown: true,
            rotation: 0,
            media_state: None,
            media_index: 0,
            transitions: Vec::new(),
            transition_state: 0,
            transition_close: false,
            opacity: 1.0,
            cursor: (0, 0, 0),
            saved_cursor: (0, 0, 0),
            tooltip: None,
            latest_update: self.update_count,
            created_by_layout: false,
            overlay: false,
        });
        if is_meter {
            self.meter_create(index);
        }
        self.set_state(index, default_state);
        self.update_count += 1;
        self.update_internal(index);
        self.set_visible(index, true);
        // `UIElement_Menu::Initialized`, once the menu's children exist.
        if self.is_menu(index) {
            self.menu_initialize(index);
        }
        Some(index)
    }

    /// `UIElement::SetVisible` (`UIRegion::SetVisible`).
    pub fn set_visible(&mut self, index: usize, visible: bool) {
        self.el_mut(index).visible = visible;
        self.changed = true;
        self.menu_visibility(index);
    }

    /// Whether anything drawn changed since the last call.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// Whether a root's surface object moved, turned or showed since the last call.
    pub fn take_moved(&mut self) -> bool {
        std::mem::take(&mut self.moved)
    }

    /// `ChatWindow::SetOpacity`'s walk: `m_alphaValue` of `index` and every child, recursively.
    pub(crate) fn set_alpha_recursive(&mut self, index: usize, alpha: f32) {
        self.el_mut(index).alpha_value = alpha;
        self.changed = true;
        for child in self.el(index).children.clone() {
            self.set_alpha_recursive(child, alpha);
        }
    }

    /// `UISurfaceObject::SetScreenPosition` of a root's object (no clamping: `UIElement::MoveTo`
    /// keeps roots on the display, this does not).
    pub fn set_object_position(&mut self, root: usize, x: i32, y: i32) {
        let el = self.el_mut(root);
        if el.object != Some((x, y)) {
            el.object = Some((x, y));
            self.moved = true;
        }
    }

    /// `UISurfaceObject::SetRotateVals(0, 0, degrees)`.
    pub fn set_object_rotation(&mut self, root: usize, degrees: i32) {
        let el = self.el_mut(root);
        if el.rotation != degrees {
            el.rotation = degrees;
            self.moved = true;
        }
    }

    /// `UIObject` show of a root's surface object.
    pub fn show_object(&mut self, root: usize, shown: bool) {
        let el = self.el_mut(root);
        if el.shown != shown {
            el.shown = shown;
            self.moved = true;
        }
    }

    /// Every root's surface object placement, in the order [`UiTree::compose`] returns them:
    /// (root, x, y, rotation, shown).
    pub fn placements(&self) -> Vec<UiPlacement> {
        self.composed_roots()
            .map(|root| {
                let el = self.el(root);
                let (x, y) = el.object.unwrap_or((0, 0));
                UiPlacement {
                    root,
                    x,
                    y,
                    rotation: el.rotation,
                    shown: el.shown,
                    opacity: el.opacity,
                }
            })
            .collect()
    }

    /// The roots [`UiTree::compose`] draws, in its order.
    pub(crate) fn composed_roots(&self) -> impl Iterator<Item = usize> + '_ {
        self.roots.iter().rev().copied().filter(|&root| {
            let el = self.el(root);
            el.visible && el.rect.width() > 0 && el.rect.height() > 0
        })
    }

    /// `UIElement::SetAttribute_Bool`: stores the value; a change of `hide` shows or hides.
    pub fn set_bool(&mut self, index: usize, id: u32, value: bool) {
        let previous = self
            .el_mut(index)
            .attributes
            .insert(id, AttributeValue::Bool(value));
        let changed = previous != Some(AttributeValue::Bool(value));
        if changed && id == HIDE {
            self.set_visible(index, !value);
        }
        if changed && id == MOUSE_INVISIBLE {
            self.el_mut(index).mouse_visible = !value;
        }
        if changed {
            self.changed = true;
            self.button_attribute(index, id, value);
            self.menu_attribute(index, id, value);
        }
        self.text_attribute(index, id, &AttributeValue::Bool(value), changed);
    }

    /// `UIElement::SetAttributeFromDesc`.
    fn set_attribute(&mut self, index: usize, attribute: &AttributeDesc) {
        match attribute.value {
            AttributeValue::Bool(value) => self.set_bool(index, attribute.id, value),
            AttributeValue::None(_) => {}
            AttributeValue::Float(value) => {
                let changed = self.set_float(index, attribute.id, value);
                self.text_attribute(index, attribute.id, &attribute.value, changed);
            }
            ref value => {
                let previous = self
                    .el_mut(index)
                    .attributes
                    .insert(attribute.id, value.clone());
                let changed = previous.as_ref() != Some(value);
                self.text_attribute(index, attribute.id, value, changed);
            }
        }
    }

    /// `UIElement::AddAttributesForState`: the state's attributes, then the stateless state's,
    /// each id set once per `SetState` (`addAttributeTable`).
    fn add_attributes_for_state(
        &mut self,
        index: usize,
        desc: &ElementDesc,
        state: u32,
        seen: &mut HashSet<u32>,
    ) {
        let states = (state != STATELESS)
            .then(|| state_of(desc, state))
            .flatten()
            .into_iter()
            .chain(state_of(desc, STATELESS));
        for state in states {
            for attribute in &state.attributes {
                if seen.insert(attribute.id) {
                    self.set_attribute(index, attribute);
                }
            }
        }
    }

    /// `UIElement::SetState`; a meter lays its fills out again (`UIElement_Meter::SetState`).
    pub fn set_state(&mut self, index: usize, state: u32) {
        self.button_set_state(index, state);
        self.transition_set_state(index, state);
        self.meter_update_layout(index);
    }

    /// `UIElement::SetState`'s change once no exit transition holds it: the state's attributes,
    /// its media, its children, a flagged state passed on to the children.
    pub(crate) fn enter_state(&mut self, index: usize, state: u32) {
        let (reference, desc) = {
            let el = self.el(index);
            (Arc::clone(&el.reference), Arc::clone(&el.desc))
        };
        let mut seen = HashSet::new();
        if !Arc::ptr_eq(&reference, &desc) {
            self.add_attributes_for_state(index, &reference, state, &mut seen);
        }
        self.add_attributes_for_state(index, &desc, state, &mut seen);
        self.el_mut(index).state = state;
        self.changed = true;
        self.reset_media(index);
        self.update_count += 1;
        self.update_internal(index);
        // A state flagged 1 passes itself on to the children.
        let target = if state == STATELESS { STATELESS } else { state };
        if state_of(&desc, target).is_some_and(|s| s.flags & 1 != 0) {
            for child in self.el(index).children.clone() {
                if self.get(child).is_some() {
                    self.set_state(child, state);
                }
            }
        }
        if self.is_panel_page(index) {
            self.panel_page_state(index, state);
        }
        // `UIElement_Scrollbar::SetState`.
        self.scrollbar_update_layout(index);
    }

    /// `UIRegion::ClearImage`.
    pub fn clear_image(&mut self, index: usize) {
        let el = self.el_mut(index);
        if el.image.take().is_some() {
            self.changed = true;
        }
    }

    /// `UIElement::SetAttribute_Int`.
    pub fn set_int(&mut self, index: usize, id: u32, value: i32) {
        self.el_mut(index)
            .attributes
            .insert(id, AttributeValue::Int(value));
    }

    /// `UIElement::SetAttribute_ID`.
    pub(crate) fn set_attribute_id(&mut self, index: usize, id: u32, value: u32) {
        self.el_mut(index)
            .attributes
            .insert(id, AttributeValue::Id(value));
    }

    /// `referenceDesc->defaultState`: the state `UIElement::SetState` returns the element to.
    pub(crate) fn default_state_of(&self, index: usize) -> u32 {
        self.el(index).reference.default_state
    }

    /// `UIElement::SetImage`: the image and, while unset, `m_alphaValue` from `alpha_value`
    /// (retail reads the attribute into the draw mode's own slot, so without the attribute the
    /// value becomes the draw mode's bits as a float: no alpha).
    pub(crate) fn set_image(&mut self, index: usize, file: u32, draw_mode: u32) {
        let alpha = match self.el(index).attributes.get(&crate::token::ALPHA_VALUE) {
            Some(AttributeValue::Float(value)) => *value,
            _ => f32::from_bits(draw_mode),
        };
        let el = self.el_mut(index);
        if el.alpha_value == -1.0 && alpha != el.alpha_value {
            el.alpha_value = alpha;
        }
        el.image = Some(UiImage { file, draw_mode });
        self.changed = true;
    }

    /// `UIElement::Update_Internal`.
    pub(crate) fn update_internal(&mut self, index: usize) {
        let min = self.update_count;
        let (reference, desc, state) = {
            let el = self.el_mut(index);
            el.latest_update = min;
            (Arc::clone(&el.reference), Arc::clone(&el.desc), el.state)
        };
        let mut listed = Vec::new();
        let mut list = |desc: &ElementDesc| {
            if state != STATELESS {
                if let Some(s) = state_of(desc, state) {
                    listed.extend(s.children.iter().cloned());
                }
            }
            if let Some(s) = state_of(desc, STATELESS) {
                listed.extend(s.children.iter().cloned());
            }
        };
        list(&desc);
        if !Arc::ptr_eq(&reference, &desc) {
            list(&reference);
        }
        for child in listed {
            self.update_child(index, child, min);
        }
        for child in self.el(index).children.clone() {
            let el = self.el(child);
            if el.latest_update < min && el.created_by_layout {
                self.set_parent(child, None);
                self.delete(child);
            }
        }
        self.update_size_and_position(index);
    }

    /// `UIElement::UpdateChild`.
    fn update_child(&mut self, index: usize, reference: Arc<ElementDesc>, min: u32) {
        let existing = self
            .el(index)
            .children
            .iter()
            .copied()
            .find(|&child| self.el(child).id() == reference.id);
        if let Some(child) = existing {
            if self.el(child).latest_update >= min {
                return;
            }
        }
        let layout = Arc::clone(&self.el(index).layout);
        let Some(desc) = dereference(&layout, &reference) else {
            return;
        };
        if let Some(child) = existing {
            let el = self.el(child);
            if el.desc.kind == desc.kind && el.created_by_layout {
                if Arc::ptr_eq(&el.reference, &reference) && Arc::ptr_eq(&el.desc, &desc) {
                    let count = self.update_count;
                    self.el_mut(child).latest_update = count;
                    return;
                }
                // `SwitchDescription`.
                let el = self.el_mut(child);
                el.reference = reference;
                el.desc = desc;
                self.reset_media(child);
                self.update_internal(child);
                self.set_parent(child, Some(index));
                return;
            }
            self.set_parent(child, None);
            self.delete(child);
        }
        if let Some(child) = self.create_element(layout, reference, false) {
            self.el_mut(child).created_by_layout = true;
            self.set_parent(child, Some(index));
        }
    }

    /// `UIElement::SetParent`: out of the old parent's children, into the new one's by z.
    pub(crate) fn set_parent(&mut self, index: usize, parent: Option<usize>) {
        if let Some(old) = self.el(index).parent {
            self.el_mut(old).children.retain(|&c| c != index);
        }
        self.el_mut(index).parent = parent;
        if let Some(parent) = parent {
            let at = self.z_slot(&self.el(parent).children, self.el(index).z);
            self.el_mut(parent).children.insert(at, index);
        }
        // `UIElement_Meter::SetParent`.
        self.meter_update_layout(index);
    }

    /// `UIElement::AddChild`'s position for z `z` in an ascending list: last when `0xFFFF` or
    /// above the last, else before the first entry whose z is not below it.
    pub(crate) fn z_slot(&self, list: &[usize], z: i32) -> usize {
        match list.last() {
            None => 0,
            Some(&last) if z == 0xFFFF || self.el(last).z < z => list.len(),
            Some(_) => list
                .iter()
                .position(|&other| z <= self.el(other).z)
                .unwrap_or(list.len()),
        }
    }

    /// Adds a root in z order (`IUIManager` root list).
    pub(crate) fn add_root(&mut self, index: usize) {
        self.roots.retain(|&r| r != index);
        let at = self.z_slot(&self.roots, self.el(index).z);
        self.roots.insert(at, index);
    }
}
