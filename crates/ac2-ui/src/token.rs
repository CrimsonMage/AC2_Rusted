//! UI names: element types, element ids, state ids, attribute ids and message ids are
//! `Tokenize` hashes of their names (`portal/shared/src/stringtoken`), held in `Token_*` globals.

/// `Tokenize(char *)`: each (signed) byte, shifted to the top seven bits, rotated right by a
/// shift that advances 7 bits per byte, xor-folded.
pub const fn tokenize(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut hash = 0_u32;
    let mut shift = 0_u32;
    let mut i = 0;
    while i < bytes.len() {
        let value = ((bytes[i] as i8 as i32) << 25) as u32;
        hash ^= value >> shift;
        if shift > 25 {
            hash ^= value << ((32 - shift) & 31);
        }
        shift = (shift + 7) & 31;
        i += 1;
    }
    hash
}

/// `Token_reference`: an element that stands for another top-level element of its layout.
pub const REFERENCE: u32 = tokenize("reference");
/// `Token_stateless`: the state every element has besides its current one.
pub const STATELESS: u32 = tokenize("stateless");
/// `Token_referent`: the referenced element's id, an `ID` attribute of a reference's stateless
/// state.
pub const REFERENT: u32 = tokenize("referent");
/// `Token_Element_hide`.
pub const HIDE: u32 = tokenize("hide");
/// `Token_Element_max_width` / `max_height` / `min_width` / `min_height`.
pub const MAX_WIDTH: u32 = tokenize("max_width");
pub const MAX_HEIGHT: u32 = tokenize("max_height");
pub const MIN_WIDTH: u32 = tokenize("min_width");
pub const MIN_HEIGHT: u32 = tokenize("min_height");
/// `Token_Element_root_mover`.
pub const ROOT_MOVER: u32 = tokenize("root_mover");
/// `Token_text`: the `UIElement_Text` class.
pub const TEXT: u32 = tokenize("text");
/// `UIElement_Text` attributes.
pub const TEXT_ENTRY: u32 = tokenize("entry");
pub const TEXT_FONT: u32 = tokenize("font");
pub const TEXT_FONT_RESET: u32 = tokenize("font_reset");
pub const TEXT_TOP_MARGIN: u32 = tokenize("top_margin");
pub const TEXT_BOTTOM_MARGIN: u32 = tokenize("bottom_margin");
pub const TEXT_LEFT_MARGIN: u32 = tokenize("left_margin");
pub const TEXT_RIGHT_MARGIN: u32 = tokenize("right_margin");
pub const TEXT_MAX_CHARS: u32 = tokenize("max_chars");
pub const TEXT_ONE_LINE: u32 = tokenize("one_line");
pub const TEXT_CENTER: u32 = tokenize("center");
pub const TEXT_CENTER_VERTICALLY: u32 = tokenize("center_vertically");
pub const TEXT_RIGHT_JUSTIFY: u32 = tokenize("right_justify");
/// Element messages (`Token_ElementMsg_*`).
pub const MSG_MOUSE_MOVE: u32 = tokenize("mouse_move");
pub const MSG_MOUSE_MOVE_TOP: u32 = tokenize("mouse_move_top");
pub const MSG_MOUSE_OVER: u32 = tokenize("mouse_over");
pub const MSG_MOUSE_OVER_TOP: u32 = tokenize("mouse_over_top");
pub const MSG_MOUSE_PRESS: u32 = tokenize("mouse_press");
pub const MSG_MOUSE_PRESS_TOP: u32 = tokenize("mouse_press_top");
pub const MSG_MOUSE_RELEASE: u32 = tokenize("mouse_release");
pub const MSG_MOUSE_RELEASE_TOP: u32 = tokenize("mouse_release_top");
/// `Token_Element_mouse_event_passthrough`, `Token_Element_mouse_invisible`.
pub const MOUSE_EVENT_PASSTHROUGH: u32 = tokenize("mouse_event_passthrough");
pub const MOUSE_INVISIBLE: u32 = tokenize("mouse_invisible");
/// `Token_button`: the `UIElement_Button` class, its attributes, states and messages.
pub const BUTTON: u32 = tokenize("button");
pub const BUTTON_GHOSTED: u32 = tokenize("ghosted");
pub const BUTTON_HIGHLIGHTED: u32 = tokenize("highlighted");
pub const BUTTON_ROLLOVER: u32 = tokenize("rollover");
pub const BUTTON_RADIO: u32 = tokenize("radio_button");
pub const BUTTON_BOOLEAN: u32 = tokenize("boolean_button");
pub const BUTTON_BROADCAST_WHEN_GHOSTED: u32 = tokenize("broadcast_when_ghosted");
pub const STATE_NORMAL: u32 = tokenize("normal");
pub const STATE_NORMAL_PRESSED: u32 = tokenize("normal_pressed");
pub const STATE_NORMAL_ROLLOVER: u32 = tokenize("normal_rollover");
pub const STATE_HIGHLIGHT: u32 = tokenize("highlight");
pub const STATE_HIGHLIGHT_PRESSED: u32 = tokenize("highlight_pressed");
pub const STATE_HIGHLIGHT_ROLLOVER: u32 = tokenize("highlight_rollover");
pub const STATE_GHOSTED: u32 = tokenize("ghosted");
pub const MSG_BUTTON_RELEASE_ON: u32 = tokenize("mouse_release_on");
pub const MSG_BUTTON_RELEASE_OFF: u32 = tokenize("mouse_release_off");
pub const MSG_HIGHLIGHT_STATE_CHANGE: u32 = tokenize("highlight_state_change");
