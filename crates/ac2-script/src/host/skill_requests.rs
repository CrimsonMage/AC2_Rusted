//! The skill panel's requests on the player's script (`ggmCWInterface::CallIFun_Request*Skill`):
//! each IFun's stub packs the skill and sends the `WM_Skill` request, so the bytes are the
//! script's (`ac2_weenie::SkillRequest::call` is the same call, kept for the native path).

use ac2_weenie::{SessionEvent, SkillRequest};

use super::ifun::IFunArg;
use super::ifun_ids::{
    IFUN_REQUEST_RAISE_SKILL, IFUN_REQUEST_TRAIN_SKILL, IFUN_REQUEST_UNTRAIN_SKILL,
    IFUN_REQUEST_UNTRAIN_SKILL_CANCEL,
};
use super::port::ScriptPort;
use super::ui_calls;

/// The skill requests, on any [`ScriptPort`] (the client implements the port).
pub trait SkillRequests: ScriptPort {
    /// `gmCWInterface::RequestTrainSkill`, `RequestUntrainSkill`, `RequestUntrainSkillCancel` and
    /// `RequestRaiseSkill`: the request's IFun on the player's script; the events it raised come
    /// back. Without the script VM nothing is sent.
    fn skill_command(&mut self, request: SkillRequest) -> Vec<SessionEvent> {
        match request {
            SkillRequest::Train(skill) => {
                ui_calls::script_request(self, IFUN_REQUEST_TRAIN_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::Untrain(skill) => {
                ui_calls::script_request(self, IFUN_REQUEST_UNTRAIN_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::Raise(skill) => {
                ui_calls::script_request(self, IFUN_REQUEST_RAISE_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::CancelUntrain => {
                ui_calls::script_request(self, IFUN_REQUEST_UNTRAIN_SKILL_CANCEL, &[])
            }
        }
    }
}

impl<P: ScriptPort + ?Sized> SkillRequests for P {}
