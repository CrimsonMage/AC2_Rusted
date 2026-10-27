//! The yes/no question a script asks the player: `ggmCWInterface::CallEFun_PopupYesNoDialog`
//! (efun `0xB400F5`) raises `gmCWInterface::PopupYesNoDialog`, which hands the question to the
//! UI, and the context stops until the answer. The answer comes back through
//! [`ScriptHost::answer_popup`], which is `ggmCWInterface::Resume_PopupYesNoDialog`: the out
//! parameter takes the answer (an int, 1 for yes), the efun's frame closes and the context runs on.

use ac2_net::protocol as net;
use ac2_weenie::GameplayNotice;

use super::{DispatchOutcome, ScriptHost};
use crate::glue::word;
use crate::vm::{tag, Context, EFuns, Fault, InterpStatus, Vm};
use crate::wire;

/// `ggmCWInterface::CallEFun_PopupYesNoDialog`: the efun the context waits in while the
/// question is open (`m_efun`).
pub(super) const POPUP_YES_NO: u32 = 0x00B4_00F5;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(POPUP_YES_NO, popup_yes_no);
}

/// The frame, as `CallEFun_PopupYesNoDialog` reads it: `this` (the interface), the
/// `StringInfo` question, then the int out parameter for the answer.
fn popup_yes_no(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 3);
    let Some(text) = string_info(vm, word(ctx, 2)) else {
        // A question the wire cannot carry is dropped, and the answer is "no".
        vm.outbox
            .dropped
            .push("PopupYesNoDialog: the question has no wire form".to_owned());
        vm.write_through(ctx, out, 0, tag::INT);
        return InterpStatus::Success;
    };
    let object = vm.owners.get(&word(ctx, 1)).copied().unwrap_or(0);
    vm.outbox.notices.push(GameplayNotice::PopupYesNo {
        object,
        context: ctx.cid,
        text,
    });
    // `gmCWInterface::PopupYesNoDialog` leaves the status Interrupted: the frame stays open.
    InterpStatus::Interrupted
}

fn string_info(vm: &Vm, reference: u32) -> Option<net::StringInfo> {
    let object = vm.refs.heap(reference).ok()?;
    match wire::native_to_net(object.native_type, object.native.as_ref()?).ok()? {
        net::NativeObject::StringInfo(text) => Some(text),
        _ => None,
    }
}

