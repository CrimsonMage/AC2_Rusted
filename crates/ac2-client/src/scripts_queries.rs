//! The UI's questions to the player's script (`gmCWInterface` -> `ggmCWInterface::CallIFun_*`
//! -> `CPlayer`). The queries themselves live in ac2-script ([`ac2_script::host::queries`]),
//! written against its [`ScriptPort`]; the client answers that port here and keeps these
//! delegates for its callers.

use ac2_net::protocol::ObjectAction;
use ac2_net::protocol::{EffectUiNode, GameplayOptionsProfile, ShortcutInfo};
use ac2_physics::CollisionParameters;
use ac2_script::host::item_queries::{ItemDisplay, ItemLook, ItemQueries};
use ac2_script::host::port::{ScriptIdentity, ScriptPort};
use ac2_script::host::queries::ScriptQueries;
use ac2_script::host::LatentPhysics;
use ac2_script::host::{
    DataRequest, DispatchOutcome, IFunArg, IFunResult, IFunReturn, ScriptHost, ScriptState,
};
use ac2_weenie::{Appearance, SessionEvent, WeenieQualities};

use crate::loader::Request;

pub use ac2_script::host::queries::{
    ifun, ChatSettings, ShortcutDisplay, ShortcutSettings, SkillLevel, SkillTime,
};

use crate::client::Client;

impl ScriptPort for Client {
    fn player_id(&self) -> Option<u64> {
        self.session.player()
    }

    fn script_host(&mut self) -> Option<&mut ScriptHost> {
        self.scripts.as_mut().map(|s| &mut s.host)
    }

    fn route_outcome(
        &mut self,
        id: u64,
        function: Option<u32>,
        result: Result<DispatchOutcome, String>,
        out: &mut Vec<SessionEvent>,
    ) {
        ac2_script::host::outcome::outcome(self, id, function, result, out);
    }

    fn script_state(&mut self) -> Option<&mut ScriptState> {
        self.scripts.as_mut()
    }

    fn stand_in_handles(&self, function: u32) -> bool {
        self.session.stand_in_handles(function)
    }

    fn apply_script_physics(&mut self, object: u64, action: &ObjectAction) {
        Client::apply_script_physics(self, object, action);
    }

    fn apply_body_state(&mut self, object: u64, state: ac2_script::host::body_state::BodyState) {
        Client::apply_body_state(self, object, state);
    }

    fn apply_collision_minimums(&mut self, object: u64, xy: f32, z: f32) {
        Client::apply_collision_minimums(self, object, xy, z);
    }

    fn apply_collision_sets(&mut self, object: u64, sets: &CollisionParameters) {
        Client::apply_collision_sets(self, object, sets);
    }

    fn start_latent(&mut self, start: &LatentPhysics) -> Option<u32> {
        self.start_latent_on_body(start)
    }

    fn process_text(&mut self, line: &str) -> Vec<SessionEvent> {
        self.chat_line(line)
    }

    fn display_help(&mut self, cmd: &str) -> Vec<SessionEvent> {
        self.session.display_help(cmd)
    }

    fn send_interp(&mut self, function: u32, arguments: Vec<u8>, encrypt: bool) {
        self.session.send_interp(function, arguments, encrypt);
    }

    fn send_to_room(&mut self, room: u32, text: &str, flags: u32) {
        self.session.send_to_room(room, text, flags);
    }

    fn push_outbox(&mut self, events: Vec<SessionEvent>) {
        self.script_outbox.extend(events);
    }

    fn request_script_data(&mut self, read: DataRequest) {
        self.loader.request(Request::ScriptData(read));
    }

    fn script_identity(&self, id: u64) -> Option<ScriptIdentity> {
        self.session.objects().get(id).map(|o| ScriptIdentity {
            package: o.script_package,
            entity: o.entity,
        })
    }

    fn object_qualities(&self, id: u64) -> Option<WeenieQualities> {
        self.session.objects().get(id).map(|o| o.qualities.clone())
    }

    fn object_appearance(&self, id: u64) -> Option<Appearance> {
        self.session
            .objects()
            .get(id)
            .map(|o| (*o.appearance).clone())
    }

    fn request_script_state(&mut self, id: u64, package: u32, did: u32) {
        self.loader.request(Request::ScriptState(id, package, did));
    }

    fn push_produced(&mut self, events: Vec<SessionEvent>) {
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(events);
        }
    }

    fn item_look(&self, id: u64) -> Option<ItemLook> {
        self.session.objects().get(id).map(|o| ItemLook {
            icon_layers: o.appearance.icon.clone(),
            value: o.qualities.weenie.as_ref().and_then(|w| w.value),
        })
    }

    fn object_int(&self, id: u64, key: u32) -> Option<i32> {
        self.session
            .objects()
            .get(id)
            .and_then(|o| o.qualities.inq_int(key))
    }
}

