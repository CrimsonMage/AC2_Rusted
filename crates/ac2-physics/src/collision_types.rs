//! Which objects a mover is blocked by: `EtherealPhysicsType` (a 64-bit physics-type mask),
//! `EtherealWeenieType` (a weenie mask plus solid and ethereal id lists), and the
//! `CollisionParameters` a script sets on a body (`CPhysicsObj::SetCollisionState`).
//! `CPhysicsObj::IsBlocking` decides, for one object, whether it stops the mover.

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

    /// `EtherealWeenieType::AddToSolidList`: each id once (tests build solid sets with it).
    #[cfg(test)]
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
    bitfield: u32,
    physics_type: PhysicsTypeMask,
    placement_physics: PhysicsTypeMask,
    movement_physics: PhysicsTypeMask,
    placement_weenie: WeenieTypes,
    movement_weenie: WeenieTypes,
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
        if object_mask.intersects(ptype) {
            weenie.in_solid_list(id)
        } else {
            !weenie.in_ethereal_list(id)
        }
    }
}
