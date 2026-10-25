//! `gmAllegianceUI` on the real toolbar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;
use ac2_net::protocol::StringInfo;
use ac2_weenie::{AllegianceMember, AllegianceProfile};

use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use crate::{
    build_gameplay_hud, AllegianceRequest, AllegianceSource, DialogOutcome, SkillVar, UiString,
    UiStrings, UiTree,
};

const PLAYER: u64 = 0x2100_0000_0000_0001;
const PATRON: u64 = 0x2100_0000_0000_0002;
const VASSAL: u64 = 0x2100_0000_0000_0003;

/// Strings as `id[vars]`, server strings as their literal.
struct Player;

impl UiStrings for Player {
    fn render_string(&self, string: &UiString) -> String {
        let vars: Vec<String> = string
            .vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::UInt(n) => n.to_string(),
                SkillVar::Wire(info) => self.render_wire(info),
                SkillVar::Text(t) => t.clone(),
                SkillVar::Info(inner) => self.render_string(inner),
                other => format!("{other:?}"),
            })
            .collect();
        match &string.literal {
            Some(literal) => literal.clone(),
            None => format!("{:08x}[{}]", string.string_id, vars.join(",")),
        }
    }
}

impl AllegianceSource for Player {
    fn player(&self) -> u64 {
        PLAYER
    }

    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

fn member(id: u64, name: &str, level: u32) -> AllegianceMember {
    AllegianceMember {
        id,
        name: StringInfo {
            literal: Some(name.into()),
            ..StringInfo::default()
        },
        level,
        species: 1,
        gender: 0x1000,
        xp_passed: 1234,
        ..AllegianceMember::default()
    }
}

fn find(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(name)))
        .unwrap_or_else(|| panic!("{name}"))
}

fn text(tree: &UiTree, row: usize, name: &str) -> String {
    let element = tree.child_recursive(row, tokenize(name)).unwrap();
    tree.get(element).unwrap().text().unwrap_or_default()
}

fn press(tree: &UiTree, element: usize) -> UiMessage {
    UiMessage {
        element_id: tree.get(element).unwrap().id(),
        element: Some(element),
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
    }
}

#[test]
fn the_allegiance_page_shows_the_player_with_patron_and_vassals() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut page = hud.allegiance.take().expect("the allegiance page");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.set_player(tree, PLAYER);
    // The social field is hidden: nobody's profile is asked for.
    assert_eq!(page.take_requests(), vec![AllegianceRequest::Query(0)]);

    let profile = AllegianceProfile {
        monarch: member(PATRON, "Thorn", 30),
        patron: member(PATRON, "Thorn", 30),
        member: member(PLAYER, "Explorer", 12),
        member_online: true,
        patron_online: true,
        vassals: vec![(member(VASSAL, "Birch", 0), true)],
        total: 3,
        name: StringInfo {
            literal: Some("Oak Hall".into()),
            ..StringInfo::default()
        },
        ..AllegianceProfile::default()
    };
    page.update_profile(tree, &Player, &profile);
    let allegiance_page = find(tree, "AllegiancePage");
    assert_eq!(
        tree.get(allegiance_page).unwrap().state(),
        tokenize("allegiance")
    );
    assert_eq!(text(tree, allegiance_page, "AllegianceName"), "Oak Hall");
    // The player, the vassal, the message of the day.
    let rows = page.rows(tree);
    assert_eq!(rows.len(), 3, "{rows:?}");
    let name = text(tree, rows[0], "MemberNameText");
    let expected = format!(
        "{:08x}[,Explorer,12,{:08x}[]]",
        tokenize("ID_Allegiance_TitleAndNameLevelSpeciesGender"),
        tokenize("ID_RaceAndSex_HumanMale")
    );
    assert_eq!(name, expected);
    let patron = text(tree, rows[0], "MemberMiscText");
    assert!(patron.contains("Thorn"), "{patron}");
    // A level 0 vassal shows the experience it passed up.
    let misc = text(tree, rows[1], "MemberMiscText");
    assert!(
        misc.starts_with(&format!("{:08x}[,1234", tokenize("ID_Allegiance_XpUp"))),
        "{misc}"
    );
    // The player is the target: Home is ghosted, Up is not (there is a patron); Break is live
    // only away from the player.
    let home = tree
        .child_recursive(rows[0], tokenize("HomeButton"))
        .unwrap();
    assert!(tree.bool_attribute(home, BUTTON_GHOSTED));
    let breaker = find(tree, "BreakButton");
    assert!(tree.bool_attribute(breaker, BUTTON_GHOSTED));

    // Down to the vassal asks for its profile.
    let down = tree
        .child_recursive(rows[1], tokenize("DownButton"))
        .unwrap();
    let message = press(tree, down);
    page.message(tree, &mut dialogs, &Player, &message);
    assert!(page
        .take_requests()
        .contains(&AllegianceRequest::Query(VASSAL)));

    // Breaking asks first; yes sends it.
    let mut vassal_view = profile.clone();
    vassal_view.member = member(VASSAL, "Birch", 0);
    vassal_view.patron = member(PLAYER, "Explorer", 12);
    vassal_view.vassals.clear();
    page.update_profile(tree, &Player, &vassal_view);
    page.take_requests();
    let message = press(tree, breaker);
    page.message(tree, &mut dialogs, &Player, &message);
    let (context, _) = dialogs.shown().expect("the break dialog");
    page.close_confirmation(tree, context, true);
    dialogs.close_dialog(tree, context);
    assert!(page
        .take_requests()
        .contains(&AllegianceRequest::Break(VASSAL)));
    let _ = DialogOutcome::Confirmation { context, yes: true };
}
