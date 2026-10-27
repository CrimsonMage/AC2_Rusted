//! The WSL virtual machine (`VM`, `Interp`, `ReferenceTable`, `ProcessState`/`ContextState`).
//!
//! A [`Vm`] owns the resolved [`Program`], the [`ReferenceTable`] and the efun table; script
//! threads are [`Context`]s the caller keeps. Package singletons and saved states come from a
//! [`ScriptData`] the host fills from the DAT cache (decoded on worker threads); the VM never
//! reads files.

mod arith;
pub mod context;
pub mod efun;
mod exec;
mod pack;
pub mod program;
pub mod refs;

pub use context::{Context, Frame, STACK_SLOTS};
pub use efun::{EFunFn, EFunHost, EFuns};
pub use program::{op, FunctionId, Program};
pub use refs::{ReferenceTable, Slot, Target};

use std::ops::Deref;
use std::sync::Arc;

use crate::data::{EnumIdMap, EnumMapper};
use ac2_db::StringTable;

use crate::native::{ntype, GetStringError, NativeObject, StringInfo};
use crate::wstate::{singleton_did, WState, WSTATE_BASE_DID};

/// Slot tags (`ContextState::m_tags`, `HeapObject::m_tags`): the low nibble says what the slot
/// holds, 0x80 marks the slots of a 64-bit value, 0x40 a time value.
pub mod tag {
    pub const REF: u8 = 0x01;
    pub const INT: u8 = 0x02;
    pub const FLOAT: u8 = 0x03;
    pub const BOOL: u8 = 0x04;
    pub const LONG: u8 = 0x80;
    pub const LONG_INT: u8 = 0x82;
    pub const DOUBLE: u8 = 0x83;
    pub const TIME: u8 = 0xC3;
    /// Uninitialised (`0xCD` fill).
    pub const UNINIT: u8 = 0xCD;
}

/// `InterpStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum InterpStatus {
    Undef = 0,
    Terminated = 1,
    Success = 2,
    Interrupted = 3,
    Crash = 4,
    NoIFun = 5,
    Breakpoint = 6,
}

/// `InitState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitState {
    DontInit,
    UseWState,
    NoWState,
}

/// Why a context crashed where retail would have faulted, read freed memory or called a null
/// efun pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// The data stack or call stack ran out of range.
    Stack,
    /// A reference id or slot that does not resolve.
    BadReference(u32),
    /// An opcode with no handler (`handlers[op]` is null).
    UnknownOpcode(u32),
    /// An efun no glue implements yet (retail's dispatcher has a function for it).
    UnboundEFun(u32),
    /// Integer division by zero (x86 `#DE`).
    DivideByZero,
    /// A jump that does not land on an instruction word.
    BadJump,
}

/// A held WState (an `Arc`, or an ac2-db `CachedResource`).
pub type StateRef = Box<dyn Deref<Target = WState>>;

/// What the VM and its glue read from the DATs, served by the host from the ac2-db cache
/// (decoded on worker threads; the VM never reads files).
/// A DAT read a cache-only [`ScriptData`] could not make on the calling thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataRequest {
    WState(u32),
    EnumMapper(u32),
    EnumIdMap(u32),
    StringTable(u32),
    /// The `PlacesTable` (resolved through the enum tables).
    PlacesTable,
    /// An entity description's property collection (`EntityDesc`, DB type 0x40000049).
    EntityDesc(u32),
    /// A `VisualDesc` record (DB type 0x40000021), its appearance.
    VisualDesc(u32),
    /// The class defaults at a qualities DID (`CBaseQualities::m_defaults`, DB type 0x40000019).
    ClassDefaults(u32),
}

/// An entity description's properties in stream order (`PropertyCollection` flattened).
pub type EntityProperties = std::sync::Arc<Vec<ac2_db::Ac2PropertyValue>>;

pub trait ScriptData {
    /// Saved script states by WState DID (`DBObj::Get` in `FillPackage` and
    /// `AllocatePackage`).
    fn wstate(&self, did: u32) -> Option<StateRef>;

    /// Whether a lookup missed since the last call (the data was not resident and is now
    /// requested); clears the flag. The efun that missed waits and runs again once the host
    /// loaded what [`ScriptData::take_requests`] listed.
    fn take_missed(&self) -> bool {
        false
    }

