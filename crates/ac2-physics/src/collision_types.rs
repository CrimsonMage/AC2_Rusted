//! Which objects a mover is blocked by: `EtherealPhysicsType` (a 64-bit physics-type mask),
//! `EtherealWeenieType` (a weenie mask plus solid and ethereal id lists), and the
//! `CollisionParameters` a script sets on a body (`CPhysicsObj::SetCollisionState`).
//! `CPhysicsObj::IsBlocking` decides, for one object, whether it stops the mover.

/// `CPhysicsObj::IsBlocking(wtype, ptype)` for the object `id` whose blocking mask is
/// `object_mask`: when the masks share a type the object blocks only if the id is in the solid
/// list; otherwise it blocks unless the id is in the ethereal list.
pub fn is_blocking(
    weenie: &WeenieTypes,
    ptype: PhysicsTypeMask,
    id: InstanceId,
    object_mask: PhysicsTypeMask,
) -> bool {
    if object_mask.intersects(ptype) {
        weenie.in_solid_list(id)
    } else {
        !weenie.in_ethereal_list(id)
    }
}

/// `MasterProperty::GetPropertyDesc(name, &bit)` for a bool property: the bit
/// `EtherealPhysicsType::SetProperty` sets. `None` for a name that has no such desc.
pub fn ethereal_property_bit(name: u32) -> Option<u64> {
    ac2_db::ac2_property_desc(name)
        .filter(|desc| desc.value_type == 0x4000_0001)
        .map(|desc| u64::from(desc.data))
}

/// The water-layer property `EtherealPhysicsType::SetEtherealToWaterLayer` names.
pub const WATER_LAYER_PROPERTY: u32 = 0x4000_001C;

/// An `InstanceID` (the 64-bit id a weenie and its lists use).
pub type InstanceId = u64;

/// `EtherealPhysicsType`: one bit per physics type, 64 types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicsTypeMask(u64);

impl PhysicsTypeMask {
    /// The raw mask.
    pub fn bits(self) -> u64 {
        self.0
    }

    /// A mask of the given bits (an object's blocking mask, read from its properties).
    pub fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// `EtherealPhysicsType::SetBit`: sets or clears bit `bit` when it is below 64; higher
    /// values change nothing.
    pub fn set_bit(&mut self, bit: u64, is_set: bool) {
        if bit < 64 {
            if is_set {
                self.0 |= 1 << bit;
            } else {
                self.0 &= !(1 << bit);
            }
        }
    }

    /// `EtherealPhysicsType::SetProperty`: the property `name` must exist in the master property
    /// table (`exists`, `MasterProperty::GetPropertyDesc`), and then its bit is set or cleared.
    /// Returns whether the property exists.
    pub fn set_property(&mut self, name: u64, is_set: bool, exists: impl Fn(u64) -> bool) -> bool {
        if !exists(name) {
            return false;
        }
        self.set_bit(name, is_set);
        true
    }

    /// `gCollisionParameters::CallEFun_Set*LowDWord`: the low word is replaced and the high word
    /// gains the sign extension of the argument (an `int` widened to 64 bits, then OR-ed in).
    pub fn set_low_dword(&mut self, low: u32) {
        let extension = if low >> 31 != 0 { u32::MAX } else { 0 };
        let high = ((self.0 >> 32) as u32) | extension;
        self.0 = u64::from(high) << 32 | u64::from(low);
    }

    /// `gCollisionParameters::CallEFun_Set*HighDWord`: the high word is replaced.
    pub fn set_high_dword(&mut self, high: u32) {
        self.0 = u64::from(high) << 32 | (self.0 & 0xFFFF_FFFF);
    }

    /// `EtherealPhysicsType::UpdateEthereality`: the lowest type `mask` has and this lacks is
    /// added (one type, not all). Returns 1 as retail does.
    pub fn update_ethereality(&mut self, mask: Self) -> bool {
        let missing = mask.0 & !self.0;
        if missing != 0 {
            self.0 |= missing & missing.wrapping_neg();
        }
        true
    }

