//! `gSkillPanelManager`: the skill tree panels' data from the `SkillPanelManager` singleton
//! script (`gSkillPanelManager::CallIFun_InqName` / `InqIcon` / `InqNodeHash`, which
//! `gmSkillTreeUI` calls), as plain data for the UI.

use ac2_net::protocol::{self as net, SkillUiNode};

use super::chargen::{call_system, Push};
use super::{HostError, ScriptHost};
use crate::native::{ntype, NativeObject, PlainKeys, RefKey, ScriptHash, DEFAULT_BUCKETS};
use crate::vm::{tag, InterpStatus};
use crate::wire::string_info_to_net;

const SYSTEM: &str = "SkillPanelManager";
const INQ_NAME: u32 = 0x00AD_0002;
const INQ_ICON: u32 = 0x00AD_0003;
const INQ_NODE_HASH: u32 = 0x00AD_0005;

/// One skill panel: its title, icon and nodes (skill enum -> node), in the order of the
/// `PHashTable<unsigned long, SkillUINode>` `ARHashToSkillUINodeHash` fills.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillPanelData {
    pub name: net::StringInfo,
    pub icon: u32,
    pub nodes: Vec<(u32, SkillUiNode)>,
}

impl ScriptHost {
    /// The skill panel `panel` (`gmSkillTreeUI::LoadPanel`'s enum, the examination request's
    /// `_data_enum`), or `None` when the library has no `SkillPanelManager`. Like
    /// `LoadPanel`, the scripts' `retval`s are not consulted. Each call is retail's: a
    /// context on the singleton, the out object, a 0 return slot, its stack address, the
    /// panel, the instance.
    pub fn skill_panel(&mut self, panel: u32) -> Result<Option<SkillPanelData>, HostError> {
        let vm = &mut self.vm;
        if vm.program.package_id(SYSTEM).is_none() {
            return Ok(None);
        }
        let args = [Push::Int(panel)];
        let call = |vm: &mut crate::vm::Vm, f: u32, out: (u32, u8)| {
            let c = call_system(vm, SYSTEM, f, Some(out), &args).ok_or(HostError::BadArguments)?;
            if c.status != InterpStatus::Terminated {
                return Err(HostError::Stopped(c.status, vm.last_fault.clone()));
            }
            Ok(c)
        };
        let hash = vm.refs.create_native(
            ntype::AR_HASH,
            NativeObject::Hash(ScriptHash::with_buckets(DEFAULT_BUCKETS)),
        );
        let nodes = call(vm, INQ_NODE_HASH, (hash, tag::REF))?;
        let name = vm.refs.create_native(
            ntype::STRING_INFO,
            NativeObject::new_empty(ntype::STRING_INFO),
        );
        let name = call(vm, INQ_NAME, (name, tag::REF))?.out;
        let icon = call(vm, INQ_ICON, (0, tag::BOOL))?.out;
        let name = match vm.native(name) {
            Some(NativeObject::StringInfo(info)) => string_info_to_net(info),
            _ => net::StringInfo::default(),
        };
        // `ARHashToSkillUINodeHash`: the script's hash in its order into a fresh C++ table.
        let mut order = ScriptHash::with_buckets(DEFAULT_BUCKETS);
        let mut found = Vec::new();
        if let Some(NativeObject::Hash(h)) = vm.native(nodes.out) {
            for n in h.nodes() {
                let RefKey::Value(key) = n.key else { continue };
                if order.add(RefKey::Value(key), found.len() as u64, &PlainKeys) {
                    let node = match vm.native(n.value as u32) {
                        Some(NativeObject::Wire(w)) => match &**w {
                            net::NativeObject::SkillUiNode(s) => (**s).clone(),
                            _ => crate::glue::empty_skill_node(),
                        },
                        _ => crate::glue::empty_skill_node(),
                    };
                    found.push((key as u32, node));
                }
            }
        }
        let nodes = order
            .nodes()
            .map(|n| found[n.value as usize].clone())
            .collect();
        Ok(Some(SkillPanelData { name, icon, nodes }))
    }
}
