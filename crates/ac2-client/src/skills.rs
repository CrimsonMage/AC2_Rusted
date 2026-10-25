//! The skill panel's requests on the player's script (`ggmCWInterface::CallIFun_Request*Skill`):
//! each IFun's stub packs the skill and sends the `WM_Skill` request, so the bytes are the
//! script's (`ac2_weenie::SkillRequest::call` is the same call, kept for the native path).

use ac2_script::host::IFunArg;
use ac2_weenie::{SessionEvent, SkillRequest};

use crate::client::Client;
use crate::scripts::*;

impl Client {
    /// `gmCWInterface::RequestTrainSkill`, `RequestUntrainSkill`, `RequestUntrainSkillCancel` and
    /// `RequestRaiseSkill`: the request's IFun on the player's script; the events it raised come
    /// back. Without the script VM nothing is sent.
    pub(crate) fn skill_command(&mut self, request: SkillRequest) -> Vec<SessionEvent> {
        match request {
            SkillRequest::Train(skill) => {
                self.script_request(IFUN_REQUEST_TRAIN_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::Untrain(skill) => {
                self.script_request(IFUN_REQUEST_UNTRAIN_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::Raise(skill) => {
                self.script_request(IFUN_REQUEST_RAISE_SKILL, &[IFunArg::Int(skill)])
            }
            SkillRequest::CancelUntrain => {
                self.script_request(IFUN_REQUEST_UNTRAIN_SKILL_CANCEL, &[])
            }
        }
    }
}
