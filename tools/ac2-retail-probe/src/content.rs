//! Recover client-retained quest definitions. No inferred objectives, rewards,
//! spawn coordinates or loot weights are emitted. Keep unresolved references.
use ac2_core::DataId;
use ac2_db::{ResourceStore, StoreStringTables};
use ac2_script::{bytestream::ByteStream, native::{NativeObject, RefKey},
    reference::{HeapObject, PackedEntry, PackedReference}};
use serde_json::{json, Value};

fn heap(refs: &PackedReference, id: u32) -> Option<&HeapObject> {
    match &refs.entries[*refs.ids.iter().position(|v| *v == id).as_ref()?] {
        PackedEntry::Heap { object, .. } => Some(object), _ => None,
    }
}
fn text(refs: &PackedReference, id: u32, tables: &StoreStringTables) -> Option<Value> {
    let Some(NativeObject::StringInfo(info)) = &heap(refs,id)?.native else { return None };
    let rendered = ac2_db::inq_string(tables,&ac2_script::data::render_info(info),false);
    Some(json!({"table":info.table_id,"key":info.string_id,
        "text":rendered.as_ref().ok().map(|s|String::from_utf16_lossy(s)),
        "error":rendered.err().map(|e|format!("{:?}",e.code)),
        "variables":format!("{:?}",info.variables)}))
}

pub fn recover(store: &ResourceStore, stream: &ByteStream) -> Value {
    let tables = StoreStringTables::new(store.clone(),0x58000000);
    let boundaries:std::collections::BTreeSet<u32>=stream.vtables.vtables.iter().flatten().copied()
        .filter(|v|*v!=u32::MAX).chain([stream.opcodes.len() as u32]).collect();
    let mut quests=Vec::new(); let mut rejected=Vec::new(); let mut scripts=Vec::new(); let mut effects=Vec::new();
    for export in &stream.exports {
        let pid=export.args.pkg_id;
        let raw_name=stream.vtables.package_names.iter().find(|(_,id)|*id==pid)
            .map(|(name,_)|name).unwrap_or(&export.args.name);
        let name=String::from_utf8_lossy(raw_name);
        if name.contains("Quest") || name.contains("Loot") || name.contains("Generator") || name.contains("Spawner") || name.contains("Sign") {
            let parent=stream.vtables.vtables.get(export.args.parent_index as usize);
            let own=stream.vtables.package_id_map.get(pid as usize).and_then(|index|stream.vtables.vtables.get(*index as usize));
            let bodies:Vec<_>=own.into_iter().flatten().enumerate().filter(|(slot,offset)|
                **offset!=u32::MAX && parent.and_then(|p|p.get(*slot))!=Some(*offset))
                .map(|(slot,&offset)|{
                    let end=boundaries.range((std::ops::Bound::Excluded(offset),std::ops::Bound::Unbounded)).next().copied().unwrap_or(offset);
                    json!({"slot":slot,"offset":offset,"kind":crate::bytecode::body_kind(&stream.opcodes,offset,end)})
                }).collect();
            scripts.push(json!({"name":name,"pid":pid,"parent_index":export.args.parent_index,
                "own_bodies":bodies,
                "virtual_functions":stream.vtables.func_mapper.get(pid as usize).map(|v|v.iter().map(|f|f.0).collect::<Vec<_>>()),
                "body_offsets":stream.vtables.package_id_map.get(pid as usize).and_then(|index|stream.vtables.vtables.get(*index as usize)),
                "functions":export.functions.iter().map(|f|json!({"name":String::from_utf8_lossy(&f.name),"id":f.fid,"offset":f.offset,"size":f.size})).collect::<Vec<_>>()}));
        }
        if name.starts_with("Eff_") && name.contains("Quest") {
            if let Ok(Some(state))=ac2_script::wstate::load_wstate(store,DataId::new(0x70000000|pid)) {
                if let Ok(refs)=&state.references {
                    if let Some(root)=refs.ids.first().and_then(|id|heap(refs,*id)) {
                        if root.pkg_id==pid && root.slots.len()>=37 && root.tags[35..37]==[2,4] {
                            effects.push(json!({"package":name,"state":0x70000000|pid,"quest":root.slots[36],
                                "quest_status":root.slots[35],"extra_slots":root.slots[37..],"extra_tags":root.tags[37..]}));
                        }
                    }
                }
            }
        }
        if !name.starts_with("Quest") {continue;}
        let did=0x70000000|pid;
        let state=match ac2_script::wstate::load_wstate(store,DataId::new(did)) {
            Ok(Some(s))=>s, Ok(None)=>{rejected.push(json!({"name":name,"state":did,"reason":"missing state"}));continue},
            Err(e)=>{rejected.push(json!({"name":name,"state":did,"reason":format!("{e:?}")}));continue}
        };
        let Ok(refs)=&state.references else {rejected.push(json!({"name":name,"state":did,"reason":"unreadable state"}));continue};
        let Some(root)=refs.ids.first().and_then(|id|heap(refs,*id)) else {continue};
        // Quest template: id, name, description, icon, phase map, summary,
        // show flag, extra map, package. PhaseInfo: related quest list, two
        // flags, journal. Keep links as links; they do not prove eligibility.
        if root.pkg_id!=pid || root.slots.len()<9 || root.tags.get(..9)!=Some(&[4,1,1,4,1,1,2,1,1]) || ![0x40,0x80].contains(&(root.slots[0]>>24)) {
            rejected.push(json!({"name":name,"state":did,"reason":"not a quest definition layout"}));continue;
        }
        let s=&root.slots;
        let Some(NativeObject::Hash(map))=heap(refs,s[4]).and_then(|h|h.native.as_ref()) else {continue};
        let mut phases=Vec::new(); let mut valid=true;
        for node in map.buckets.iter().flatten() {
            let RefKey::Value(phase)=node.key else {valid=false;break};
            let Some(p)=heap(refs,node.value as u32) else {valid=false;break};
            if p.pkg_id!=422 || p.slots.len()!=4 || p.tags!=[1,2,2,1] {valid=false;break;}
            let related=if p.slots[0]==u32::MAX {Some(Vec::new())} else {
                match heap(refs,p.slots[0]).and_then(|h|h.native.as_ref()) {
                    Some(NativeObject::List(list)) if !list.wide=>Some(list.items.clone()), _=>None,
                }
            };
            phases.push(json!({"phase":phase,"related_quest_reference":p.slots[0],"related_quests":related,
                "flags":[p.slots[1],p.slots[2]],"journal":text(refs,p.slots[3],&tables)}));
        }
        if !valid {rejected.push(json!({"name":name,"state":did,"reason":"unsupported phase layout"}));continue;}
        phases.sort_by_key(|v|v["phase"].as_u64());
        quests.push(json!({"package":name,"pid":pid,"state":did,"id":s[0],"icon":s[3],
            "name":text(refs,s[1],&tables),"description":text(refs,s[2],&tables),
            "summary":text(refs,s[5],&tables),"phases":phases}));
    }
    quests.sort_by_key(|v|v["id"].as_u64());
    json!({"schema":1,"source":"Installed Legions WLib and WState; admin gmQuestInfo and native StringInfo decoding",
        "import_symbol_count":stream.imports.iter().map(|i|i.symbols.len()).sum::<usize>(),
        "debug_section_bytes":stream.debug_sections.iter().map(|(_,v)|v.len()).sum::<usize>(),
        "quests":quests,"rejected":rejected,"scripts":scripts,"quest_effects":effects,
        "limitations":"Phase text is not executable quest logic. Do not infer objectives, rewards, NPC bindings, loot weights or spawn coordinates from names."})
}
