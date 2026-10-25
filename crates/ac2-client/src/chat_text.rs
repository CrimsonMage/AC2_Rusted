//! The chat's lines as text: `UIElement_Text::AddStringInfowFont` queues a `StringDownload` that
//! fetches the string's table asynchronously and then renders it (`StringInfo::GetString`). Here a
//! worker thread reads the tables on demand and renders; the frame takes the finished lines in
//! the order they were asked for.

use std::sync::mpsc::{self, Receiver, Sender};

use ac2_db::{RenderInfo, RenderVar, ResourceStore, StoreStringTables};
use ac2_net::protocol::{StringInfo, StringVariable};

/// `StringState`'s DID (enum 0x40000001 of group 0x40000004).
const STRING_STATE: u32 = 0x5800_0000;

/// A wire `StringInfo` as the renderer's input (`StringInfo::UnPack`'s variables by stream type).
pub fn render_info(info: &StringInfo) -> RenderInfo {
    let variables = info
        .variables
        .iter()
        .map(|(id, value)| {
            let value = match value {
                StringVariable::Word { kind: 2, value } => RenderVar::Int(*value as i32),
                StringVariable::Word { value, .. } => RenderVar::UInt(*value),
                StringVariable::WordsShort { kind: 3, a, b, c } => RenderVar::FormattedInt {
                    value: *a as i32,
                    grouping: *b as i32,
                    base: *c,
                },
                StringVariable::WordsShort { a, b, c, .. } => RenderVar::FormattedUInt {
                    value: *a,
                    grouping: *b as i32,
                    base: *c,
                },
                StringVariable::LongShort { value, c } => RenderVar::Float {
                    value: f64::from_bits(*value),
                    precision: *c,
                },
                StringVariable::Long5 { value, c, d, e } => RenderVar::FormattedFloat {
                    value: f64::from_bits(*value),
                    precision: *c,
                    grouping: *d as i32,
                    base: *e,
                },
                StringVariable::Nested(inner) => RenderVar::Info(Box::new(render_info(inner))),
                StringVariable::Long { kind: 10, value } => RenderVar::LInt(*value as i64),
                StringVariable::Long { value, .. } => RenderVar::ULInt(*value),
                StringVariable::LongWordShort {
                    kind: 11,
                    value,
                    d,
                    e,
                } => RenderVar::FormattedLInt {
                    value: *value as i64,
                    grouping: *d as i32,
                    base: *e,
                },
                StringVariable::LongWordShort { value, d, e, .. } => RenderVar::FormattedULInt {
                    value: *value,
                    grouping: *d as i32,
                    base: *e,
                },
                StringVariable::Empty { .. } => RenderVar::Skipped,
            };
            (*id, value)
        })
        .collect();
    RenderInfo {
        string_id: info.string_id,
        table_id: info.table_id,
        literal: info.literal.as_ref().map(|s| s.encode_utf16().collect()),
        variables,
    }
}

/// The rendering worker.
pub(crate) struct ChatText {
    send: Sender<(u32, RenderInfo)>,
    receive: Receiver<(u32, String)>,
}

impl ChatText {
    pub(crate) fn new(store: ResourceStore) -> Self {
        let (send, requests) = mpsc::channel::<(u32, RenderInfo)>();
        let (done, receive) = mpsc::channel();
        std::thread::Builder::new()
            .name("ac2 chat text".into())
            .spawn(move || {
                let tables = StoreStringTables::new(store, STRING_STATE);
                for (kind, info) in requests {
                    let text = ac2_db::get_string(&tables, &info, false);
                    if done.send((kind, String::from_utf16_lossy(&text))).is_err() {
                        return;
                    }
                }
            })
            .expect("spawn the chat text worker");
        Self { send, receive }
    }

    /// Queues a line for rendering.
    pub(crate) fn render(&self, kind: u32, info: &StringInfo) {
        let _ = self.send.send((kind, render_info(info)));
    }

    /// The lines rendered since the last call, in order.
    pub(crate) fn take(&self) -> Vec<(u32, String)> {
        self.receive.try_iter().collect()
    }
}
