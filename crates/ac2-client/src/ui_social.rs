//! The social pages' player side: the answers they read from the player (`ClientPlayer` implements
//! their sources), and the calls they queue become gameplay commands in `hud_request`. The pages' notices,
//! dialogs and request mapping live in `ac2_ui::social_notices`.

use ac2_script::host::social_queries::SocialQueries;
use ac2_ui::{AllegianceSource, SkillVar};
use ac2_ui::{FellowStrings, FellowshipSource, SkillSource};

use crate::ui_sources::ClientPlayer;

impl FellowStrings for ClientPlayer<'_> {
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        SkillSource::render(self, string_id, vars)
    }
}

impl FellowshipSource for ClientPlayer<'_> {
    fn player(&self) -> u64 {
        self.frame.player.as_ref().map_or(0, |p| p.id)
    }

    /// `gmCWInterface::GetFellowXPHash`: the shares the player's script fills in.
    fn fellow_xp(&self) -> Option<Vec<(u64, f32)>> {
        self.client.borrow_mut().fellow_xp_hash().ok()
    }
}

impl AllegianceSource for ClientPlayer<'_> {
    fn player(&self) -> u64 {
        FellowshipSource::player(self)
    }

    fn render_wire(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.render_info(info)
    }
}
