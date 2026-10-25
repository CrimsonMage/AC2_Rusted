//! UI-initiated calls into a weenie's script (`ggmCWInterface::CallIFun_*`, e.g. selecting,
//! using, setting a shortcut): a new context on the object's process with a 0 return slot
//! (tag 2), the arguments, the process instance, then `VM::CallIFun(instance package, fid)`;
//! the return slot is read back.

use ac2_net::protocol as net;

use super::{DispatchOutcome, HostError, ScriptHost};
use crate::vm::{tag, Context, FunctionId};
use crate::wire;

/// One value a `CallIFun_*` wrapper pushes.
#[derive(Debug, Clone, PartialEq)]
pub enum IFunArg {
    /// `ulong` / `int` / `bool`: one slot, tag 4.
    Int(u32),
    /// `InstanceID` / `ulong64`: low word then high word, both tagged 0x84.
    Long(u64),
    /// A `PStringBase<unsigned short>`: a new `wpstring` native (`CreateNativeObject(0x76)`).
    Text(String),
    /// Another native (`StringInfo`, `ShortcutInfo`, ...), created as a new native object.
    Native(net::NativeObject),
}

/// How a UI call ended, with its return slot (the `retval` the wrapper reads back).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IFunResult {
    pub outcome: DispatchOutcome,
    pub retval: u32,
}

impl ScriptHost {
    /// `ggmCWInterface::CallIFun_*`: calls `function` on `id`'s process. Retail deletes the
    /// context when the call returns; a context that stops for data here waits like a
    /// dispatched one ([`ScriptHost::resume_data_waits`]) and its `retval` is not read.
    pub fn call_ifun(
        &mut self,
        id: u64,
        function: u32,
        args: &[IFunArg],
    ) -> Result<IFunResult, HostError> {
        let instance = self
            .processes
            .get(&id)
            .ok_or(HostError::UnknownObject(id))?
            .instance;
        let cid = self.next_context;
        self.next_context += 1;
        let mut ctx = Context::new(cid, Some(instance));
        ctx.can_yield = true;
        let ret_slot = ctx.sp;
        ctx.push(0, tag::INT).map_err(|_| HostError::BadArguments)?;
        for arg in args {
            let pushed = match arg {
                IFunArg::Int(v) => ctx.push(*v, tag::BOOL),
                IFunArg::Long(v) => ctx.push64(*v, tag::BOOL | tag::LONG),
                IFunArg::Text(text) => {
                    let r = self.vm.new_wstring(text.encode_utf16().collect());
                    ctx.push(r, tag::REF)
                }
                IFunArg::Native(native) => {
                    let object = wire::native_from_net(native);
                    let r = self.vm.refs.create_native(native.native_type(), object);
                    ctx.push(r, tag::REF)
                }
            };
            pushed.map_err(|_| HostError::BadArguments)?;
        }
        ctx.push(instance, tag::REF)
            .map_err(|_| HostError::BadArguments)?;
        self.vm.last_fault = None;
        let pkg = self.vm.package_id(instance).unwrap_or(u32::MAX);
        let status = self.vm.call_ifun(&mut ctx, pkg, FunctionId(function));
        let retval = ctx.slot(ret_slot).map_or(0, |(v, _)| v);
        let outcome = self.finish(id, ctx, status);
        Ok(IFunResult { outcome, retval })
    }
}
