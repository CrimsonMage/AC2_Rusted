//! UI-initiated calls into a weenie's script (`ggmCWInterface::CallIFun_*`, e.g. selecting,
//! using, setting a shortcut, the skill panel's questions): a new context on the object's
//! process, then `VM::CallIFun(instance package, fid)`, and the values read back.
//!
//! The wrappers' frame, bottom up: each out parameter's current value in reverse parameter
//! order (a word tagged 4, or a new native, tagged 1), the return slot (one 0 word tagged 2,
//! or two words tagged 0x82 for a `ulong64`), then the parameters in reverse order (so word 2
//! is the first) - an out parameter as the stack address of its value (`KIND_VALUE | slot <<
//! 10`) - and the instance. After the call the return slot and the out values are read (a
//! native out takes the object the slot then names).

use ac2_net::protocol as net;

use super::{DispatchOutcome, HostError, ScriptHost};
use crate::native::{ntype, NativeObject, PlainKeys, RefKey, ScriptHash};
use crate::vm::refs::{flags, Target};
use crate::vm::{tag, Context, FunctionId};
use crate::wire;

/// One parameter of a `CallIFun_*` wrapper.
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
    /// An out parameter (`ulong *`, `float *`, `StringInfo *`, ...) with its current value.
    Out(OutValue),
    /// A VM native passed by value, as the wrapper's own native of type `.0` (a table the script
    /// reads).
    Vm(u32, NativeObject),
    /// A list of VM natives of type `.0` passed by value: an `RList` (native 0x65) whose items are
    /// new natives of that type (a list the script reads, such as the friends' statuses).
    VmList(u32, Vec<NativeObject>),
    /// An out parameter that is a new empty native of this type (a list the script fills, an
    /// adaptor): the native is the out value, its address the parameter, and it is read back
    /// after the call ([`OutValue::Items`] for a list, [`OutValue::Vm`] otherwise).
    Filled(u32),
}

/// An out parameter's value.
#[derive(Debug, Clone, PartialEq)]
pub enum OutValue {
    /// A word (`ulong`, `int`, a `float`'s bits).
    Word(u32),
    /// A native (`StringInfo`, ...).
    Native(net::NativeObject),
    /// A VM native the call filled (an adaptor the script wrote into).
    Vm(NativeObject),
    /// A list the call filled: the natives its items name, in order (items that are not natives
    /// are left out).
    Items(Vec<NativeObject>),
}

impl IFunArg {
    /// The `HashTable<unsigned long, InstanceID>` a craft's ingredients are passed as
    /// (`IngredientOrdinalHashToALHash`: an `ALHash`, native 8): one entry per slot ordinal.
    pub fn ingredient_table(entries: &[(u32, u64)]) -> Self {
        let mut hash = ScriptHash::default();
        for (ordinal, object) in entries {
            hash.set(RefKey::Value(u64::from(*ordinal)), *object, &PlainKeys);
        }
        IFunArg::Vm(ntype::AL_HASH, NativeObject::Hash(hash))
    }
}

/// The shape of a wrapper's return slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IFunReturn {
    /// `int *retval`: one word.
    Word,
    /// `ulong64 *retval`: two words, low word deeper.
    Long,
}

/// How a UI call ended: its return value and its out parameters' values, in parameter order
/// (`None` for a native out whose slot no longer names a native the wire can carry).
#[derive(Debug, Clone, PartialEq)]
pub struct IFunResult {
    pub outcome: DispatchOutcome,
    pub retval: u64,
    pub outs: Vec<Option<OutValue>>,
}

impl IFunResult {
    /// Out parameter `n` as a word (0 when it is not one).
    pub fn out_word(&self, n: usize) -> u32 {
        match self.outs.get(n) {
            Some(Some(OutValue::Word(v))) => *v,
            _ => 0,
        }
    }

    /// Out parameter `n` as a native.
    pub fn out_native(&self, n: usize) -> Option<&net::NativeObject> {
        match self.outs.get(n) {
            Some(Some(OutValue::Native(v))) => Some(v),
            _ => None,
        }
    }
}

impl ScriptHost {
    /// `ggmCWInterface::CallIFun_*` with an `int *retval`; see [`ScriptHost::query`].
    pub fn call_ifun(
        &mut self,
        id: u64,
        function: u32,
        args: &[IFunArg],
    ) -> Result<IFunResult, HostError> {
        self.query(id, function, args, IFunReturn::Word)
    }