    /// The reads lookups missed since the last call, for a worker thread.
    fn take_requests(&self) -> Vec<DataRequest> {
        Vec::new()
    }

    /// `DBObj::Get` of the entity description at `did`, its properties decoded (`EntityDesc`'s
    /// `m_properties`). `None` when it is not held or not read yet (a cache-only host records a
    /// [`DataRequest::EntityDesc`] and the efun runs again once loaded).
    fn entity_properties(&self, _did: u32) -> Option<EntityProperties> {
        None
    }

    /// `DBObj::Get` of the `VisualDesc` at `did` (DB type 0x40000021) as an appearance. `None`
    /// when it is not held or not read yet (a cache-only host records a
    /// [`DataRequest::VisualDesc`]).
    fn visual_desc(&self, _did: u32) -> Option<std::sync::Arc<ac2_weenie::Appearance>> {
        None
    }

    /// `CBaseQualities::m_defaults`: `DBObj::Get` of the qualities record at `did` (the class
    /// defaults of a weenie with that qualities DID), decoded. `None` when the DATs hold none or
    /// it is not read yet (a cache-only host records a [`DataRequest::ClassDefaults`]).
    fn class_defaults(&self, _did: u32) -> Option<std::sync::Arc<ac2_weenie::WeenieQualities>> {
        None
    }

    /// `FileArray::GetDID(name, db_type)`: a data file's DID by name (the `DBFile2IDTable`).
    fn file_did(&self, _name: &[u8], _db_type: u32) -> Option<u32> {
        None
    }

    /// `FileArray::GetDIDFromEnum(value, db_type)`.
    fn enum_did(&self, _value: u32, _db_type: u32) -> Option<u32> {
        None
    }

    /// `MasterDBMap::DivineType`: the DB type a DID belongs to.
    fn db_type(&self, _did: u32) -> Option<u32> {
        None
    }

    /// `FileArray::GetFileName`: a DID's data file name.
    fn file_name(&self, _did: u32) -> Option<Vec<u8>> {
        None
    }

    /// An `EnumMapper` (`DB_TYPE_ENUM_MAPPER`).
    fn enum_mapper(&self, _did: u32) -> Option<Box<dyn Deref<Target = EnumMapper>>> {
        None
    }

    /// An `EnumIDMap` (`DB_TYPE_DID_MAPPER`).
    fn enum_id_map(&self, _did: u32) -> Option<Box<dyn Deref<Target = EnumIdMap>>> {
        None
    }

    /// The `PlacesTable` (`DBObj::GetByEnum(PlacesTable_UniqueDBID, UNIQUEDB_GROUP_ENUM)`).
    fn places_table(&self) -> Option<Box<dyn Deref<Target = crate::data::PlacesTable>>> {
        None
    }

    /// A `StringTable` (`DB_TYPE_STRING_TABLE`, local DAT).
    fn string_table(&self, _did: u32) -> Option<Box<dyn Deref<Target = StringTable>>> {
        None
    }

    /// The text hook: `StringTable::GetString` (vtable +0x80) of `info`'s string with its
    /// variables, rendered by `StringTableMetaLanguage::RenderString`, meta letters kept (the
    /// glue strips them). The default renders only strings without variables or meta syntax
    /// and reports `gsGenericError` for the rest; a host with the meta-language renderer
    /// overrides it.
    fn render_string(
        &self,
        table: &StringTable,
        info: &StringInfo,
    ) -> Result<Vec<u16>, GetStringError> {
        plain_string(table, info)
    }
}

/// [`ScriptData::render_string`]'s default: a string with no variables and none of the
/// meta-language's block characters is its pieces joined.
pub fn plain_string(table: &StringTable, info: &StringInfo) -> Result<Vec<u16>, GetStringError> {
    const BLOCK: [u16; 6] = [
        b'{' as u16,
        b'}' as u16,
        b'|' as u16,
        b'^' as u16,
        b'!' as u16,
        b'#' as u16,
    ];
    let entry = table
        .get(info.string_id)
        .ok_or(GetStringError::MissingString)?;
    if !entry.variables.is_empty() {
        return Err(GetStringError::GenericError);
    }
    let text: Vec<u16> = entry.pieces.iter().flatten().flatten().copied().collect();
    if text.iter().any(|c| BLOCK.contains(c)) {
        return Err(GetStringError::GenericError);
    }
    Ok(text)
}