impl ScriptHost {
    /// `ggmCWInterface::Resume_PopupYesNoDialog(obj, cid, userResponse)`: the context `context`
    /// of object `id` takes the answer when it waits in the popup efun. Its out parameter gets
    /// `yes` as 1 or 0, the efun's frame closes and the context runs on. `None` when no such
    /// context waits there (retail returns 0 and changes nothing).
    pub fn answer_popup(&mut self, id: u64, context: u32, yes: bool) -> Option<DispatchOutcome> {
        let process = self.processes.get_mut(&id)?;
        let at = process
            .waiting
            .iter()
            .position(|c| c.cid == context && c.efun == POPUP_YES_NO)?;
        let mut ctx = process.waiting.remove(at);
        let out = ctx.arg(3)?;
        ctx.efun = 0;
        self.vm.last_fault = None;
        self.vm
            .write_through(&mut ctx, out, u32::from(yes), tag::INT);
        let status = match ctx.leave() {
            Ok(()) => self.vm.execute(&mut ctx),
            Err(_) => self.vm.fault(Fault::Stack),
        };
        Some(self.finish(id, ctx, status))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_net::protocol::StringInfo;

    use super::*;
    use crate::host::ScriptHost;
    use crate::native::{ntype, NativeObject};
    use crate::vm::refs::{self, Target};
    use crate::vm::{NoData, Program, ReferenceTable};

    fn host() -> ScriptHost {
        let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
        ScriptHost::new(Arc::new(program), Box::new(NoData))
    }

    fn question(vm: &mut Vm, literal: &str) -> u32 {
        let info = StringInfo {
            string_id: 0,
            table_id: 0,
            literal: Some(literal.to_owned()),
            variables: Vec::new(),
        };
        let native = NativeObject::StringInfo(wire::string_info_from_net(&info));
        vm.refs.create_native(ntype::STRING_INFO, native)
    }

    /// An int slot a script passes as an out parameter (a one-word heap instance).
    fn int_slot(vm: &mut Vm) -> u32 {
        let object = ReferenceTable::new_instance(0, 1);
        vm.refs.add(
            refs::flags::FILLED | refs::flags::KIND_HEAP,
            Target::Heap(Box::new(object)),
        )
    }

    /// The efun's frame as the VM opens it: the return slot, then the arguments (`si`, out) and
    /// `this` on top.
    fn open_frame(ctx: &mut Context, this: u32, si: u32, out: u32) {
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        ctx.push(out, tag::REF).unwrap();
        ctx.push(si, tag::REF).unwrap();
        ctx.push(this, tag::REF).unwrap();
    }

    #[test]
    fn the_question_is_raised_and_the_context_waits_for_the_answer() {
        let mut host = host();
        let this = host.vm.refs.create_native(
            ntype::PACK_BUFFER,
            NativeObject::new_empty(ntype::PACK_BUFFER),
        );
        host.vm.owners.insert(this, 0x5000_0009);
        let si = question(&mut host.vm, "Accept the quest?");
        let out = int_slot(&mut host.vm);
        let mut ctx = Context::new(0x44, Some(this));
        open_frame(&mut ctx, this, si, out);
        let status = host.vm.run_efun(POPUP_YES_NO, &mut ctx);
        assert_eq!(status, InterpStatus::Interrupted);
        assert_eq!(ctx.efun, POPUP_YES_NO);
        let notices = host.take_notices();
        assert_eq!(notices.len(), 1);
        let GameplayNotice::PopupYesNo {
            object,
            context,
            text,
        } = &notices[0]
        else {
            panic!("a popup notice, not {:?}", notices[0]);
        };
        assert_eq!((*object, *context), (0x5000_0009, 0x44));
        assert_eq!(text.literal.as_deref(), Some("Accept the quest?"));
    }

    #[test]
    fn the_answer_writes_the_out_parameter_and_runs_the_context_on() {
        let mut host = host();
        let this = host.vm.refs.create_native(
            ntype::PACK_BUFFER,
            NativeObject::new_empty(ntype::PACK_BUFFER),
        );
        host.vm.owners.insert(this, 0x5000_0009);
        let si = question(&mut host.vm, "Leave the instance?");
        let out = int_slot(&mut host.vm);
        let mut ctx = Context::new(0x51, Some(this));
        open_frame(&mut ctx, this, si, out);
        assert_eq!(
            host.vm.run_efun(POPUP_YES_NO, &mut ctx),
            InterpStatus::Interrupted
        );
        let id = 0x5000_0009;
        host.processes.insert(
            id,
            super::super::Process {
                instance: this,
                waiting: vec![ctx],
            },
        );

        assert!(
            host.answer_popup(id, 0x99, true).is_none(),
            "another context"
        );
        let outcome = host
            .answer_popup(id, 0x51, true)
            .expect("the context waits");
        assert_eq!(outcome.status, InterpStatus::Terminated);
        assert_eq!(outcome.context, 0x51);
        assert!(!outcome.data_wait);
        assert_eq!(host.vm.read_through(&Context::new(0, None), out), Some(1));
        // Answered once: the context is no longer waiting in the efun.
        assert!(host.answer_popup(id, 0x51, false).is_none());
    }

    #[test]
    fn a_no_answer_writes_zero() {
        let mut host = host();
        let this = host.vm.refs.create_native(
            ntype::PACK_BUFFER,
            NativeObject::new_empty(ntype::PACK_BUFFER),
        );
        host.vm.owners.insert(this, 0x5000_0009);
        let si = question(&mut host.vm, "Decline?");
        let out = int_slot(&mut host.vm);
        let mut ctx = Context::new(0x52, Some(this));
        open_frame(&mut ctx, this, si, out);
        host.vm.run_efun(POPUP_YES_NO, &mut ctx);
        host.processes.insert(
            0x5000_0009,
            super::super::Process {
                instance: this,
                waiting: vec![ctx],
            },
        );
        let outcome = host.answer_popup(0x5000_0009, 0x52, false).unwrap();
        assert_eq!(outcome.status, InterpStatus::Terminated);
        assert_eq!(host.vm.read_through(&Context::new(0, None), out), Some(0));
    }

    #[test]
    fn a_context_that_is_not_in_the_popup_efun_is_not_resumed() {
        let mut host = host();
        let this = host.vm.refs.create_native(
            ntype::PACK_BUFFER,
            NativeObject::new_empty(ntype::PACK_BUFFER),
        );
        let mut ctx = Context::new(0x60, Some(this));
        ctx.enter(0).unwrap();
        host.processes.insert(
            0x5000_0009,
            super::super::Process {
                instance: this,
                waiting: vec![ctx],
            },
        );
        assert!(host.answer_popup(0x5000_0009, 0x60, true).is_none());
        assert_eq!(host.processes[&0x5000_0009].waiting.len(), 1);
    }
}
