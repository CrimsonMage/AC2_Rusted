//! Read-only listing of retained function bodies, including stripped exports.
use ac2_script::{bytestream::ByteStream, vm::{Program, FunctionId}};
use serde_json::{json,Value};
use std::collections::BTreeSet;

pub fn body_kind(code:&[u8], offset:u32, end:u32)->&'static str {
    let Some(body)=code.get(offset as usize..end as usize) else{return "invalid"};
    let words:Vec<_>=body.chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap())).collect();
    if words.is_empty(){"empty"}
    else if words.iter().all(|v|[0x4b,0x4c,0x4a].contains(&(v&255))){"trap"}
    else if words.iter().all(|v|[0x09,0x0a,0x4a].contains(&(v&255))){"empty_stub"}
    else{"retained"}
}

pub fn listing(stream:&ByteStream, names:&[String])->Value {
    let p=Program::from_stream(stream);
    let ends:BTreeSet<u32>=p.vtables.iter().flatten().copied()
        .filter(|v|*v!=u32::MAX).chain([stream.opcodes.len() as u32]).collect();
    let mut packages=Vec::new();
    for (name,pid) in &p.package_names {
        let name=String::from_utf8_lossy(name);
        if !names.iter().any(|n|n==name.as_ref()){continue;}
        let index=p.package_vtable[*pid as usize] as usize;
        let parent=p.exports[index].1 as usize;
        let inherited=p.vtables.get(parent);
        let mut functions=Vec::new();let mut seen=BTreeSet::new();
        for (slot,&offset) in p.vtables[index].iter().enumerate() {
            if offset==u32::MAX || inherited.and_then(|v|v.get(slot))==Some(&offset) || !seen.insert(offset){continue;}
            let end=ends.range((std::ops::Bound::Excluded(offset),std::ops::Bound::Unbounded)).next().copied().unwrap_or(offset);
            let mut instructions=Vec::new();let mut pc=offset as usize/4;
            while pc<end as usize/4 && pc<p.code.len() {
                let word=p.code[pc];let next=p.walk(pc).min(p.code.len());
                let operands=&p.code[pc+1..next];let op=word&0xff;
                let target=if [0x0b,0x4f,0x51,0x53].contains(&op) {
                    operands.first().map(|fid|{let fid=FunctionId(*fid);
                        json!({"package":p.package_names.iter().find(|(_,id)|*id==fid.pkg_num()).map(|(n,_)|String::from_utf8_lossy(n)),
                            "function":fid.func_num(),"absolute":fid.is_absolute(),"offset":p.function_offset(*pid,fid)})})
                }else{None};
                let literal=if op==8{operands.first().and_then(|id|p.string_literals.get(*id as usize)).map(|s|String::from_utf16_lossy(s))}else{None};
                instructions.push(json!({"offset":pc*4,"op":op,"tag":(word>>8)&255,"immediate":(word as i32)>>16,"operands":operands,"target":target,"literal":literal}));
                pc=next;
            }
            functions.push(json!({"slot":slot,"offset":offset,"kind":body_kind(&stream.opcodes,offset,end),"instructions":instructions}));
        }
        packages.push(json!({"name":name,"pid":pid,"functions":functions}));
    }
    json!(packages)
}