    /// `ggmCWInterface::CallIFun_*`: calls `function` on `id`'s process with `args` and
    /// reads back the return slot and the out parameters. Retail deletes the context when the
    /// call returns; a context that stops for data here waits like a dispatched one
    /// ([`ScriptHost::resume_data_waits`]) and its values are not read.
    pub fn query(
        &mut self,
        id: u64,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Result<IFunResult, HostError> {
        let bad = |_| HostError::BadArguments;
        let instance = self
            .processes
            .get(&id)
            .ok_or(HostError::UnknownObject(id))?
            .instance;
        let cid = self.next_context;
        self.next_context += 1;
        let mut ctx = Context::new(cid, Some(instance));
        ctx.can_yield = true;
        // The out values, last parameter deepest.
        let mut out_slots = vec![None; args.len()];
        for (i, arg) in args.iter().enumerate().rev() {
            if let IFunArg::Filled(native_type) = arg {
                out_slots[i] = Some(ctx.sp);
                let r = self
                    .vm
                    .refs
                    .create_native(*native_type, NativeObject::new_empty(*native_type));
                ctx.push(r, tag::REF).map_err(bad)?;
            }
            if let IFunArg::Out(value) = arg {
                out_slots[i] = Some(ctx.sp);
                match value {
                    OutValue::Word(v) => ctx.push(*v, tag::BOOL).map_err(bad)?,
                    OutValue::Native(n) => {
                        let r = self.native(n);
                        ctx.push(r, tag::REF).map_err(bad)?;
                    }
                    // Results only: an input is a [`IFunArg::Filled`] or [`IFunArg::Vm`] native.
                    OutValue::Vm(_) | OutValue::Items(_) => return Err(HostError::BadArguments),
                }
            }
        }
        let ret_slot = ctx.sp;
        match ret {
            IFunReturn::Word => ctx.push(0, tag::INT).map_err(bad)?,
            IFunReturn::Long => {
                ctx.push(0, tag::LONG_INT).map_err(bad)?;
                ctx.push(0, tag::LONG_INT).map_err(bad)?;
            }
        }
        for (i, arg) in args.iter().enumerate().rev() {
            match arg {
                IFunArg::Int(v) => ctx.push(*v, tag::BOOL),
                IFunArg::Long(v) => ctx.push64(*v, tag::BOOL | tag::LONG),
                IFunArg::Text(text) => {
                    let r = self.vm.new_wstring(text.encode_utf16().collect());
                    ctx.push(r, tag::REF)
                }
                IFunArg::Native(native) => {
                    let r = self.native(native);
                    ctx.push(r, tag::REF)
                }
                IFunArg::Vm(native_type, native) => {
                    let r = self.vm.refs.create_native(*native_type, native.clone());
                    ctx.push(r, tag::REF)
                }
                IFunArg::VmList(item_type, items) => {
                    let refs = items
                        .iter()
                        .map(|item| u64::from(self.vm.refs.create_native(*item_type, item.clone())))
                        .collect();
                    let list = crate::native::ScriptList {
                        wide: false,
                        items: refs,
                    };
                    let r = self
                        .vm
                        .refs
                        .create_native(ntype::R_LIST, NativeObject::List(list));
                    ctx.push(r, tag::REF)
                }
                IFunArg::Out(_) | IFunArg::Filled(_) => {
                    let slot = out_slots[i].unwrap_or(0) as u32;
                    let blob = flags::KIND_VALUE | ((slot & 0xFFFF) << flags::OFFSET_SHIFT);
                    let address = self.vm.refs.add(blob, Target::Ref(u32::MAX));
                    ctx.push(address, tag::REF)
                }
            }
            .map_err(bad)?;
        }
        ctx.push(instance, tag::REF).map_err(bad)?;
        self.vm.last_fault = None;
        let pkg = self.vm.package_id(instance).unwrap_or(u32::MAX);
        let status = self.vm.call_ifun(&mut ctx, pkg, FunctionId(function));
        let word = |n: usize| ctx.slot(n).map_or(0, |(v, _)| v);
        let retval = match ret {
            IFunReturn::Word => u64::from(word(ret_slot)),
            IFunReturn::Long => u64::from(word(ret_slot)) | u64::from(word(ret_slot + 1)) << 32,
        };
        let outs = args
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| match arg {
                IFunArg::Out(OutValue::Word(_)) => {
                    out_slots[i].map(|s| Some(OutValue::Word(word(s))))
                }
                IFunArg::Out(OutValue::Native(_)) => {
                    let r = word(out_slots[i]?);
                    let native = self.vm.refs.heap(r).ok().and_then(|o| {
                        let n = o.native.as_ref()?;
                        wire::native_to_net(o.native_type, n).ok()
                    });
                    Some(native.map(OutValue::Native))
                }
                IFunArg::Filled(_) => out_slots[i].map(|s| self.read_filled(word(s))),
                _ => None,
            })
            .collect();
        let outcome = self.finish(id, ctx, status);
        Ok(IFunResult {
            outcome,
            retval,
            outs,
        })
    }

    /// A filled native's value after the call: a list's items, or the native itself.
    fn read_filled(&self, r: u32) -> Option<OutValue> {
        let native = self.vm.refs.heap(r).ok()?.native.clone()?;
        Some(match native {
            NativeObject::List(list) => OutValue::Items(
                list.items
                    .iter()
                    .filter_map(|item| self.vm.refs.heap(*item as u32).ok()?.native.clone())
                    .collect(),
            ),
            other => OutValue::Vm(other),
        })
    }

    fn native(&mut self, native: &net::NativeObject) -> u32 {
        let object = wire::native_from_net(native);
        self.vm.refs.create_native(native.native_type(), object)
    }
}
