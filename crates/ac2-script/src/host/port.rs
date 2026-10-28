//! The script host's view of the client (`ac2_client::Client`): the player's id, the attached VM,
//! and where a call's output goes. The client implements it; the script-side queries
//! ([`super::queries::ScriptQueries`]) are written against this trait, so they live here and
//! name no client type.

use ac2_net::protocol::ObjectAction;
use ac2_physics::CollisionParameters;
use ac2_weenie::{Appearance, SessionEvent, WeenieQualities};

use super::item_queries::ItemLook;
use super::latent::LatentPhysics;
use super::outcome::ScriptState;
use super::{DataRequest, DispatchOutcome, ScriptHost};

/// The script facts the session keeps for an object (`CWDistributor::Construct`'s inputs): its
/// script package and its entity DID, when it has them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptIdentity {
    pub package: Option<u32>,
    pub entity: Option<u32>,
}

pub trait ScriptPort {
    /// The player's object id, when the session has one (`OnlineSession::player`).
    fn player_id(&self) -> Option<u64>;

    /// The attached script VM, when there is one.
    fn script_host(&mut self) -> Option<&mut ScriptHost>;

    /// The end of one call on object `id` (`function` is `None` for an IFun the UI made):
    /// its problems are noted, its output routed into `out` when the call is the VM's.
    fn route_outcome(
        &mut self,
        id: u64,
        function: Option<u32>,
        result: Result<DispatchOutcome, String>,
        out: &mut Vec<SessionEvent>,
    );

    /// Events the scripts produced outside a frame (a query's output), for the next frame.
    fn push_produced(&mut self, events: Vec<SessionEvent>);

    /// The session's view of object `id` for the item queries: its icon layers and weenie value,
    /// `None` when the session has no such object.
    fn item_look(&self, id: u64) -> Option<ItemLook>;

    /// The integer property `key` of object `id` (`ObjectQualities::inq_int`), when it has one.
    fn object_int(&self, id: u64, key: u32) -> Option<i32>;

    /// The script state of the client's VM (`None` when no VM is attached).
    fn script_state(&mut self) -> Option<&mut ScriptState>;

    /// Whether a stand-in of the session answers server call `function` (it was not handed to the
    /// VM, `SessionEvent` notices come from it).
    fn stand_in_handles(&self, function: u32) -> bool;

    /// A physics change a script asked of object `object`'s physics object (`gCWPhysicsObject`'s
    /// scale and FX efuns): applied to the player's body or a remote's.
    fn apply_script_physics(&mut self, object: u64, action: &ObjectAction);

    /// `CCollisionManager::SetCollisionState` minimums for object `object` (the impact minimums).
    fn apply_collision_minimums(&mut self, object: u64, xy: f32, z: f32);

    /// `CCollisionManager::SetCollisionState` sets for object `object` (the ethereal and
    /// physics-type sets the call names).
    fn apply_collision_sets(&mut self, object: u64, sets: &CollisionParameters);

    /// Starts a latent mode or behaviour on the object's body. `None` while the body runs it;
    /// `Some(error)` when it finished (or was refused) at once, with that error for the context.
    fn start_latent(&mut self, start: &LatentPhysics) -> Option<u32>;

    /// The session's chat shell takes a line the scripts entered (`CWInterface::ProcessText`),
    /// returning the events it raised.
    fn process_text(&mut self, line: &str) -> Vec<SessionEvent>;

    /// A server call the VM made, its packed arguments (`gCWInterface::SendServerEvt`).
    fn send_interp(&mut self, function: u32, arguments: Vec<u8>, encrypt: bool);

    /// Room chat the scripts sent (`gmCWInterface::RequestGUN_SendToRoom` on the Turbine path).
    fn send_to_room(&mut self, room: u32, text: &str, flags: u32);

    /// A DAT read the VM stopped for, handed to the loader; its answer resumes the waiting calls.
    fn request_script_data(&mut self, read: DataRequest);

    /// The script package and entity DID the session has for object `id`, `None` when it has no
    /// such object.
    fn script_identity(&self, id: u64) -> Option<ScriptIdentity>;

    /// A copy of object `id`'s qualities as the session keeps them, `None` without the object.
    fn object_qualities(&self, id: u64) -> Option<WeenieQualities>;

    /// A copy of object `id`'s local visual description as the session keeps it, `None` without
    /// the object.
    fn object_appearance(&self, id: u64) -> Option<Appearance>;

    /// Asks the loader for object `id`'s script state (package `package`, entity DID `did`); its
    /// answer comes back as `script_state_loaded`.
    fn request_script_state(&mut self, id: u64, package: u32, did: u32);
}
