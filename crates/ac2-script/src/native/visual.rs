//! `VisualDesc` (native 0x71) and `AppliedAppearanceKey` (native 0x10): the appearance a script
//! builds for a character (`gCharacterGenSystem::CallIFun_InqAppliedAppearanceVDesc`) and one
//! applied key of it (`AppliedAppearanceKey`: the key and its scalar).

/// `AppliedAppearanceKey` (`m_key`, `m_scalar_mod`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AppliedAppearanceKey {
    pub key: u32,
    pub scalar: f32,
}
