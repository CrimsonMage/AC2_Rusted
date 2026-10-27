//! What the script VM produces, routed: server calls (`gCWInterface::SendServerEvt`), typed
//! notices into the frame's events, the other `CM_*` notices for the UI, room chat
//! (`gmCWInterface::RequestGUN_SendToRoom` on the Turbine chat path), and the DAT reads a call
//! stopped for, handed to the loader threads. Also the UI's entries into the VM: IFun calls
//! (`ggmCWInterface::CallIFun_*`) and the skill panels (`gSkillPanelManager`).

use ac2_net::protocol::pack_values;
use ac2_script::host::{
    ChatRequest, DispatchOutcome, IFunArg, IFunResult, IFunReturn, ScriptNotice, SkillPanelData,
};
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
            // A context stopped for a DAT read runs again when the read lands; one stopped for
            // a question (`PopupYesNoDialog`) waits for its answer.
            Ok(outcome) if outcome.data_wait => {
                scripts.data_waits.insert(outcome.context, (id, function));
            }
            Ok(_) => {}
        }
        let lines = scripts.host.take_shell_lines();
        for (object, flag) in scripts.host.take_no_draw() {
            if flag {
                scripts.no_draw.insert(object);
            } else {
                scripts.no_draw.remove(&object);
            }
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
        // The lines the scripts entered for the chat shell (`CWInterface::ProcessText`), in order.
        for line in lines {
            out.extend(self.session.process_text(&line));
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
                // The GUN service's (friends, ignore list, presence): only queued once
                // `set_gun_service(true)` says the client has one.
                other => {
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts.gun_requests.push(other);
                    }
                }
            }
        }
        out.extend(notices.into_iter().map(SessionEvent::Notice));
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.script_notices.extend(script_notices);
        }
    }

    /// `ggmCWInterface::Resume_PopupYesNoDialog`: the player's answer to a script's question
    /// resumes the context that asked it, and its output goes out with the frame.
    pub(crate) fn answer_popup(
        &mut self,
        object: u64,
        context: u32,
        yes: bool,
    ) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let Some(scripts) = self.scripts.as_mut() else {
            return out;
        };
        // `ggmCWInterface::CallEFun_PopupYesNoDialog` (0xB400F5) is the call the context stopped in.
        let Some(outcome) = scripts.host.answer_popup(object, context, yes) else {
            return out;
        };
        self.script_outcome(object, Some(0x00B4_00F5), Ok(outcome), &mut out);
        out
    }

    /// `ggmCWInterface::CallIFun_*` on the player's process, its output routed into `out`.
    pub(crate) fn ifun_into(
        &mut self,
        function: u32,
        args: &[IFunArg],
        out: &mut Vec<SessionEvent>,
    ) -> Result<IFunResult, String> {
        self.ifun_return_into(function, args, IFunReturn::Word, out)
    }

    /// [`Client::ifun_into`] with the return slot's shape (`ulong64 *retval` takes two words).
    pub(crate) fn ifun_return_into(
        &mut self,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
        out: &mut Vec<SessionEvent>,
    ) -> Result<IFunResult, String> {
        let player = self.session.player().ok_or("no player")?;
        let scripts = self.scripts.as_mut().ok_or("no script VM")?;
        let result = scripts
            .host
            .query(player, function, args, ret)
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

impl Client {
    /// Whether the client has a GUN server (retail: `gmClient::Init` builds
    /// `NSTurbineGun::TurbineGun` only for a `-gunserver` argument, the launcher's, and drops
    /// it when `TurbineGun::Init` fails). Without one, `gmCWInterface::RequestGUN_*` answer 0 and
    /// the player's login (`CPlayer` 0x65, after `RequestGUN_GetIgnoredList`) shows "Cannot
    /// connect to GUN server ..."; with one, the requests come out of
    /// [`Client::take_gun_requests`] for it. Room chat goes to Turbine chat either way.
    pub fn set_gun_service(&mut self, gun: bool) {
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.gun_service = gun;
            scripts
                .host
                .set_chat_services(ac2_script::host::ChatServices {
                    turbine_chat: true,
                    gun,
                });
        }
    }

    /// The GUN requests the scripts made (friends, ignore list, flags, presence, calls) since
    /// the last call, for the GUN service.
    pub fn take_gun_requests(&mut self) -> Vec<ChatRequest> {
        self.scripts
            .as_mut()
            .map(|s| std::mem::take(&mut s.gun_requests))
            .unwrap_or_default()
    }
}
