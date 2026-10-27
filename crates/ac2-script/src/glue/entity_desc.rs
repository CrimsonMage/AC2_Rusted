//! `gEntityDesc` (0x22): the entity inquiries the store's sale build calls
//! (`gEntityDesc.cpp`, `EntityDesc::InqEntityDescFromDataID` in
//! `portal/shared/src/entitydesc/entitydesc.cpp.c`).
//!
//! An entity link (`NativeObject::EntityDesc`) holds the entity description's DID. Its
//! properties are read through the data source ([`crate::vm::ScriptData::entity_properties`]),
//! so a cache-only host loads the record on its worker first and the efun runs again.
//! `EntityDesc::InqDataID` and `EntityDesc::InqStringInfo` read the property of the name the
//! script passes, when its value type matches (`BasePropertyValue::GetValue` of the type); a
//! property of another type, or no property, answers 0.

use ac2_db::Ac2PropertyValue;

use super::{word, Call, INT_RESULT};
use crate::data::entity::string_info_property;
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

/// `DB_TYPE_ENTITYDESC`.
const DB_TYPE_ENTITY_DESC: u32 = 0x4000_0049;
/// The MasterProperty value type of a data-file (DID) property (`BasePropertyValue` DID).
const TYPE_DATA_FILE: u32 = 0x4000_0008;
/// The MasterProperty value type of a `StringInfo` property.
const TYPE_STRING_INFO: u32 = 0x4000_000A;
/// The MasterProperty value type of an integer property (`BasePropertyValue` integer).
const TYPE_INTEGER: u32 = 0x4000_0002;
/// The tag a DID carries in a script slot (retail writes `'\x04'`, the VM's `BOOL` tag).
const DID_TAG: u8 = tag::BOOL;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0022_0010, inq_entity_desc_from_data_id);
    efuns.register(0x0022_0014, inq_integer);
    efuns.register(0x0022_0018, inq_data_id);
    efuns.register(0x0022_001D, inq_string_info);
}

/// `gEntityDesc::CallEFun_InqInteger(name, out long)` (`EntityDesc::InqInteger`): the integer
/// property `name`, written to the out reference. A property of another type answers 0 (the
/// `BasePropertyValue` integer getter only takes an integer value).
fn inq_integer(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (name, out) = (word(ctx, 2), word(ctx, 3));
    let value = property(vm, this, name)
        .filter(|p| p.value_type == TYPE_INTEGER)
        .and_then(|p| p.word());
    let found = match value {
        Some(value) => {
            if let Ok(Some(slot)) = vm.refs.locate(out, 0) {
                _ = vm.refs.write(ctx, slot, value, tag::INT);
            }
            true
        }
        None => false,
    };
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// `gEntityDesc::CallEFun_InqEntityDescFromDataID(did, entity)`: `did` at fp-8, the entity
/// (the reference the script created) at fp-0xC, the bool at fp-0x10. The entity links to the
/// record when the DB type is `DB_TYPE_ENTITYDESC` and the record loads.
fn inq_entity_desc_from_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let did = word(ctx, 2);
    let entity = word(ctx, 3);
    let found = vm.data().db_type(did) == Some(DB_TYPE_ENTITY_DESC)
        && vm.data().entity_properties(did).is_some();
    if found {
        if let Some(target) = vm.read_through(ctx, entity) {
            if let Ok(heap) = vm.refs.heap_mut(target) {
                heap.native = Some(NativeObject::EntityDesc(did));
            }
        }
    }
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// `gEntityDesc::CallEFun_InqDataID(name, out did)`: the DID property `name`, written to the
/// out reference (`EntityDesc::InqDataID`).
fn inq_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (name, out) = (word(ctx, 2), word(ctx, 3));
    let found = match property(vm, this, name) {
        Some(p) if p.value_type == TYPE_DATA_FILE => match p.word() {
            Some(did) => {
                if let Ok(Some(slot)) = vm.refs.locate(out, 0) {
                    _ = vm.refs.write(ctx, slot, did, DID_TAG);
                }
                true
            }
            None => false,
        },
        _ => false,
    };
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// `gEntityDesc::CallEFun_InqStringInfo(name, out info)`: the `StringInfo` property `name`,
/// written into the out reference's object (`EntityDesc::InqStringInfo`).
fn inq_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (name, out) = (word(ctx, 2), word(ctx, 3));
    let info = property(vm, this, name)
        .filter(|p| p.value_type == TYPE_STRING_INFO)
        .and_then(|p| string_info_property(&p.bytes));
    let found = match info {
        Some(info) => {
            if let Some(target) = vm.read_through(ctx, out) {
                if let Ok(heap) = vm.refs.heap_mut(target) {
                    heap.native = Some(NativeObject::StringInfo(info));
                }
            }
            true
        }
        None => false,
    };
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// The property `name` of the entity link `this`. `None` when `this` is not an entity link,
/// its record is not read yet, or the entity has no such property.
fn property(vm: &Vm, this: u32, name: u32) -> Option<Ac2PropertyValue> {
    let Some(NativeObject::EntityDesc(did)) = vm.native(this) else {
        return None;
    };
    let properties = vm.data().entity_properties(*did);
    let properties = properties?;
    properties.iter().find(|p| p.name == name).cloned()
}
