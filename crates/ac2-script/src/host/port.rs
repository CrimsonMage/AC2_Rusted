//! The script host's view of the client (`ac2_client::Client`): the player's id, the attached VM,
//! and where a call's output goes. The client implements it; the script-side queries
//! ([`super::queries::ScriptQueries`]) are written against this trait, so they live here and
//! name no client type.

use ac2_weenie::SessionEvent;

use super::item_queries::ItemLook;
use super::{DispatchOutcome, ScriptHost};

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
}
