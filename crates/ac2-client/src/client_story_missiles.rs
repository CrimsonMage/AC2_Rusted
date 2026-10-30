//! A story's missiles (`StoryManager::CreateAndParentMissiles`, `FireMissile`, `CreateMissile`,
//! the missile half of `CleanupStory`): the script's `MissileInfo` for a fire hook
//! (`gmCWInterface::GetMissileInformationFromWSL`) names the missile's entity, the attacker's
//! holding location it waits in, and its flight, targeting and collision parameters. The client
//! makes the missile itself as a temporary object (`CObjectMaint::CreateTemporaryObject`, gone
//! after `MISSILE_TIMEOUT_TIME`), hangs it from the attacker until the fire hook, and fires it
//! (`CPhysicsObj::MissilePlaceAndFire`); the missile manager then flies it like a server's.
//!
//! Each object's story manager is [`Stories`]'s, beside what retail's `StoryManager` keeps for
//! its cleanup: the missiles made ahead of their fire hook (`m_pendingMissiles`) and the thrown
//! weapons faded out (`m_fadedObjects`).

use std::collections::{BTreeMap, HashMap};

use ac2_net::protocol::{PhysicsStory, StoryHook};
use ac2_physics::{ObjectLookup, ObjectView, StoryManager};
use ac2_script::host::interface_calls::InterfaceCalls;
use ac2_script::native::{MissileInfo, NativeObject};

use crate::client::Client;
use crate::remote::{Context, RemoteCalls};

/// `MISSILE_TIMEOUT_TIME` (8.0): a story missile is destroyed this long after it is made
/// (`CObjectMaint::InitializeTemporaryObject`'s destruction queue).
const MISSILE_TIMEOUT_TIME: f64 = 8.0;
/// The opacity slider a thrown weapon fades on (`CPhysicsObj::DoSlider(0, 0x40000015, ...)`).
const OPACITY_SLIDER: u32 = 0x4000_0015;

/// The objects' story managers and their missiles.
#[derive(Default)]
pub(crate) struct Stories {
    pub(crate) managers: HashMap<u64, StoryManager>,
    /// Per story object: `m_pendingMissiles` and `m_fadedObjects`.
    held: HashMap<u64, Held>,
    /// The story missiles alive, by id.
    launches: BTreeMap<u64, Launch>,
}

#[derive(Default)]
struct Held {
    pending: Vec<u64>,
    faded: Vec<u64>,
}

/// A story missile: what `CreateMissile` hands its body once the body is in, the holder it
/// hangs from (`SetParent`) until it fires, the bone it sticks to when it hits
/// (`StickySetParent`), and when it times out.
struct Launch {
    info: MissileInfo,
    /// A clone's look: the weapon's local `VisualDesc` (`SetVisualDesc(GetLocalVisualDesc())`).
    appearance: Option<std::sync::Arc<ac2_weenie::Appearance>>,
    parent: Option<(u64, u32)>,
    stuck: Option<(u64, u32, ac2_physics::ChildFrame)>,
    initialized: bool,
    fire: bool,
    fired: bool,
    expires: f64,
}

impl Stories {
    /// The object's story manager and missile lists go with it.
    pub(crate) fn remove_object(&mut self, object: u64) {
        self.managers.remove(&object);
        self.held.remove(&object);
    }

    /// The object a story missile hangs from and how: a holding location until it fires
    /// (`SetParent`), a bone once it sticks (`StickySetParent`).
    pub(crate) fn missile_holder(&self, id: u64) -> Option<(u64, ac2_physics::ChildInfo)> {
        let launch = self.launches.get(&id)?;
        if let Some((holder, bone, offset)) = launch.stuck {
            let info = ac2_physics::ChildInfo {
                id,
                kind: ac2_physics::ChildType::BoneIndex,
                location: bone,
                offset,
                remove_rotation: false,
            };
            return Some((holder, info));
        }
        let (holder, location) = launch.parent?;
        Some((holder, ac2_physics::ChildInfo::holding(id, location)))
    }

    /// The local `VisualDesc` a cloned missile took from its weapon.
    pub(crate) fn missile_appearance(
        &self,
        id: u64,
    ) -> Option<std::sync::Arc<ac2_weenie::Appearance>> {
        self.launches.get(&id)?.appearance.clone()
    }

    /// `CPhysicsObj::StickySetParent` for a story missile: it hangs from `bone` of `object` from
    /// now on. `false` when `missile` is not a story missile.
    pub(crate) fn stick(
        &mut self,
        missile: u64,
        object: u64,
        bone: u32,
        offset: ac2_physics::ChildFrame,
    ) -> bool {
        let Some(launch) = self.launches.get_mut(&missile) else {
            return false;
        };
        launch.parent = None;
        launch.stuck = Some((object, bone, offset));
        true
    }