    /// `EtherealPhysicsType::SetEtherealToWaterLayer`: an ethereal layer first sets the water
    /// layer's bit, and the bit is then cleared in either case (`bit`, the water property's, as
    /// `ethereal_property_bit` gives it).
    pub fn set_ethereal_to_water_layer(
        &mut self,
        ethereal: bool,
        bit: u64,
        exists: impl Fn(u64) -> bool,
    ) {
        if ethereal {
            self.set_property(bit, true, &exists);
        }
        self.set_property(bit, false, &exists);
    }

    /// `EtherealPhysicsType::SetCameraEthereal`: the mask empties, then the camera property's bit
    /// is set (`exists` is `MasterProperty::GetPropertyDesc`, `camera` its bit).
    pub fn set_camera_ethereal(&mut self, camera: u64, exists: impl Fn(u64) -> bool) {
        self.0 = 0;
        self.set_property(camera, true, exists);
    }

    /// Whether the two masks share a bit (`IsBlocking`'s test).
    pub fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// `EtherealWeenieType`: the weenie mask and the per-object solid and ethereal lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeenieTypes {
    mask: u64,
    solid: Vec<InstanceId>,
    ethereal: Vec<InstanceId>,
}

impl Default for WeenieTypes {
    /// `EtherealWeenieType::EtherealWeenieType` (mask 0, empty lists), then `SetToSolid`, as
    /// `CollisionParameters::CollisionParameters` leaves its weenie sets.
    fn default() -> Self {
        let mut types = Self {
            mask: 0,
            solid: Vec::new(),
            ethereal: Vec::new(),
        };
        types.set_to_solid();
        types
    }
}

impl WeenieTypes {
    /// `EtherealWeenieType::EtherealWeenieType`: mask 0 and empty lists (the constructor
    /// `LOSParameters` runs; `Default` goes on to `SetToSolid`, as `CollisionParameters` does).
    pub fn empty() -> Self {
        Self {
            mask: 0,
            solid: Vec::new(),
            ethereal: Vec::new(),
        }
    }

    /// `EtherealWeenieType::m_wtype_mask`.
    pub fn mask(&self) -> u64 {
        self.mask
    }

    /// `EtherealWeenieType::SetToEthereal`: the mask empties.
    pub fn set_to_ethereal(&mut self) {
        self.mask = 0;
    }

    /// `EtherealWeenieType::NeedLOSCheck`: a line of sight must be checked against this set
    /// unless its mask is empty and it names no solid object.
    pub fn need_los_check(&self) -> bool {
        self.mask != 0 || !self.solid.is_empty()
    }

    /// `EtherealWeenieType::SetToSolid`: the mask fills.
    pub fn set_to_solid(&mut self) {
        self.mask = u64::MAX;
    }

    /// `EtherealWeenieType::InSolidList`.
    pub fn in_solid_list(&self, id: InstanceId) -> bool {
        self.solid.contains(&id)
    }

    /// `EtherealWeenieType::InEtherealList`.
    pub fn in_ethereal_list(&self, id: InstanceId) -> bool {
        self.ethereal.contains(&id)
    }

    /// `EtherealWeenieType::AddToEtherealList`: each id once.
    pub fn add_to_ethereal_list(&mut self, id: InstanceId) -> bool {
        if !self.in_ethereal_list(id) {
            self.ethereal.push(id);
        }
        true
    }

    /// `EtherealWeenieType::Update(source, this)`: a source with empty lists copies over whole;
    /// otherwise its mask replaces this one's and its solid and ethereal ids join this one's
    /// lists, each id once.
    pub fn update_from(&mut self, source: &WeenieTypes) {
        if source.ethereal.is_empty() && source.solid.is_empty() {
            *self = source.clone();
            return;
        }
        self.mask = source.mask;
        for &id in &source.ethereal {
            self.add_to_ethereal_list(id);
        }
        for &id in &source.solid {
            if !self.in_solid_list(id) {
                self.solid.push(id);
            }
        }
    }

