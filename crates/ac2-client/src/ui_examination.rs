//! The examination panel's effect nodes (`gmExaminationUI::ProcessEffectIDNode`): an effect id
//! expands into the nodes the player's script adds for it
//! (`gmCWInterface::AddEffectDataToProfileByEID`, `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`),
//! which take the effect node's place in the profile. The expansion's own effect nodes are not
//! expanded again.

use ac2_net::protocol::{ExaminationDataNode, ExaminationRequest, NativeObject};
use ac2_script::host::interface_calls::ids;
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::{ntype, NativeObject as ScriptNative};
use ac2_weenie::SessionEvent;

use crate::client::Client;

/// `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`'s function (`0xB40123`, called on the
/// player's process with the context, order and effect id, then the profile the script fills).
pub const IFUN_ADD_EFFECT_DATA_TO_PROFILE: u32 = 0x00B4_0123;

impl Client {
    /// Whether the current examination request is not an object's: an effect of the status bar
    /// (type `0x40000001`), whose profile the player's script builds from its own data
    /// (`CExaminationSystem`) where an object's comes from the server.
    fn examining_locally(&mut self) -> bool {
        let kind = self.session.player_script().examination_request().kind;
        !matches!(
            kind,
            0 | ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
        ) && self.scripts_own(ids::SET_EXAMINATION_REQUEST)
    }

    /// `gmCWInterface::SetExaminationRequest`. The session's stand-in keeps the request and asks
    /// the server for an object's profile; a request of another kind goes to the player's script
    /// too (`CPlayer` 0xE6), which restarts a running examination on it.
    pub(crate) fn examine_request(&mut self, request: ExaminationRequest) -> Vec<SessionEvent> {
        let mut events = self.session.examine(request);
        if self.examining_locally() {
            events.extend(self.script_ifun_args(
                ids::SET_EXAMINATION_REQUEST,
                &[IFunArg::Native(NativeObject::ExaminationRequest(request))],
            ));
        }
        events
    }

    /// `gmCWInterface::StartExamination`: the panel opened with `context` (its filter word). For
    /// an effect the player's script builds the profile now (the effect's name and description,
    /// its end time as a countdown) and raises it as a notice.
    pub(crate) fn start_examination_request(&mut self, context: u32) -> Vec<SessionEvent> {
        let mut events = self.session.start_examination(context);
        if self.examining_locally() {
            events.extend(self.script_ifun_args(ids::START_EXAMINATION, &[IFunArg::Int(context)]));
        }
        events
    }

    /// `gmCWInterface::StopExamination`: the panel closed; the player's script stops building the
    /// profile of a request it had (`CPlayer` 0xF6).
    pub(crate) fn stop_examination_request(&mut self) -> Vec<SessionEvent> {
        let local = self.examining_locally();
        let mut events = self.session.stop_examination();
        if local {
            events.extend(self.script_ifun_args(ids::STOP_EXAMINATION, &[]));
        }
        events
    }

    /// `gmCWInterface::AddEffectDataToProfileByEID`: the script fills an empty profile (the
    /// fourth parameter) with the effect's nodes.
    pub(crate) fn add_effect_data(
        &mut self,
        context: u32,
        order: u32,
        eid: u32,
    ) -> Vec<ExaminationDataNode> {
        let args = [
            IFunArg::Int(context),
            IFunArg::Int(order),
            IFunArg::Int(eid),
            IFunArg::Filled(ntype::EXAMINATION_PROFILE),
        ];
        let Ok(result) = self.query(IFUN_ADD_EFFECT_DATA_TO_PROFILE, &args, IFunReturn::Word)
        else {
            return Vec::new();
        };
        match result.outs.get(3) {
            Some(Some(OutValue::Vm(ScriptNative::Wire(profile)))) => match &**profile {
                NativeObject::ExaminationProfile(p) => p.nodes.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}
