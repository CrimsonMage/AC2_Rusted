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

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, StateDesc};
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
    /// `UIElement_PanelPage::page`: the tab's content.
    pub(crate) page: Option<usize>,
    /// `UIElement_Tree`'s state, once items are added.
    pub(crate) tree: Option<Box<crate::tree_view::TreeState>>,
    /// `m_mouseOver`, `m_mouseOverTop`, `m_mouseVisible` and the buttons pressed on the element
    /// (`m_mouseDownTable`).
    pub(crate) mouse_over: bool,
    pub(crate) mouse_over_top: bool,
    pub(crate) mouse_visible: bool,
    pub(crate) mouse_down: Vec<u32>,
    /// `UIElement_Button::mousePressedOnButton`.
    pub(crate) button_pressed: bool,
    /// A root's surface object: shown (`UIObject` show, apart from the element's own
    /// visibility) and rotated about its centre in degrees (`UISurfaceObject::SetRotateVals`).
    pub(crate) shown: bool,
    pub(crate) rotation: i32,
    media_state: Option<(Arc<ElementDesc>, usize)>,
    media_index: usize,
    latest_update: u32,
    created_by_layout: bool,
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
            text_data: None,
            messages: Vec::new(),
            pointer: (0, 0),
            mouse_top: None,
            changed: true,
            moved: true,
            active: None,
        }
    }

    /// The fonts and strings text elements made from now on draw from.
    pub fn set_text_data(&mut self, data: Arc<dyn UiTextData>) {
        self.text_data = Some(data);
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

    /// Frees an element and its descendants (`AddElementToDeleteQueue`).
    pub(crate) fn delete(&mut self, index: usize) {
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
            page: None,
            mouse_over: false,
            mouse_over_top: false,
            mouse_visible: true,
            mouse_down: Vec::new(),
            button_pressed: false,
            shown: true,
            rotation: 0,
            media_state: None,
            media_index: 0,
            latest_update: self.update_count,
            created_by_layout: false,
        });
        self.set_state(index, default_state);
        self.update_count += 1;
        self.update_internal(index);
        self.set_visible(index, true);
        Some(index)
    }

    /// `UIElement::SetVisible` (`UIRegion::SetVisible`).
    pub fn set_visible(&mut self, index: usize, visible: bool) {
        self.el_mut(index).visible = visible;
        self.changed = true;
    }

    /// Whether anything drawn changed since the last call.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// Whether a root's surface object moved, turned or showed since the last call.
    pub fn take_moved(&mut self) -> bool {
        std::mem::take(&mut self.moved)
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
    pub fn placements(&self) -> Vec<(usize, i32, i32, i32, bool)> {
        self.composed_roots()
            .map(|root| {
                let el = self.el(root);
                let (x, y) = el.object.unwrap_or((0, 0));
                (root, x, y, el.rotation, el.shown)
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
        }
        self.text_attribute(index, id, &AttributeValue::Bool(value), changed);
    }

    /// `UIElement::SetAttributeFromDesc`.
    fn set_attribute(&mut self, index: usize, attribute: &AttributeDesc) {
        match attribute.value {
            AttributeValue::Bool(value) => self.set_bool(index, attribute.id, value),
            AttributeValue::None(_) => {}
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

    /// `UIElement::SetState`.
    pub fn set_state(&mut self, index: usize, state: u32) {
        self.button_set_state(index, state);
        if state == self.el(index).state {
            return;
        }
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
    }

    /// `UIElement::ResetMedia`: the media of the current state, else the stateless state's, else
    /// (for a reference) the reference's current then stateless state's; then runs them.
    fn reset_media(&mut self, index: usize) {
        let el = self.el(index);
        let pick = |desc: &Arc<ElementDesc>, id: u32| {
            desc.states
                .iter()
                .position(|s| s.id == id && !s.media.is_empty())
                .map(|i| (Arc::clone(desc), i))
        };
        let state = el.state;
        let current = (state != STATELESS)
            .then(|| pick(&el.desc, state))
            .flatten();
        let media = current.or_else(|| pick(&el.desc, STATELESS)).or_else(|| {
            if Arc::ptr_eq(&el.reference, &el.desc) {
                return None;
            }
            (state != STATELESS)
                .then(|| pick(&el.reference, state))
                .flatten()
                .or_else(|| pick(&el.reference, STATELESS))
        });
        let el = self.el_mut(index);
        el.media_state = media;
        el.media_index = 0;
        self.update_media(index);
    }

    /// `UIElement::UpdateMedia` at the run's start: media in order until one waits.
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
    }

    fn update_media(&mut self, index: usize) {
        loop {
            let el = self.el(index);
            let Some((desc, state)) = el.media_state.clone() else {
                return;
            };
            let Some(media) = desc.states[state].media.get(el.media_index) else {
                return;
            };
            match media {
                MediaDesc::Image {
                    file, draw_mode, ..
                } => {
                    let (file, draw_mode) = (*file, *draw_mode);
                    self.set_image(index, file, draw_mode);
                }
                MediaDesc::Anim {
                    draw_mode, frames, ..
                } => {
                    if let Some(&file) = frames.first() {
                        let draw_mode = *draw_mode;
                        self.set_image(index, file, draw_mode);
                    }
                    if frames.len() > 1 {
                        return;
                    }
                }
                MediaDesc::Alpha { file, .. } => self.el_mut(index).alpha_image = Some(*file),
                MediaDesc::Pause { .. } => return,
                MediaDesc::State { state } => {
                    let state = *state;
                    self.set_state(index, state);
                    return;
                }
                _ => {}
            }
            self.el_mut(index).media_index += 1;
        }
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
