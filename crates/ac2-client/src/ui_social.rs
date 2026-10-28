//! The social pages' player side: the answers they read from the player (`ClientPlayer` implements
//! their sources), and the calls they queue becoming gameplay commands. The pages' notices,
//! dialogs and request mapping live in `ac2_ui::social_notices`.

use ac2_script::host::social_queries::SocialQueries;
use ac2_ui::{AllegianceRequest, FellowStrings, FellowshipRequest, FellowshipSource, SkillSource};
use ac2_ui::{AllegianceSource, SkillVar, SocialCall};

use crate::gameplay::Command;
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

/// The calls as gameplay commands; an assisted tell's name is returned for the chat.
fn social_commands(calls: Vec<SocialCall>, commands: &mut Vec<Command>) -> Vec<String> {
    let mut tells = Vec::new();
    for call in calls {
        match call {
            SocialCall::Social(request) => commands.push(Command::Social(request)),
            SocialCall::Select(id) => commands.push(Command::Select(Some(id))),
            SocialCall::AssistedTell(name) => tells.push(name),
        }
    }
    tells
}

/// The allegiance page's requests as commands.
pub(crate) fn allegiance_commands(requests: Vec<AllegianceRequest>, commands: &mut Vec<Command>) {
    social_commands(ac2_ui::allegiance_calls(requests), commands);
}

/// The fellowship page's requests as commands; an assisted tell's name is returned for the chat.
pub(crate) fn fellowship_commands(
    requests: Vec<FellowshipRequest>,
    selection: Option<u64>,
    commands: &mut Vec<Command>,
) -> Vec<String> {
    social_commands(ac2_ui::fellowship_calls(requests, selection), commands)
}