/// No DAT data: every package starts from its initializer, no names resolve.
pub struct NoData;

impl ScriptData for NoData {
    fn wstate(&self, _did: u32) -> Option<StateRef> {
        None
    }
}

/// `VM`.
pub struct Vm {
    pub program: Arc<Program>,
    pub refs: ReferenceTable,
    /// `RuntimeByteStream`'s literal references, by literal index.
    pub string_literals: Vec<u32>,
    pub efuns: EFuns,
    states: Box<dyn ScriptData>,
    /// The fault behind the most recent [`InterpStatus::Crash`].
    pub last_fault: Option<Fault>,
    /// `WLogSystem`'s log-type masks.
    pub log_types: LogTypes,
    /// What the scripts produced for the host (server calls, UI notices).
    pub outbox: crate::host::Outbox,
    /// The weenie object that owns each process instance (`HeapObject::m_link` of a process
    /// instance is its `CWDistributor`).
    pub owners: std::collections::HashMap<u32, u64>,
    /// The chat lines the scripts entered for the shell (`CWInterface::ProcessText`, efun
    /// 0x7F0052), in order; the client runs each through its chat shell.
    pub shell_lines: Vec<String>,
    /// The no-draw changes the scripts asked of weenies' physics objects
    /// (`CWPhysicsObject::SetNoDrawPhysics`, efun 0x800057): the object and the flag, in order.
    pub no_draw: Vec<(u64, bool)>,
    /// The physics changes the scripts asked of weenies' physics objects (the scale, FX and
    /// hover setters of `gCWPhysicsObject`), as the server event each one is: the object and
    /// the action, in order. The client applies them to the object's body.
    pub physics: Vec<(u64, ac2_net::protocol::ObjectAction)>,
    /// The latent starts: `gCWPhysicsObject::CallEFun_DoMode` and `CallEFun_DoBehavior` stop
    /// their context until the body reports the mode or behaviour done (`ModeDone` /
    /// `BehaviorDone` with the context). The client starts each on the body with its context.
    pub latent_physics: Vec<crate::host::LatentPhysics>,
    /// Each weenie's `CWDistributor::m_initData` (`WInitData::m_pid`, `m_did`), by object id.
    pub init_data: std::collections::HashMap<u64, (u32, u32)>,
    /// `CObjectMaint::GetPlayerID` (0 before the player exists).
    pub player: u64,
    /// Each weenie's qualities (`CWRepository::m_qualities`), by object id.
    pub repositories: std::collections::HashMap<u64, ac2_weenie::WeenieQualities>,
    /// Each weenie's local visual description (`CPhysicsObj::m_visual`'s local delta, as the
    /// client's object table holds it), by object id: `CWPhysicsObject::GetLocalVisualDesc`
    /// copies it.
    pub appearances: std::collections::HashMap<u64, ac2_weenie::Appearance>,
    /// The chat services `gmCWInterface::RequestGUN_*` can reach.
    pub chat: crate::host::ChatServices,
    /// The world's name as the client holds it (`Turbine::Client`'s, set by the world-name
    /// notice); `gTurbineGunUtils::CallEFun_GetWorldName` answers with it.
    pub world_name: String,
    /// `Random`: retail has one generator for the whole client; a host that runs other
    /// consumers of it on the same stream shares its state through this field.
    pub random: ac2_core::Random,
}

/// `WLogSystem::m_logTypeFile` / `m_logTypePrint` (Legions starts them at 0x1A4 and 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogTypes {
    pub file: u32,
    pub print: u32,
}

impl Default for LogTypes {
    fn default() -> Self {
        Self {
            file: 0x1A4,
            print: 0,
        }
    }
}