    /// The story missiles alive.
    #[cfg(feature = "test-support")]
    pub(crate) fn missiles(&self) -> Vec<u64> {
        self.launches.keys().copied().collect()
    }
}

/// The objects a missile's targeting reads (`CObjectMaint::GetObjectA`).
struct Views(HashMap<u64, ObjectView>);

impl ObjectLookup for Views {
    fn object(&self, id: u64) -> Option<ObjectView> {
        self.0.get(&id).copied()
    }
}

impl Client {
    /// `StoryManager::CreateAndParentMissiles` (a hook of type 2): with no missile pending, each
    /// fire hook of the story whose missile waits in a holding location has it made now and hung
    /// from the attacker there; a thrown weapon fades out meanwhile.
    pub(crate) fn create_and_parent_missiles(&mut self, object: u64, story: &PhysicsStory) {
        if self
            .stories
            .held
            .get(&object)
            .is_some_and(|held| !held.pending.is_empty())
        {
            return;
        }
        for hook in story.hooks.iter().filter(|h| h.kind() == FIRE_MISSILE_HOOK) {
            let Some(info) = self.story_missile_info(story, hook) else {
                continue;
            };
            if info.holding_location == 0 {
                continue;
            }
            let location = info.holding_location;
            let thrown = info.thrown;
            let Some(id) = self.create_missile(object, hook, info) else {
                continue;
            };
            if let Some(launch) = self.stories.launches.get_mut(&id) {
                launch.parent = Some((object, location));
            }
            let held = self.stories.held.entry(object).or_default();
            if thrown {
                if let Some(weapon) = hook.weapon {
                    held.faded.push(weapon);
                    if let Some(body) = self.story_body(weapon) {
                        body.do_slider(0, OPACITY_SLIDER, 0.0, 0.0);
                    }
                }
            }
            self.stories
                .held
                .entry(object)
                .or_default()
                .pending
                .push(id);
        }
    }

    /// `StoryManager::FireMissile` (a hook of type 3): a missile with no holding location is made
    /// now; otherwise the first pending one is taken. Either is placed and fired
    /// (`CPhysicsObj::MissilePlaceAndFire`) as soon as its body is in.
    pub(crate) fn fire_missile(&mut self, object: u64, story: &PhysicsStory, hook: &StoryHook) {
        let Some(info) = self.story_missile_info(story, hook) else {
            return;
        };
        let id = if info.holding_location == 0 {
            self.create_missile(object, hook, info)
        } else {
            self.stories
                .held
                .get_mut(&object)
                .filter(|held| !held.pending.is_empty())
                .map(|held| held.pending.remove(0))
        };
        if let Some(launch) = id.and_then(|id| self.stories.launches.get_mut(&id)) {
            launch.fire = true;
        }
    }

    /// `StoryManager::CleanupStory`'s missile half: the missiles still pending are destroyed
    /// (`CObjectMaint::DestroyObject`) and the faded weapons fade back in over half a second.
    pub(crate) fn cleanup_story_missiles(&mut self, object: u64) {
        let Some(held) = self.stories.held.remove(&object) else {
            return;
        };
        for id in held.pending {
            self.stories.launches.remove(&id);
            self.remotes.removed(id, &mut self.terrain);
        }
        for weapon in held.faded {
            if let Some(body) = self.story_body(weapon) {
                body.do_slider(0, OPACITY_SLIDER, 1.0, 0.5);
            }
        }
    }

    /// Each physics quantum, before the objects step: a missile whose body has come in takes its
    /// collision state and its missile manager (`SetCollisionState`, `MissileInitialize`); a
    /// hung one starts from its holder's holding location (`m_posDrawing` while it `HasParent`);
    /// a fired one is placed and fired, then let go of its holder; timed-out and vanished ones
    /// are forgotten (`CObjectMaint`'s temporary destruction queue).
    pub(crate) fn story_missiles_quantum(&mut self) {
        if self.stories.launches.is_empty() {
            return;
        }
        let now = self.session.cur_time();
        let views = Views(self.object_views());
        let ids: Vec<u64> = self.stories.launches.keys().copied().collect();
        for id in ids {
            if self.remotes.get(id).is_none() {
                self.stories.launches.remove(&id);
                continue;
            }
            if self.stories.launches[&id].expires <= now {
                self.stories.launches.remove(&id);
                self.remotes.removed(id, &mut self.terrain);
                continue;
            }
            // A stuck missile is out of the world, drawn on its bone.
            if self.stories.launches[&id].stuck.is_some() {
                continue;
            }
            let parent = self.stories.launches[&id].parent;
            let placed = parent.and_then(|(holder, location)| {
                self.held_drawing_position(holder, &ac2_physics::ChildInfo::holding(id, location))
            });
            let terrain = &self.terrain;
            let (Some(launch), Some(body)) = (
                self.stories.launches.get_mut(&id),
                self.remotes.body_mut(id),
            ) else {
                continue;
            };
            if !launch.initialized {
                let collision = &launch.info.collision_params;
                body.set_collision_state(&collision.sets, &|_| None);
                body.set_collision_minimums(
                    collision.min_reported_impact_xy,
                    collision.min_reported_impact_z,
                );
                body.missile_initialize(
                    terrain,
                    &views,
                    &launch.info.physics_parameters(),
                    &launch.info.physics_targeting(),
                );
                launch.initialized = true;
            }
            if launch.fired {
                // Placed last quantum: the missile no longer hangs from its holder.
                launch.parent = None;
                body.set_parent_drawing_position(None);
                continue;
            }
            body.set_parent_drawing_position(placed);
            if launch.fire {
                body.missile_request(true, true);
                launch.fired = true;
            }
        }
    }

