//! `gmStackUI`: the dialog that asks how many of a stack to move (`StackableField` of the
//! gameplay layout). `RecvNotice_InitStackUI` (from a drop that splits a stack: the backpack,
//! the money bag, a shop, a trade) shows it at the pointer with the requester's question, the
//! default amount in `AmountEntry` (digits only, `NumberInputFilter`) and the most allowed. `OK`
//! reads the amount as `wcstoul` does (base 0: a leading `0` is octal, `0x` hex) and closes with
//! `CM_UI::SendNotice_StackUIDone(context, amount)`, or shows why it cannot (`ID_Stack_UIntPlease`,
//! `ID_Stack_MaxQuantity`); `AmountToggleButton` sets the amount to all or half. The dialog closes
//! without an answer on `Cancel`, `RecvNotice_CloseStackUI` with its context, and when the object
//! the amount comes from goes (`RecvNotice_BeingDeleted`) or, being a container, closes
//! (`RecvNotice_ContainerStateUpdate`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON};
use crate::ui_string::{UiString, UiStrings};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `CM_UI::SendNotice_StackUIDone`: the requester's context and the amount chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackUiDone {
    pub context: u32,
    pub amount: u32,
}

/// `gmStackUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct StackUi {
    /// `m_stackableField`, `m_label`, `m_amountEntry`, `m_errorText`.
    field: Option<usize>,
    label: Option<usize>,
    amount_entry: Option<usize>,
    error_text: Option<usize>,
    default: u32,
    max: u32,
    /// `m_fromID`, `m_targetID`, `m_curContext` (0: not shown).
    from: u64,
    target: u64,
    context: u32,
    /// The modern style's popover (`modern_stack`); `None` in the classic style.
    modern: Option<crate::modern_stack::ModernStack>,
}

/// `NumberInputFilter`: `isdigit`.
fn number_input_filter(ch: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&ch)
}

/// `PSUtils::is_uint32` and the `wcstoul(text, NULL, 0)` read after it: the value when the
/// whole text is one (an empty text reads as 0; out of range is not one).
pub(crate) fn read_uint32(text: &str) -> Option<u32> {
    let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut at = 0;
    while matches!(units[at], 0x09..=0x0D | 0x20) {
        at += 1;
    }
    let negative = units[at] == u16::from(b'-');
    if matches!(units[at], 0x2B | 0x2D) {
        at += 1;
    }
    let digit = |u: u16, base: u32| char::from_u32(u32::from(u)).and_then(|c| c.to_digit(base));
    let mut base = 10;
    if units[at] == u16::from(b'0') {
        if matches!(units[at + 1], 0x58 | 0x78) {
            // `0x` with no hex digit after it is no number.
            digit(units[at + 2], 16)?;
            base = 16;
            at += 2;
        } else {
            base = 8;
        }
    }
    let start = at;
    let mut value: u32 = 0;
    let mut overflow = false;
    while let Some(d) = digit(units[at], base) {
        match value.checked_mul(base).and_then(|v| v.checked_add(d)) {
            Some(v) => value = v,
            None => overflow = true,
        }
        at += 1;
    }
    // No digits: the end is the text's start, so only an empty text ends there.
    if at == start {
        return (units[0] == 0).then_some(0);
    }
    if units[at] != 0 || overflow {
        return None;
    }
    Some(if negative {
        value.wrapping_neg()
    } else {
        value
    })
}

impl UiTree {
    /// `++gmStackUI::m_globalContext`: a new stack dialog context (`ContainerDisplay::
    /// HandleListBoxDrop` and `gmBackpackUI` count on it, admin 0x42de1f).
    pub fn request_stack_context(&mut self) -> u32 {
        self.stack_context += 1;
        self.stack_context
    }
}

