//! `FxScript` (DB type `0x40000056`, DIDs `0x57xxxxxx`): a timeline of effect and impulse
//! triggers over one period, which `FXScriptTracker` plays on a physics object
//! (`CPhysicsObj::DoFXScript`). The record is `FxScript::StreamPack`: the embedded DID
//! (`DBObj::UnPack_Type`), the effect triggers (`mEffectsTriggers`), the impulse triggers
//! (`mImpulseTriggers`) and the period (`mPeriod`). Editor-only parsing (`FromFileNode`,
//! `ToFileNode`, the parse helpers) is not ported.

use ac2_core::QualifiedDataId;
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::dbanimator::{decode_effects, decode_impulses, Ac2AnimTrigger, Ac2ImpulseTrigger};

/// `FxScript::GetDBOType` (`0x40000056`); the DIDs of the type are `0x57000000..=0x57FFFFFE`.
pub const FX_SCRIPT_PREFIX: u32 = 0x5700_0000;

/// A decoded `FxScript`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxScript {
    /// `mEffectsTriggers`: the effects the timeline starts (`Trigger`s, time in periods).
    pub effects: Vec<Ac2AnimTrigger>,
    /// `mImpulseTriggers`.
    pub impulses: Vec<Ac2ImpulseTrigger>,
    /// `mPeriod`: seconds for the timeline to run once (`FXScriptTracker::Update` divides the
    /// quantum by it).
    pub period: f32,
}

impl ResourcePackedDecode for Ac2FxScript {
    const TYPE_NAME: &'static str = "AC2 FxScript";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let effects = decode_effects(&mut cursor).map_err(malformed)?;
        let impulses = decode_impulses(&mut cursor).map_err(malformed)?;
        let period = cursor.read_f32()?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after FxScript"));
        }
        Ok(Self {
            effects,
            impulses,
            period,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.effects.len() * std::mem::size_of::<Ac2AnimTrigger>()
            + self.impulses.len() * std::mem::size_of::<Ac2ImpulseTrigger>()
    }
}

/// Loads the `FxScript` `did` from the portal DAT (`FXTrack::FillCache`'s `DBObj::Get` with
/// the `0x40000056` type). `None` when the DAT has no such record.
pub fn load_ac2_fx_script(
    store: &ResourceStore,
    did: u32,
) -> ResourceResult<Option<CachedResource<Ac2FxScript>>> {
    store.load_cached_packed::<Ac2FxScript>(TypedResourceRef::new(
        DatFileKind::Portal,
        ac2_core::DataId::new(did),
        0,
    ))
}

#[cfg(test)]
#[path = "fxscript_tests.rs"]
mod tests;