    /// `StoryManager::CreateMissile`: a temporary object of the missile's entity (or, for a
    /// clone, of the weapon's data id with the weapon's look) at the story's object, timed out after
    /// `MISSILE_TIMEOUT_TIME`. Its collision state and missile manager wait for its body.
    fn create_missile(&mut self, object: u64, hook: &StoryHook, info: MissileInfo) -> Option<u64> {
        // A clone is made from the weapon's data id and takes its local `VisualDesc`.
        let weapon = hook
            .weapon
            .filter(|_| info.clone)
            .and_then(|weapon| self.session.objects().get(weapon))
            .and_then(|o| Some((o.visual?, o.appearance.clone())));
        let (visual, appearance) = match weapon {
            Some((visual, appearance)) => (visual, Some(appearance)),
            None => (info.entity_did, None),
        };
        if visual == 0 {
            return None;
        }
        let position = self.story_body(object).map(|body| body.position())?;
        // `CObjectMaint::AllocateTemporaryID`; none left, no missile.
        let id = self.remotes.allocate_temporary_id()?;
        let player = self
            .player
            .as_ref()
            .filter(|p| p.body.is_some())
            .map(|p| p.position());
        let mut cx = Context {
            session: &mut self.session,
            terrain: &mut self.terrain,
            player,
        };
        // Drawn at once, with no fade in.
        self.remotes.add_local(
            id,
            visual,
            position,
            None,
            ac2_smartbox::remotes::LocalStart {
                alpha_moves: [(1.0, 0.0), (1.0, 0.0)],
                behavior: 0,
            },
            &mut cx,
        );
        self.request_remote_bodies();
        let expires = self.session.cur_time() + MISSILE_TIMEOUT_TIME;
        self.stories.launches.insert(
            id,
            Launch {
                info,
                appearance,
                parent: None,
                stuck: None,
                initialized: false,
                fire: false,
                fired: false,
                expires,
            },
        );
        Some(id)
    }

    /// `gmCWInterface::GetMissileInformationFromWSL` for a fire hook: the story's skill, the
    /// hook's attack hook number, the story's attacker and the hook's target. `None` without a
    /// script, when the call fails, or when the script named no missile.
    fn story_missile_info(
        &mut self,
        story: &PhysicsStory,
        hook: &StoryHook,
    ) -> Option<MissileInfo> {
        self.scripts.as_ref()?;
        let info = self
            .missile_info(
                story.skill.unwrap_or(0),
                hook.attack_hook.unwrap_or(0),
                story.attacker.unwrap_or(0),
                hook.target.unwrap_or(0),
            )
            .ok()??;
        match info {
            NativeObject::MissileInfo(info) => Some(*info),
            _ => None,
        }
    }

    /// The bodies a missile's targeting can name: the player's and every object's in the world.
    fn object_views(&self) -> HashMap<u64, ObjectView> {
        let mut views: HashMap<u64, ObjectView> = self
            .remotes
            .iter()
            .filter(|(_, remote)| remote.in_world())
            .filter_map(|(id, remote)| Some((id, remote.body()?.object_view())))
            .collect();
        if let Some(player) = self.player.as_ref().filter(|p| p.in_world) {
            if let Some(body) = player.body.as_ref() {
                views.insert(player.id, body.physics.object_view());
            }
        }
        views
    }
}

/// `StoryHookData`'s types for missiles: make and hang them (2), fire one (3).
pub(crate) const PARENT_MISSILES_HOOK: u32 = 2;
pub(crate) const FIRE_MISSILE_HOOK: u32 = 3;
