//! Native type ids per DAT release. The script VM, the packers and the glue use the ids of the
//! 7160 and 7230 clients (`CNativeFactory::GetNativeTypeName` Legions 0x0049F540,
//! `gmCNativeFactory::GetNativeTypeName` Legions 0x00467150); the 0930 client renumbered the
//! highest ones into the `0x1Axx` range, so the 0930 DATs' WStates and wlib bytecode name them
//! by those ids (`CNativeFactory::GetNativeTypeName` client0930 0x004A98E0,
//! `gmCNativeFactory::GetNativeTypeName` client0930 0x00412F10). Every other id is equal in the
//! two tables (107 and 28 names compared). The `CEFunDispatcher::GetEFunAddress` classes moved
//! with them: for the native types the glue functions of each class compare instruction for
//! instruction equal between the Legions and 0930 executables (`LAHashSet` 15 of 15 efuns,
//! the four multi-maps 12 of 12 each, `CaseInsensitiveWPString` 33 of 33, `EntityLinkDesc` 6 of
//! 6, the iterators 5 of 5). The efun-only classes 0x25BB, 0x3839, 0x23AC and 0x200A are paired
//! by function count and the code that is equal or similar in order (0x200A: 2 of 2 equal; the
//! others align on their unchanged functions). A process reads and writes the ids of the mounted
//! release and keeps the 7160/7230 ids in between.

use crate::{data_release, DataRelease};

/// `(7160/7230 id, 0930 id, name)`.
const RENUMBERED: [(u32, u32, &str); 19] = [
    (0x2E0B, 0x1AC8, "LAHashSet"),
    (0x2E0C, 0x1AC9, "LAHashSetIter"),
    (0x353F, 0x1ACF, "CaseInsensitiveWPString"),
    (0x3691, 0x1AC3, "AAMultiMap"),
    (0x3692, 0x1AC4, "AAMultiMapIter"),
    (0x3693, 0x1AC5, "ALMultiMap"),
    (0x3694, 0x1AC6, "ALMultiMapIter"),
    (0x3695, 0x1ACA, "LAMultiMap"),
    (0x3696, 0x1ACB, "LAMultiMapIter"),
    (0x3697, 0x1ACC, "LLMultiMap"),
    (0x3698, 0x1ACD, "LLMultiMapIter"),
    (0x37A0, 0x1AC7, "EntityLinkDesc"),
    (0x33A5, 0x1AD4, "CraftSkillAdaptor"),
    (0x380A, 0x1AD6, "UIShop"),
    (0x380B, 0x1AD7, "UIShopSale"),
    // Efun-only classes (no native type): the `CEFunDispatcher` / `gmCEFunDispatcher` class
    // numbers moved the same way. Matched by function count and identical glue code.
    (0x23AC, 0x1AD2, "efuns of 0x23AC"),
    (0x25BB, 0x1ACE, "string utils efuns"),
    (0x3839, 0x1AD3, "shop efuns"),
    (0x200A, 0x1AD5, "TurbineGunUtils"),
];

/// The 7160/7230 id of a native type named by the mounted release's id.
pub fn native_type_from_release(id: u32) -> u32 {
    if data_release() != DataRelease::Release0930 {
        return id;
    }
    RENUMBERED
        .iter()
        .find(|(_, release, _)| *release == id)
        .map_or(id, |(canonical, _, _)| *canonical)
}

/// The mounted release's id of a native type with the 7160/7230 id.
pub fn native_type_to_release(id: u32) -> u32 {
    if data_release() != DataRelease::Release0930 {
        return id;
    }
    RENUMBERED
        .iter()
        .find(|(canonical, _, _)| *canonical == id)
        .map_or(id, |(_, release, _)| *release)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_renumbered_ids_do_not_collide() {
        for (i, a) in RENUMBERED.iter().enumerate() {
            for b in &RENUMBERED[..i] {
                assert!(a.0 != b.0 && a.1 != b.1, "{}", a.2);
            }
            // None of the 0930 ids is another type's 7160 id.
            assert!(RENUMBERED.iter().all(|b| b.0 != a.1), "{}", a.2);
        }
    }
}