    /// `EtherealWeenieType::AddToSolidList`: each id once.
    pub fn add_to_solid_list(&mut self, id: InstanceId) -> bool {
        if !self.in_solid_list(id) {
            self.solid.push(id);
        }
        true
    }
}

/// `CollisionParameters` bits (`m_bitfield`): which of the placement and movement sets, and the
/// physics-type mask, have been set by a script.
const MOVEMENT_PHYSICS_BIT: u32 = 0x1;
const MOVEMENT_WEENIE_BIT: u32 = 0x2;
const PLACEMENT_PHYSICS_BIT: u32 = 0x4;
const PLACEMENT_WEENIE_BIT: u32 = 0x8;
const PHYSICS_TYPE_BIT: u32 = 0x10;

/// The collision parameters of one body: the placement and movement physics-type masks and
/// weenie lists, and the body's own physics type (`CollisionParameters`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollisionParameters {
    pub(crate) bitfield: u32,
    pub(crate) physics_type: PhysicsTypeMask,
    pub(crate) placement_physics: PhysicsTypeMask,
    pub(crate) movement_physics: PhysicsTypeMask,
    pub(crate) placement_weenie: WeenieTypes,
    pub(crate) movement_weenie: WeenieTypes,
    /// `CollisionParameters::m_temp_ethereal_list`: a plain id list, separate from the weenie
    /// sets, that the collision manager fills per frame.
    temp_ethereal: Vec<InstanceId>,
}

impl CollisionParameters {
    /// `CollisionParameters::FindInTempEtherealList`: the index of the first matching id.
    pub fn find_in_temp_ethereal_list(&self, id: InstanceId) -> Option<usize> {
        self.temp_ethereal.iter().position(|&held| held == id)
    }

    /// `CollisionParameters::InTempEtherealList`.
    pub fn in_temp_ethereal_list(&self, id: InstanceId) -> bool {
        self.find_in_temp_ethereal_list(id).is_some()
    }

    /// `CollisionParameters::AddToTempEtherealList`: an id already in the list is not added
    /// again; otherwise it is appended.
    pub fn add_to_temp_ethereal_list(&mut self, id: InstanceId) -> bool {
        if !self.in_temp_ethereal_list(id) {
            self.temp_ethereal.push(id);
        }
        true
    }

    /// `CollisionParameters::RemoveFromTempEtherealList`: a found id is removed by moving the
    /// last entry into its slot. Retail returns 1 whether or not the id was held.
    pub fn remove_from_temp_ethereal_list(&mut self, id: InstanceId) -> bool {
        if let Some(index) = self.find_in_temp_ethereal_list(id) {
            self.temp_ethereal.swap_remove(index);
        }
        true
    }

    /// `CollisionParameters::UpdateMovementEthereal` / `UpdateMovementSolid`: the first update
    /// clears the movement mask, then the property's bit is set (ethereal) or cleared (solid).
    pub fn update_movement(
        &mut self,
        name: u64,
        ethereal: bool,
        exists: impl Fn(u64) -> bool,
    ) -> bool {
        Self::update(
            &mut self.bitfield,
            MOVEMENT_PHYSICS_BIT,
            &mut self.movement_physics,
            name,
            ethereal,
            exists,
        )
    }

    /// `CollisionParameters::UpdatePlacementEthereal` / `UpdatePlacementSolid`.
    pub fn update_placement(
        &mut self,
        name: u64,
        ethereal: bool,
        exists: impl Fn(u64) -> bool,
    ) -> bool {
        Self::update(
            &mut self.bitfield,
            PLACEMENT_PHYSICS_BIT,
            &mut self.placement_physics,
            name,
            ethereal,
            exists,
        )
    }

    /// `CollisionParameters::SetBit`: ORs a raw flag into the set's bitfield (`FXManager::
    /// DoCreateObjectFX` sets `0x10` on an FX object's parameters).
    pub fn set_bit(&mut self, bit: u32) {
        self.bitfield |= bit;
    }

