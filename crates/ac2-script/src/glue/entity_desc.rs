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
const TYPE_BOOL: u32 = 0x4000_0001;
const TYPE_ENUM: u32 = 0x4000_0007;
/// The tag a DID carries in a script slot (retail writes `'\x04'`, the VM's `BOOL` tag).
const DID_TAG: u8 = tag::BOOL;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0022_0010, inq_entity_desc_from_data_id);
    efuns.register(0x0022_0013, |vm, c| inq_word(vm, c, TYPE_BOOL));
    efuns.register(0x0022_0014, inq_integer);
    efuns.register(0x0022_0017, |vm, c| inq_word(vm, c, TYPE_ENUM));
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

/// `gEntityDesc::CallEFun_InqBool(name, out)` and `InqEnum(name, out)` (`EntityDesc::InqBool`,
/// `InqEnum`): the property `name` when it has the type `want`, its word written to the out
/// reference (tagged 4); the flag is the result.
fn inq_word(vm: &mut Vm, ctx: &mut Context, want: u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (name, out) = (word(ctx, 2), word(ctx, 3));
    let value = property(vm, this, name)
        .filter(|p| p.value_type == want)
        .and_then(|p| p.word());
    if let Some(value) = value {
        if let Ok(Some(slot)) = vm.refs.locate(out, 0) {
            _ = vm.refs.write(ctx, slot, value, tag::BOOL);
        }
    }
    Call::new(ctx).ret(4, u32::from(value.is_some()), INT_RESULT);
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_db::Ac2PropertyValue;

    use super::*;
    use crate::host::ScriptHost;
    use crate::vm::{refs::flags, refs::Target, Program, ScriptData};

    const ENTITY: u32 = 0x3000_0001;
    const NAME_INT: u32 = 0x1000_0001;
    const NAME_LONG: u32 = 0x1000_0002;
    const NAME_BOOL: u32 = 0x1000_0003;
    const NAME_ENUM: u32 = 0x1000_0004;

    /// One entity whose properties are the given values.
    struct Entity(Arc<Vec<Ac2PropertyValue>>);

    impl ScriptData for Entity {
        fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
            None
        }

        fn db_type(&self, did: u32) -> Option<u32> {
            (did == ENTITY).then_some(DB_TYPE_ENTITY_DESC)
        }

        fn entity_properties(&self, did: u32) -> Option<crate::vm::EntityProperties> {
            (did == ENTITY).then(|| self.0.clone())
        }
    }

    fn prop(name: u32, value_type: u32, bytes: &[u8]) -> Ac2PropertyValue {
        Ac2PropertyValue {
            name,
            value_type,
            bytes: bytes.into(),
        }
    }

    /// Calls `InqInteger(name, out)` on the entity link; the out is a script local at slot 0.
    fn inq_integer(name: u32, local: u32) -> (u32, u32) {
        inq("gEntityDesc::CallEFun_InqInteger", name, local)
    }

    fn inq(efun: &str, name: u32, local: u32) -> (u32, u32) {
        let props = vec![
            prop(NAME_BOOL, TYPE_BOOL, &1u32.to_le_bytes()),
            prop(NAME_ENUM, TYPE_ENUM, &42u32.to_le_bytes()),
            prop(NAME_INT, TYPE_INTEGER, &7i32.to_le_bytes()),
            // A LongInteger (0x4000000D) holds eight bytes; `EntityDesc::InqInteger` answers 0 for it.
            prop(NAME_LONG, 0x4000_000D, &9i64.to_le_bytes()),
        ];
        let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
        let mut host = ScriptHost::new(Arc::new(program), Box::new(Entity(Arc::new(props))));
        let vm = host.vm_mut();
        let link = vm
            .refs
            .create_native(0x22, crate::native::NativeObject::EntityDesc(ENTITY));
        let addr = vm.refs.add(
            (0u32 << flags::OFFSET_SHIFT) | flags::KIND_VALUE,
            Target::Ref(u32::MAX),
        );
        let id = crate::glue::efun_table()
            .find(|(_, n)| *n == efun)
            .map(|(id, _)| id)
            .unwrap();
        let mut ctx = crate::vm::Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(local, crate::vm::tag::INT).unwrap();
        ctx.push(0, crate::vm::tag::INT).unwrap();
        ctx.push(addr, crate::vm::tag::REF).unwrap();
        ctx.push(name, crate::vm::tag::INT).unwrap();
        ctx.push(link, crate::vm::tag::REF).unwrap();
        let fp = ctx.sp;
        assert_eq!(vm.run_efun(id, &mut ctx), crate::vm::InterpStatus::Success);
        (ctx.data[fp - 4], ctx.data[0])
    }

    #[test]
    fn inq_integer_reads_integer_properties_and_answers_zero_for_long_integers() {
        let (found, value) = inq_integer(NAME_INT, 0);
        assert_eq!((found, value), (1, 7));
        let (found, value) = inq_integer(NAME_LONG, 0xDEAD);
        assert_eq!(
            (found, value),
            (0, 0xDEAD),
            "a LongInteger is not read and nothing is written"
        );
    }

    #[test]
    fn inq_bool_and_inq_enum_read_only_their_own_type() {
        let bool_name = "gEntityDesc::CallEFun_InqBool";
        let enum_name = "gEntityDesc::CallEFun_InqEnum";
        assert_eq!(inq(bool_name, NAME_BOOL, 0), (1, 1));
        assert_eq!(inq(enum_name, NAME_ENUM, 0), (1, 42));
        assert_eq!(inq(bool_name, NAME_ENUM, 9), (0, 9));
        assert_eq!(inq(enum_name, NAME_INT, 9), (0, 9));
    }
}