impl Vm {
    /// `VM::VM` + `RuntimeByteStream::LoadStringLitTableSection`: each string literal becomes
    /// a `wpstring` native object (`ReferenceTable::CreateStringLiteral`).
    pub fn new(program: Arc<Program>, states: Box<dyn ScriptData>, efuns: EFuns) -> Self {
        let mut refs = ReferenceTable::new();
        let string_literals = program
            .string_literals
            .iter()
            .map(|s| refs.create_native(ntype::WPSTRING, NativeObject::WString(s.clone())))
            .collect();
        Self {
            program,
            refs,
            string_literals,
            efuns,
            states,
            last_fault: None,
            log_types: LogTypes::default(),
            random: ac2_core::Random::default(),
            outbox: Default::default(),
            owners: Default::default(),
            shell_lines: Vec::new(),
            no_draw: Vec::new(),
            physics: Vec::new(),
            latent_physics: Vec::new(),
            init_data: Default::default(),
            player: 0,
            repositories: Default::default(),
            appearances: Default::default(),
            chat: Default::default(),
            world_name: String::new(),
        }
    }

    pub(crate) fn fault(&mut self, fault: Fault) -> InterpStatus {
        self.last_fault = Some(fault);
        InterpStatus::Crash
    }

    /// `VM::Execute` (no debugger, no timers): runs `ctx` until it stops.
    pub fn execute(&mut self, ctx: &mut Context) -> InterpStatus {
        if ctx.efun != 0 {
            match self.resume_efun(ctx) {
                InterpStatus::Success => {}
                status => return status,
            }
        }
        self.exec(ctx)
    }

    /// `VM::CallIFun`: enters `fid` of package `inst_pkg` on `ctx` and runs it. The caller has
    /// pushed the return slot, the arguments and `this`. A missing function runs nothing.
    pub fn call_ifun(&mut self, ctx: &mut Context, inst_pkg: u32, fid: FunctionId) -> InterpStatus {
        if let Some(offset) = self.program.function_offset(inst_pkg, fid) {
            if ctx.enter((offset / 4) as usize).is_err() {
                return self.fault(Fault::Stack);
            }
        }
        self.execute(ctx)
    }

    /// `ReferenceTable::GetPackageID`, filling an unfilled singleton first.
    pub fn package_id(&mut self, reference: u32) -> Result<u32, Fault> {
        let entry = self
            .refs
            .get(reference)
            .map_err(|e| Fault::BadReference(e.0))?;
        if entry.blob & refs::flags::FILLED == 0 {
            let state = match &entry.target {
                Target::Heap(object) => object.pkg_state_id,
                _ => 0,
            };
            let singleton = self.refs.singleton(state);
            let filled = self
                .refs
                .get(singleton)
                .map_or(true, |e| e.blob & refs::flags::FILLED != 0);
            if !filled {
                self.fill_package(singleton);
            }
        }
        self.refs
            .heap(reference)
            .map(|object| object.pkg_id)
            .map_err(|e| Fault::BadReference(e.0))
    }

    /// `ReferenceTable::FillPackage`: a singleton's data comes from its WState, or else a fresh
    /// instance runs its package initializer.
    pub fn fill_package(&mut self, reference: u32) {
        let Ok(object) = self.refs.heap(reference) else {
            return;
        };
        let did = object.pkg_state_id;
        let state = self.states.wstate(did);
        match state.as_ref().map(|s| &s.references) {
            Some(Ok(packed)) => {
                self.refs.unpack(packed, Some(reference));
            }
            Some(Err(_)) => {}
            None => {
                // `WDataMapper::WStateIDToPackageID`.
                let pkg = if (did ^ (WSTATE_BASE_DID + 0x0100_0000)) & 0xFF00_0000 == 0 {
                    did & 0xFF_FFFF
                } else {
                    u32::MAX
                };
                let size = self.program.package_size(pkg);
                if let Ok(object) = self.refs.heap_mut(reference) {
                    let fresh = ReferenceTable::new_instance(pkg, size);
                    object.pkg_id = fresh.pkg_id;
                    object.slots = fresh.slots;
                    object.tags = fresh.tags;
                }
            }
        }
        if let Ok(entry) = self.refs.get_mut(reference) {
            entry.blob |= refs::flags::FILLED | refs::flags::PACKAGE_DID;
        }
        if let Ok(object) = self.refs.heap_mut(reference) {
            object.pkg_state_id = did;
        }
        if state.is_none() {
            self.call_package_initializer(reference);
        }
    }