impl Client {
    pub fn query(
        &mut self,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Result<IFunResult, String> {
        ScriptQueries::query(self, function, args, ret)
    }

    pub fn can_skill_be_trained(&mut self, skill: u32) -> Result<u32, String> {
        ScriptQueries::can_skill_be_trained(self, skill)
    }

    pub fn can_skill_be_untrained(&mut self, skill: u32) -> Result<u32, String> {
        ScriptQueries::can_skill_be_untrained(self, skill)
    }

    pub fn can_skill_be_raised(&mut self, skill: u32) -> Result<u32, String> {
        ScriptQueries::can_skill_be_raised(self, skill)
    }

    pub fn skill_cost_to_train(&mut self, skill: u32) -> Result<u32, String> {
        ScriptQueries::skill_cost_to_train(self, skill)
    }

    pub fn is_hero_skill(&mut self, skill: u32) -> Result<bool, String> {
        ScriptQueries::is_hero_skill(self, skill)
    }

    pub fn can_auto_attack_target(&mut self, target: u64) -> Result<bool, String> {
        ScriptQueries::can_auto_attack_target(self, target)
    }

    pub fn skill_credits(&mut self) -> Result<u32, String> {
        ScriptQueries::skill_credits(self)
    }

    pub fn hero_skill_credits(&mut self) -> Result<u32, String> {
        ScriptQueries::hero_skill_credits(self)
    }

    pub fn skill_xp(&mut self, skill: u32) -> Result<u64, String> {
        ScriptQueries::skill_xp(self, skill)
    }

    pub fn is_skill_trained(&mut self, skill: u32) -> Result<bool, String> {
        ScriptQueries::is_skill_trained(self, skill)
    }

    pub fn is_skill_being_untrained(&mut self, skill: u32) -> Result<bool, String> {
        ScriptQueries::is_skill_being_untrained(self, skill)
    }

    pub fn is_skill_usable_by_inventory(&mut self, skill: u32) -> Result<bool, String> {
        ScriptQueries::is_skill_usable_by_inventory(self, skill)
    }

    pub fn is_skill_harmful(&mut self, skill: u32) -> Result<bool, String> {
        ScriptQueries::is_skill_harmful(self, skill)
    }

    pub fn use_shortcut(&mut self, index: u32) -> Result<u32, String> {
        ScriptQueries::use_shortcut(self, index)
    }

    pub fn skill_experience_to_level(&mut self, skill: u32) -> Result<u64, String> {
        ScriptQueries::skill_experience_to_level(self, skill)
    }

    pub fn skill_level(&mut self, skill: u32) -> Result<SkillLevel, String> {
        ScriptQueries::skill_level(self, skill)
    }

    pub fn skill_time_left(&mut self, skill: u32) -> Result<SkillTime, String> {
        ScriptQueries::skill_time_left(self, skill)
    }

    pub fn root_panel_from_race(&mut self, race: u32) -> Result<Option<u32>, String> {
        ScriptQueries::root_panel_from_race(self, race)
    }

    pub fn set_shortcut(&mut self, index: u32, shortcut: ShortcutInfo) -> Result<u32, String> {
        ScriptQueries::set_shortcut(self, index, shortcut)
    }

    pub fn shortcut_info(&mut self, index: u32) -> Result<ShortcutDisplay, String> {
        ScriptQueries::shortcut_info(self, index)
    }

    pub fn gameplay_options(&mut self) -> Result<GameplayOptionsProfile, String> {
        ScriptQueries::gameplay_options(self)
    }

    pub fn shortcut_array(&mut self) -> Result<Vec<ShortcutInfo>, String> {
        ScriptQueries::shortcut_array(self)
    }

    pub fn show_object_names(&mut self) -> bool {
        ScriptQueries::show_object_names(self)
    }

    pub fn show_radar_coords(&mut self) -> bool {
        ScriptQueries::show_radar_coords(self)
    }

    pub fn auto_target(&mut self) -> bool {
        ScriptQueries::auto_target(self)
    }

    pub fn shortcut_settings(&mut self) -> Result<ShortcutSettings, String> {
        ScriptQueries::shortcut_settings(self)
    }

    pub fn chat_settings(&mut self) -> Result<ChatSettings, String> {
        ScriptQueries::chat_settings(self)
    }

    pub fn item_display(&mut self, id: u64) -> Result<ItemDisplay, String> {
        ItemQueries::item_display(self, id)
    }

    pub fn container_capacity(&mut self, id: u64) -> Result<u32, String> {
        ItemQueries::container_capacity(self, id)
    }

    pub fn contents_info(&mut self, segment: u32) -> Result<(Vec<u64>, u32, u32), String> {
        ItemQueries::contents_info(self, segment)
    }

    pub fn container_segment_info(&mut self, segment: u32) -> Result<(u32, u32), String> {
        ItemQueries::container_segment_info(self, segment)
    }

    pub fn effect_list(&mut self) -> Result<Vec<EffectUiNode>, String> {
        ItemQueries::effect_list(self)
    }

    pub fn coin_max_stack_size(&mut self) -> Option<i32> {
        ItemQueries::coin_max_stack_size(self)
    }

    pub fn melts_to_money(&self, item: u64) -> bool {
        ItemQueries::melts_to_money(self, item)
    }
}
