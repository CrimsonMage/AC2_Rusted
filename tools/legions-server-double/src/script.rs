//! The double's gameplay side: the script calls the client sends (`InterpSEvent`), decoded
//! with the client's own codec and answered as a server's `WM_*` handlers would, from data a
//! test configures.

use std::collections::HashMap;

use ac2_net::protocol::{
    ExaminationProfile, ExaminationRequest, ScriptMessage, ScriptRequest, SelectionInfo,
};

use crate::State;

/// What the double answers script calls with.
#[derive(Debug, Clone, Default)]
pub struct ScriptWorld {
    /// `SelectionInfo` sent back (`WM_Player` 0x2C) when the client targets the object.
    pub selection_infos: HashMap<u64, SelectionInfo>,
    /// The examination profile sent back (`WM_Examination` 0x9) for an object request.
    pub profiles: HashMap<u64, ExaminationProfile>,
}

impl State {
    /// An `InterpSEvent` (0x10053) or its encrypted twin (0x100D0) arrived: decode it, record
    /// it, answer it.
    pub(crate) fn script_event(&mut self, data: &[u8]) {
        let word = |at: usize| {
            data.get(at..at + 4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        };
        let (Some(function), Some(size)) = (word(4), word(8)) else {
            self.script_errors
                .push(format!("short script event {data:02x?}"));
            return;
        };
        let arguments = &data[12..];
        if arguments.len() != size as usize {
            self.script_errors.push(format!(
                "PackBuffer size {size} for {} bytes",
                arguments.len()
            ));
            return;
        }
        match ScriptRequest::decode(function, arguments) {
            Some(Ok(request)) => {
                self.answer_script(&request);
                self.script_requests.push(request);
            }
            Some(Err(error)) => self
                .script_errors
                .push(format!("function {function:#010x}: {error}")),
            None => self.script_unknown.push((function, arguments.to_vec())),
        }
    }

    fn answer_script(&mut self, request: &ScriptRequest) {
        match request {
            ScriptRequest::SetTarget(id) => {
                if let Some(info) = self.script_world.selection_infos.get(id).copied() {
                    self.send_script(&ScriptMessage::UpdateSelectionInfo { info, id: *id });
                }
            }
            ScriptRequest::Examine(request) => {
                let object = matches!(
                    request.kind,
                    ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
                );
                if let Some(profile) = object
                    .then(|| self.script_world.profiles.get(&request.iid).cloned())
                    .flatten()
                {
                    let profile = ExaminationProfile {
                        request: *request,
                        ..profile
                    };
                    self.send_script(&ScriptMessage::ExaminationProfile(profile));
                }
            }
        }
    }

    /// Queues a script call to the player (`InterpCEvent_Broadcast`, 0x10066).
    pub fn send_script(&mut self, message: &ScriptMessage) {
        self.send_event(&message.to_broadcast());
    }
}
