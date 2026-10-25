//! What the script VM produces, routed: server calls (`gCWInterface::SendServerEvt`), typed
//! notices into the frame's events, the other `CM_*` notices for the UI, room chat
//! (`gmCWInterface::RequestGUN_SendToRoom` on the Turbine chat path), and the DAT reads a call
//! stopped for, handed to the loader threads. Also the UI's entries into the VM: IFun calls
//! (`ggmCWInterface::CallIFun_*`) and the skill panels (`gSkillPanelManager`).

use ac2_net::protocol::pack_values;
use ac2_script::host::{
    ChatRequest, DispatchOutcome, IFunArg, IFunResult, ScriptNotice, SkillPanelData,
};
use ac2_script::vm::InterpStatus;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::loader::Request;

impl Client {
    /// The end of one dispatch, resume or IFun: problems noted, a call that stopped for data
    /// remembered, the output sent and shown when the call is the VM's (set aside when a
    /// stand-in already answered it), and the reads it needs handed to the loader.
    pub(crate) fn script_outcome(
        &mut self,
        id: u64,
        function: Option<u32>,
        result: Result<DispatchOutcome, String>,
        out: &mut Vec<SessionEvent>,
    ) {
        let owned = function.is_none_or(|f| !self.session.stand_in_handles(f));
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        let call = function.map_or("IFun".to_owned(), |f| format!("{f:#010x}"));
        match result {
            Err(error) => scripts.problems.push(format!("{id:#018x} {call}: {error}")),
            Ok(outcome) if outcome.fault.is_some() => scripts.problems.push(format!(
                "{id:#018x} {call}: {:?} {:?}",
                outcome.status, outcome.fault
            )),
            Ok(outcome) if outcome.status == InterpStatus::Interrupted => {
                scripts.data_waits.insert(outcome.context, (id, function));
            }
            Ok(_) => {}
        }
        let calls = scripts.host.take_server_calls();
        let notices = scripts.host.take_notices();
        let script_notices = scripts.host.take_script_notices();
        let chat = scripts.host.take_chat_requests();
        let dropped = scripts.host.take_dropped();
        scripts.problems.extend(dropped);
        let reads = scripts.host.take_data_requests();
        scripts.data_pending += reads.len();
        for read in reads {
            self.loader.request(Request::ScriptData(read));
        }
        if !owned {
            return;
        }
        for call in calls {
            self.session
                .send_interp(call.function, pack_values(&call.args), call.encrypt);
        }
        for request in chat {
            match request {
                ChatRequest::SendToRoom { room, text, flags } => {
                    self.session.send_to_room(room, &text, flags);
                }
                // `set_chat_services` says there is no GUN, so these never queue.
                other => {
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts.problems.push(format!("no GUN for {other:?}"));
                    }
                }
            }
        }
        out.extend(notices.into_iter().map(SessionEvent::Notice));
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.script_notices.extend(script_notices);
        }
    }

    /// `ggmCWInterface::CallIFun_*` on the player's process, its output routed into `out`.
    pub(crate) fn ifun_into(
        &mut self,
        function: u32,
        args: &[IFunArg],
        out: &mut Vec<SessionEvent>,
    ) -> Result<IFunResult, String> {
        let player = self.session.player().ok_or("no player")?;
        let scripts = self.scripts.as_mut().ok_or("no script VM")?;
        let result = scripts
            .host
            .call_ifun(player, function, args)
            .map_err(|e| format!("{e:?}"));
        let outcome = result
            .as_ref()
            .map(|r| r.outcome.clone())
            .map_err(Clone::clone);
        self.script_outcome(player, None, outcome, out);
        result
    }

    /// A UI call into the player's script (`ggmCWInterface::CallIFun_*`: `function` with
    /// `args`, e.g. `SetSelection` 0xB40002 with the object's id). Its server calls go out now;
    /// its notices join the next frame's events. A call that stops for data finishes when the
    /// data lands, and its `retval` is then not read.
    pub fn call_ifun(&mut self, function: u32, args: &[IFunArg]) -> Result<IFunResult, String> {
        let mut out = Vec::new();
        let result = self.ifun_into(function, args, &mut out);
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(out);
        }
        result
    }

    /// `gSkillPanelManager`'s data for skill panel `panel` (`gmSkillTreeUI::LoadPanel`): its
    /// name, icon and nodes, from the scripts.
    pub fn skill_panel(&mut self, panel: u32) -> Result<Option<SkillPanelData>, String> {
        let scripts = self.scripts.as_mut().ok_or("no script VM")?;
        scripts
            .host
            .skill_panel(panel)
            .map_err(|e| format!("{e:?}"))
    }

    /// The `CM_*` notices without a `GameplayNotice` type the VM raised since the last call
    /// (for calls that are the VM's), in order.
    pub fn take_script_notices(&mut self) -> Vec<ScriptNotice> {
        self.scripts
            .as_mut()
            .map(|s| std::mem::take(&mut s.script_notices))
            .unwrap_or_default()
    }
}
