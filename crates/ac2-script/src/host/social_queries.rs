//! The fellowship page's question to the player's script that hands back a native the script
//! fills: `gmCWInterface::GetFellowXPHash` (`ggmCWInterface::CallIFun_GetFellowXPHash`: an
//! empty `LAHash` (native 0x32) as the out parameter, `CPlayer` 0x2C ->
//! `FellowshipSystem` 0x41 adds each fellow's share, `LAHashToXPHash` reads it back as
//! `PHashTable<InstanceID, float>`).

use ac2_net::protocol::{ntype, HashKey, NativeObject, ScriptHash};

use super::queries::ScriptQueries;
use super::{IFunArg, IFunReturn, OutValue};

/// `GetFellowXPHash`'s IFun.
const GET_FELLOW_XP_HASH: u32 = 0x00B4_00A8;

/// The fellowship page's queries, on any [`ScriptQueries`] (the client implements the port).
pub trait SocialQueries: ScriptQueries {
    /// `gmCWInterface::GetFellowXPHash`: each fellow's share of the experience the fellowship
    /// earns (`FellowshipHelper::UpdateXP`), in the hash's order; empty for a social fellowship
    /// (`FellowshipSystem` 0x41 fills nothing then) or none.
    fn fellow_xp_hash(&mut self) -> Result<Vec<(u64, f32)>, String> {
        let empty = NativeObject::Hash(ScriptHash {
            native_type: ntype::LA_HASH,
            bucket_index: 0,
            keys: Vec::new(),
            values: Vec::new(),
        });
        let result = self.query(
            GET_FELLOW_XP_HASH,
            &[IFunArg::Out(OutValue::Native(empty))],
            IFunReturn::Word,
        )?;
        match result.out_native(0) {
            Some(NativeObject::Hash(hash)) => Ok(hash
                .keys
                .iter()
                .zip(&hash.values)
                .filter_map(|(key, value)| match key {
                    HashKey::Value(id) => Some((*id, f32::from_bits(*value as u32))),
                    _ => None,
                })
                .collect()),
            other => Err(format!("GetFellowXPHash left {other:?}")),
        }
    }
}

impl<P: ScriptQueries + ?Sized> SocialQueries for P {}