    /// `CollisionParameters::SetWeeniePlacementType`: the placement weenie set takes
    /// `wtype` (`EtherealWeenieType::Update`) and its bit is set.
    pub fn set_weenie_placement_type(&mut self, wtype: &WeenieTypes) {
        self.placement_weenie.update_from(wtype);
        self.set_bit(PLACEMENT_WEENIE_BIT);
    }

    /// `CollisionParameters::SetWeenieMovementType`.
    pub fn set_weenie_movement_type(&mut self, wtype: &WeenieTypes) {
        self.movement_weenie.update_from(wtype);
        self.set_bit(MOVEMENT_WEENIE_BIT);
    }

    /// `CollisionParameters::SetPhysicsPlacementType`: the placement physics mask is replaced.
    pub fn set_physics_placement_type(&mut self, ptype: PhysicsTypeMask) {
        self.placement_physics = ptype;
        self.set_bit(PLACEMENT_PHYSICS_BIT);
    }

    /// `CollisionParameters::SetPhysicsMovementType`.
    pub fn set_physics_movement_type(&mut self, ptype: PhysicsTypeMask) {
        self.movement_physics = ptype;
        self.set_bit(MOVEMENT_PHYSICS_BIT);
    }

    /// `CCollisionManager::GetCollisionState`: the state a script reads. The placement physics
    /// mask is this body's, the movement mask is the manager's `m_desired_physics_mask`
    /// (`desired`); only those two sets are given.
    pub fn collision_state(&self, desired: PhysicsTypeMask) -> CollisionParameters {
        let mut state = CollisionParameters::default();
        state.set_physics_placement_type(self.placement_physics);
        state.set_physics_movement_type(desired);
        state
    }

    /// `CCollisionManager::UpdateCurrentPhysicsMask`: the movement physics mask gains the
    /// desired mask's types.
    pub fn update_current_physics_mask(&mut self, desired: PhysicsTypeMask) {
        self.movement_physics =
            PhysicsTypeMask::from_bits(self.movement_physics.bits() | desired.bits());
        self.set_bit(MOVEMENT_PHYSICS_BIT);
    }

    /// `CollisionParameters::UpdatePhysicsType` (`ethereal` true) and `RemovePhysicsType`.
    pub fn update_physics_type(
        &mut self,
        name: u64,
        set: bool,
        exists: impl Fn(u64) -> bool,
    ) -> bool {
        Self::update(
            &mut self.bitfield,
            PHYSICS_TYPE_BIT,
            &mut self.physics_type,
            name,
            set,
            exists,
        )
    }

    fn update(
        bitfield: &mut u32,
        flag: u32,
        mask: &mut PhysicsTypeMask,
        name: u64,
        set: bool,
        exists: impl Fn(u64) -> bool,
    ) -> bool {
        if *bitfield & flag == 0 {
            *mask = PhysicsTypeMask::default();
            *bitfield |= flag;
        }
        mask.set_property(name, set, exists)
    }

    /// Whether the body's own physics type is named (`TestBitMask(0x10)`).
    pub fn has_physics_type(&self) -> bool {
        self.bitfield & PHYSICS_TYPE_BIT != 0
    }

    /// Whether the movement physics mask is named (`TestBitMask(1)`).
    pub fn has_movement_physics(&self) -> bool {
        self.bitfield & MOVEMENT_PHYSICS_BIT != 0
    }

    /// `CCollisionManager::SetCollisionState`: merges a script's `info` into this body's sets,
    /// only for the sets `info`'s bits name. A movement or placement physics mask, and the body's
    /// physics type (`CPhysicsObj::SetPhysicsCollideMask`), are replaced; a weenie set merges
    /// (`EtherealWeenieType::Update`), so ids the body already holds stay.
    pub fn merge_set_state(&mut self, info: &CollisionParameters) {
        if info.bitfield & MOVEMENT_PHYSICS_BIT != 0 {
            self.movement_physics = info.movement_physics;
            self.bitfield |= MOVEMENT_PHYSICS_BIT;
        }
        if info.bitfield & MOVEMENT_WEENIE_BIT != 0 {
            self.movement_weenie.update_from(&info.movement_weenie);
            self.bitfield |= MOVEMENT_WEENIE_BIT;
        }
        if info.bitfield & PLACEMENT_PHYSICS_BIT != 0 {
            self.placement_physics = info.placement_physics;
            self.bitfield |= PLACEMENT_PHYSICS_BIT;
        }
        if info.bitfield & PLACEMENT_WEENIE_BIT != 0 {
            self.placement_weenie.update_from(&info.placement_weenie);
            self.bitfield |= PLACEMENT_WEENIE_BIT;
        }
        if info.bitfield & PHYSICS_TYPE_BIT != 0 {
            self.physics_type = info.physics_type;
            self.bitfield |= PHYSICS_TYPE_BIT;
        }
    }