    /// `ReferenceTable::CallPackageInitializer`: function 0 of the instance's package, on a
    /// fresh context with an int return slot and `this`.
    pub fn call_package_initializer(&mut self, reference: u32) -> bool {
        let mut ctx = Context::new(0, None);
        if ctx.push(0, tag::INT).is_err() || ctx.push(reference, tag::REF).is_err() {
            return false;
        }
        let Ok(pkg) = self.package_id(reference) else {
            return false;
        };
        let status = self.call_ifun(&mut ctx, pkg, FunctionId::new(pkg, 0, false));
        matches!(status, InterpStatus::Terminated | InterpStatus::NoIFun)
    }

    /// `ReferenceTable::AllocatePackage(pid, ...)`: a new instance of package `pid`, from its
    /// class-default WState when `init` is `UseWState` and one exists, else fresh with its
    /// initializer run (unless `DontInit`).
    pub fn allocate_package(&mut self, pid: u32, init: InitState, is_process: bool) -> Option<u32> {
        let mut run_initializer = true;
        let mut reference = None;
        if init == InitState::UseWState {
            if let Some(state) = self.states.wstate(singleton_did(pid)) {
                let packed = state.references.as_ref().ok()?;
                reference = Some(self.refs.unpack(packed, None)?);
                run_initializer = false;
            }
        }
        let reference = match reference {
            Some(r) => r,
            None => {
                let object = ReferenceTable::new_instance(pid, self.program.package_size(pid));
                self.refs
                    .add(refs::flags::KIND_HEAP, Target::Heap(Box::new(object)))
            }
        };
        self.mark_allocated(reference, is_process);
        if run_initializer && init != InitState::DontInit {
            self.call_package_initializer(reference);
        }
        Some(reference)
    }

    /// `ReferenceTable::AllocatePackage(WState, ...)`: a process instance from a saved state.
    pub fn allocate_from_state(&mut self, state: &WState, is_process: bool) -> Option<u32> {
        let packed = state.references.as_ref().ok()?;
        let reference = self.refs.unpack(packed, None)?;
        if let Ok(object) = self.refs.heap_mut(reference) {
            object.pkg_state_id = state.did;
        }
        self.mark_allocated(reference, is_process);
        Some(reference)
    }

    fn mark_allocated(&mut self, reference: u32, is_process: bool) {
        if let Ok(entry) = self.refs.get_mut(reference) {
            entry.blob = (entry.blob & !refs::flags::RELATIVE)
                | refs::flags::FILLED
                | if is_process { refs::flags::RELATIVE } else { 0 };
        }
    }

    /// `VM::GetSingletonPackageByID`.
    pub fn singleton_package(&mut self, pid: u32) -> u32 {
        self.refs.singleton(singleton_did(pid))
    }

    /// The package id a WState was saved from (`WState::m_pid`).
    pub fn state_package(&self, did: u32) -> Option<u32> {
        self.states.wstate(did).and_then(|state| state.pid)
    }

    /// Writes the slot a reference points at (`GetReferencePtr`/`GetReferenceTagPtr` out
    /// parameters).
    pub fn write_through(
        &mut self,
        ctx: &mut Context,
        reference: u32,
        value: u32,
        tag: u8,
    ) -> bool {
        match self.refs.locate(reference, 0) {
            Ok(Some(slot)) => self.refs.write(ctx, slot, value, tag).is_ok(),
            Ok(None) => {
                self.fill_package(reference);
                match self.refs.locate(reference, 0) {
                    Ok(Some(slot)) => self.refs.write(ctx, slot, value, tag).is_ok(),
                    _ => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Reads the slot a reference points at.
    pub fn read_through(&mut self, ctx: &Context, reference: u32) -> Option<u32> {
        if let Ok(None) = self.refs.locate(reference, 0) {
            self.fill_package(reference);
        }
        let slot = self.refs.locate(reference, 0).ok()??;
        self.refs.read(ctx, slot).ok().map(|(v, _)| v)
    }

    /// The host's DAT services.
    pub fn data(&self) -> &dyn ScriptData {
        self.states.as_ref()
    }

    /// `VM::GetStringLiteral`.
    pub fn string_literal(&self, index: u32) -> Option<u32> {
        self.string_literals.get(index as usize).copied()
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_calls;