impl StackUi {
    /// `gmStackUI::gmStackUI`: the hidden `StackableField` root of the gameplay layout.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let field = tree.create_root(Arc::clone(layout), id("StackableField"));
        let mut ui = Self {
            field,
            ..Self::default()
        };
        if let Some(field) = field {
            ui.label = tree.child_recursive(field, id("Label"));
            ui.amount_entry = tree.child_recursive(field, id("AmountEntry"));
            if let Some(entry) = ui.amount_entry {
                tree.set_text_filter(entry, Some(number_input_filter));
            }
            ui.error_text = tree.child_recursive(field, id("ErrorText"));
            tree.set_visible(field, false);
        }
        ui
    }

    /// [`Self::create`] in `style`: the modern style lays the dialog out as the design's popover
    /// with a slider (`modern_stack`); `layouts` give its small font.
    pub fn create_styled(
        tree: &mut UiTree,
        layout: &Arc<LayoutDesc>,
        style: crate::HudStyle,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Self {
        let mut ui = Self::create(tree, layout);
        if style == crate::HudStyle::Modern {
            ui.modern = ui
                .field
                .map(|field| crate::modern_stack::ModernStack::adopt(tree, field, layouts));
        }
        ui
    }

    /// The shown dialog's context, if one is up.
    pub fn shown(&self) -> Option<u32> {
        (self.context != 0).then_some(self.context)
    }

    /// The dialog's root.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `gmStackUI::RecvNotice_InitStackUI`: a shown dialog closes unanswered, then this one
    /// shows with `text` (the requester's question), `default` in the entry and `max` the most.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        tree: &mut UiTree,
        context: u32,
        text: &str,
        default: u32,
        max: u32,
        from: u64,
        target: u64,
    ) {
        if self.context != 0 {
            self.close(tree, None);
        }
        (self.from, self.target, self.context) = (from, target, context);
        (self.default, self.max) = (default, max);
        if let Some(error) = self.error_text {
            tree.set_text(error, "");
        }
        if let Some(label) = self.label {
            tree.set_text(label, text);
        }
        let toggle = self
            .field
            .and_then(|f| tree.child_recursive(f, id("AmountToggleButton")));
        if let Some(toggle) = toggle {
            tree.set_bool(toggle, BUTTON_HIGHLIGHTED, false);
        }
        self.set_amount(tree, self.default);
        if let Some(modern) = &self.modern {
            modern.init(tree, max);
        }
        self.position_window(tree);
        if let Some(field) = self.field {
            tree.set_visible(field, true);
        }
    }

    /// `gmStackUI::PositionWindow`: centred on the pointer, kept on the display.
    fn position_window(&self, tree: &mut UiTree) {
        let Some(field) = self.field else {
            return;
        };
        let rect = tree.get(field).map(|e| e.rect).unwrap_or_default();
        let (w, h) = (rect.x1 - rect.x0 + 1, rect.y1 - rect.y0 + 1);
        let (px, py) = tree.pointer();
        let (dw, dh) = tree.display;
        let mut x = px - (f64::from(w) * 0.5) as i32;
        let mut y = py - (f64::from(h) * 0.5) as i32;
        if x < 0 {
            x = 0;
        } else if dw < x + w {
            x = dw - w;
        }
        if y < 0 {
            y = 0;
        } else if dh < h + y {
            y = dh - h;
        }
        tree.set_object_position(field, x, y);
    }

    /// `gmStackUI::SetAmount`: the entry shows the amount (`%d`).
    fn set_amount(&self, tree: &mut UiTree, amount: u32) {
        if let Some(entry) = self.amount_entry {
            tree.set_text(entry, &(amount as i32).to_string());
        }
    }

    /// `gmStackUI::Close`: an answer (`ok`) is sent as `StackUIDone`; the dialog hides.
    fn close(&mut self, tree: &mut UiTree, amount: Option<u32>) -> Option<StackUiDone> {
        let done = amount.map(|amount| StackUiDone {
            context: self.context,
            amount,
        });
        (self.context, self.from, self.target) = (0, 0, 0);
        if let Some(field) = self.field {
            tree.set_visible(field, false);
        }
        done
    }

    /// `gmStackUI::RecvNotice_CloseStackUI`.
    pub fn close_stack_ui(&mut self, tree: &mut UiTree, context: u32) {
        if context != 0 && context == self.context {
            self.close(tree, None);
        }
    }

    /// `gmStackUI::RecvNotice_ContainerStateUpdate` (registered on the `from` object): the
    /// `from` container closing closes the dialog (the compare is on `m_fromID`, admin
    /// 0x424b70).
    pub fn container_state_update(&mut self, tree: &mut UiTree, container: u64, open: bool) {
        if self.context != 0 && container == self.from && !open {
            self.close(tree, None);
        }
    }

    /// `gmStackUI::RecvNotice_BeingDeleted` (registered on the `from` object).
    pub fn being_deleted(&mut self, tree: &mut UiTree, object: u64) {
        if self.context != 0 && object != 0 && object == self.from {
            self.close(tree, None);
        }
    }

    /// `gmStackUI::HandleOkButton`.
    fn handle_ok_button(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn UiStrings,
    ) -> Option<StackUiDone> {
        let text = self
            .amount_entry
            .and_then(|e| tree.get(e))
            .and_then(|e| e.text())
            .unwrap_or_default();
        let error = match read_uint32(&text) {
            Some(amount) if amount <= self.max => return self.close(tree, Some(amount)),
            Some(_) => UiString::ui(
                id("ID_Stack_MaxQuantity"),
                vec![(id("size"), SkillVar::UInt(u64::from(self.max)))],
            ),
            None => UiString::ui(id("ID_Stack_UIntPlease"), Vec::new()),
        };
        if let Some(element) = self.error_text {
            tree.set_text(element, &strings.render_string(&error));
        }
        None
    }

    /// `gmStackUI::ListenToMessage`: `OK`, `Cancel` and the all / half toggle.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
        strings: &dyn UiStrings,
    ) -> Option<StackUiDone> {
        // The modern slider: a drag sets the amount (as the toggle does), and the knob follows
        // what the entry holds.
        if self.context != 0 {
            if let Some(modern) = self.modern.clone() {
                if let Some(amount) = modern.message(tree, message, self.max) {
                    self.set_amount(tree, amount);
                    if let Some(error) = self.error_text {
                        tree.set_text(error, "");
                    }
                }
                modern.sync(tree, self.max);
                self.modern = Some(modern);
            }
        }
        if message.message != MSG_BUTTON_RELEASE_ON || self.context == 0 {
            return None;
        }
        match message.element_id {
            e if e == id("StackOkButton") => self.handle_ok_button(tree, strings),
            e if e == id("StackCancelButton") => self.close(tree, None),
            e if e == id("AmountToggleButton") => {
                let half = message
                    .element
                    .is_some_and(|b| tree.bool_attribute(b, BUTTON_HIGHLIGHTED));
                let amount = if half {
                    (f64::from(self.max) * 0.5) as u32
                } else {
                    self.max
                };
                self.set_amount(tree, amount);
                if let Some(error) = self.error_text {
                    tree.set_text(error, "");
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_uint32;

    #[test]
    fn amounts_read_as_wcstoul_base_0() {
        assert_eq!(read_uint32("12"), Some(12));
        assert_eq!(read_uint32(""), Some(0));
        assert_eq!(read_uint32("010"), Some(8));
        assert_eq!(read_uint32("08"), None);
        assert_eq!(read_uint32("0x1f"), Some(31));
        assert_eq!(read_uint32("0x"), None);
        assert_eq!(read_uint32(" 7"), Some(7));
        assert_eq!(read_uint32("7 "), None);
        assert_eq!(read_uint32("-1"), Some(u32::MAX));
        assert_eq!(read_uint32("4294967296"), None);
        assert_eq!(read_uint32("abc"), None);
    }
}