    /// `CollisionParameters::AddToEtherealList`: the id joins the placement and movement
    /// ethereal lists. A set that was never touched starts solid (`SetToSolid`) first.
    pub fn add_to_ethereal_list(&mut self, id: InstanceId) -> bool {
        if self.bitfield & PLACEMENT_WEENIE_BIT == 0 {
            self.placement_weenie = WeenieTypes::default();
            self.bitfield |= PLACEMENT_WEENIE_BIT;
        }
        if self.bitfield & MOVEMENT_WEENIE_BIT == 0 {
            self.movement_weenie = WeenieTypes::default();
            self.bitfield |= MOVEMENT_WEENIE_BIT;
        }
        let placed = self.placement_weenie.add_to_ethereal_list(id);
        let moved = self.movement_weenie.add_to_ethereal_list(id);
        placed && moved
    }

    /// The body's own physics-type mask (`CollisionParameters::m_physics_type`).
    pub fn physics_type(&self) -> PhysicsTypeMask {
        self.physics_type
    }

    /// `gCollisionParameters::CallEFun_SetPhysicsType`: the physics-type mask is replaced and its
    /// bit (0x10) set.
    pub fn set_physics_type(&mut self, ptype: PhysicsTypeMask) {
        self.physics_type = ptype;
        self.set_bit(PHYSICS_TYPE_BIT);
    }

    /// `CollisionParameters::m_physics_type`, to write raw words into
    /// (`gCollisionParameters::CallEFun_SetPhysicsTypeLowDWord` and its kin set no bitfield bit).
    pub fn physics_type_mut(&mut self) -> &mut PhysicsTypeMask {
        &mut self.physics_type
    }

    /// `CollisionParameters::m_placement_physics`.
    pub fn placement_physics(&self) -> PhysicsTypeMask {
        self.placement_physics
    }

    /// `CollisionParameters::m_placement_physics`, to write raw words into.
    pub fn placement_physics_mut(&mut self) -> &mut PhysicsTypeMask {
        &mut self.placement_physics
    }

    /// `CollisionParameters::m_movement_physics`.
    pub fn movement_physics(&self) -> PhysicsTypeMask {
        self.movement_physics
    }

    /// `CollisionParameters::m_movement_physics`, to write raw words into.
    pub fn movement_physics_mut(&mut self) -> &mut PhysicsTypeMask {
        &mut self.movement_physics
    }

    /// `CollisionParameters::m_placement_weenie`.
    pub fn placement_weenie(&self) -> &WeenieTypes {
        &self.placement_weenie
    }

    /// `CollisionParameters::m_movement_weenie`.
    pub fn movement_weenie(&self) -> &WeenieTypes {
        &self.movement_weenie
    }

    /// `CPhysicsObj::IsBlocking(wtype, ptype)` against this body's sets: the placement or the
    /// movement set, as the transition runs. `object_mask` is the object's blocking physics-type
    /// mask (`m_blocking_mask`). When the masks share a type the object blocks only if the id is
    /// in the solid list; otherwise it blocks unless the id is in the ethereal list.
    pub fn blocks(&self, id: InstanceId, object_mask: PhysicsTypeMask, placement: bool) -> bool {
        let (weenie, ptype) = if placement {
            (&self.placement_weenie, self.placement_physics)
        } else {
            (&self.movement_weenie, self.movement_physics)
        };
        is_blocking(weenie, ptype, id, object_mask)
    }
}
