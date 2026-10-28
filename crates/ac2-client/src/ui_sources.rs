//! The player as the UI frameworks see it for one frame: the skills panel's and the shortcut bar's
//! answers come from the player's script and the frame's mirror (`ClientPlayer`'s queries), and
//! the UI text comes from `ac2_ui::UiText` over the tables the client preloaded.

use std::cell::RefCell;

use ac2_db::PreloadedStringTables;
use ac2_ui::{
    Shortcut, ShortcutOptions, ShortcutSource, SkillNode, SkillPanel, SkillSource, SkillStatus,
    SkillVar, UiStrings, UiText,
};

use crate::client::Client;
use crate::frame::ClientFrame;

/// The player as the UI frameworks see it for one frame.
pub(crate) struct ClientPlayer<'a> {
    pub client: RefCell<&'a mut Client>,
    pub frame: &'a ClientFrame,
    pub strings: &'a PreloadedStringTables,
    /// The UI string table's DID (enum `0x41000018`).
    pub ui_table: u32,
    /// The DIDs of the tables the UI names by enum.
    pub enums: &'a std::collections::HashMap<u32, u32>,
}

impl ClientPlayer<'_> {
    /// The UI text view over the frame's tables.
    fn ui(&self) -> UiText<'_> {
        UiText::new(self.strings, self.ui_table, self.enums)
    }

    /// A wire `StringInfo` rendered from the tables.
    pub(crate) fn render_info(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.ui().render_info(info)
    }

    fn text(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.ui().text(info)
    }
}

impl ac2_ui::UiStrings for ClientPlayer<'_> {
    fn render_string(&self, string: &ac2_ui::UiString) -> String {
        self.ui().render_string(string)
    }
}

impl ac2_ui::AssistanceSource for ClientPlayer<'_> {
    fn world_name(&self) -> String {
        self.client.borrow().world_name().to_owned()
    }

    fn call_create(&self, call_type: u32, world: &str, target: &str, description: &str) -> bool {
        self.client
            .borrow_mut()
            .call_create(call_type, world, target, description)
    }
}

impl ac2_ui::PetSource for ClientPlayer<'_> {
    fn render_wire(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.render_info(info)
    }
}

impl ac2_ui::StatusSource for ClientPlayer<'_> {
    /// `gmEffectsUI`'s list: `InqEffectList`, as the effect strip's entries.
    fn effects(&self) -> Vec<ac2_ui::EffectEntry> {
        let nodes = self.client.borrow_mut().effect_list().unwrap_or_default();
        nodes
            .into_iter()
            .map(|n| ac2_ui::EffectEntry {
                eid: n.eid,
                pulsed: n.is_pulsed(),
                name: n.name,
                icon: n.icon,
                time_cast: n.time_cast,
                duration: n.duration,
            })
            .collect()
    }
}

impl ac2_ui::SelectionSource for ClientPlayer<'_> {
    fn snapshot(&self) -> Option<ac2_ui::SelectionSnapshot> {
        let player = self.frame.player.as_ref()?.id;
        let client = self.client.borrow();
        Some(ac2_ui::SelectionSnapshot {
            player,
            selection: self.frame.selection.as_ref().map(|s| s.id),
            last_selection: client.session().player_script().last_selection(),
            last_attacker: client.session().progress().selection.last_attacker,
        })
    }

    fn auto_target(&self) -> bool {
        self.client.borrow_mut().auto_target()
    }

    fn radar_range(&self) -> f32 {
        self.frame.radar.range
    }

    fn radar_blips(&self) -> &[ac2_ui::RadarBlip] {
        &self.frame.radar.blips
    }

    fn object(&self, id: u64) -> Option<ac2_ui::SelectionObject> {
        let client = self.client.borrow_mut();
        let objects = client.session().objects();
        let qualities = &objects.get(id)?.qualities;
        Some(ac2_ui::SelectionObject {
            kind: qualities.inq_int(0x14).unwrap_or(0) as u32,
            selectable: qualities.inq_bool(0x270).unwrap_or(0) != 0,
            owner: qualities.inq_instance_id(0x1F9),
        })
    }

    fn is_fellow(&self, id: u64) -> bool {
        self.frame
            .progress
            .fellowship
            .as_ref()
            .is_some_and(|f| f.fellows.iter().any(|(m, _)| *m == id))
    }
}

impl SkillSource for ClientPlayer<'_> {
    fn panel(&self, panel: u32) -> Option<SkillPanel> {
        let data = self.client.borrow_mut().skill_panel(panel).ok()??;
        let nodes = data
            .nodes
            .iter()
            .map(|(_, node)| SkillNode {
                skill: node.skill_enum,
                kind: node.skill_type,
                name: self.text(&node.name),
                icon: node.icon,
                parents: node.parents.clone(),
                dependencies: node.dependencies.clone(),
                x: node.x,
                y: node.y,
                passive: node.is_passive != 0,
                untrainable: node.untrainable != 0,
            })
            .collect();
        Some(SkillPanel {
            name: self.text(&data.name),
            icon: data.icon,
            nodes,
        })
    }

    fn root_panel_from_race(&self, root: u32) -> Option<u32> {
        self.client
            .borrow_mut()
            .root_panel_from_race(root)
            .ok()
            .flatten()
    }

    fn status(&self, skill: u32) -> SkillStatus {
        let mut client = self.client.borrow_mut();
        // A query that fails reads 0, as the retail wrappers' return slots start.
        let level = client.skill_level(skill).ok();
        SkillStatus {
            trained: client.is_skill_trained(skill).unwrap_or(false),
            untraining: client.is_skill_being_untrained(skill).unwrap_or(false),
            hero: client.is_hero_skill(skill).unwrap_or(false),
            can_train: client.can_skill_be_trained(skill).unwrap_or(0),
            can_raise: client.can_skill_be_raised(skill).unwrap_or(0),
            can_untrain: client.can_skill_be_untrained(skill).unwrap_or(0),
            level: level.map_or(0, |l| l.raw),
            raw_level: level.map_or(0, |l| l.enchanted),
            cost_to_train: u64::from(client.skill_cost_to_train(skill).unwrap_or(0)),
            xp_to_level: client.skill_experience_to_level(skill).unwrap_or(0),
            skill_xp: client.skill_xp(skill).unwrap_or(0),
        }
    }

    /// The available experience (the qualities), then the skill credits and hero credits
    /// (`gmCWInterface::GetSkillCredits` and `GetHeroSkillCredits`).
    fn available(&self) -> (u64, u32, u32) {
        let mut client = self.client.borrow_mut();
        (
            self.frame.progress.experience.available_xp,
            client.skill_credits().unwrap_or(0),
            client.hero_skill_credits().unwrap_or(0),
        )
    }

    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        self.render_string(&ac2_ui::UiString::ui(string_id, vars.to_vec()))
    }
}

impl ShortcutSource for ClientPlayer<'_> {
    fn shortcuts(&self) -> Vec<Shortcut> {
        self.client
            .borrow_mut()
            .shortcut_array()
            .map(|all| all.iter().map(ac2_ui::shortcut_from_info).collect())
            .unwrap_or_default()
    }

    fn icon(&self, index: usize, _: &Shortcut) -> u32 {
        self.client
            .borrow_mut()
            .shortcut_info(index as u32)
            .map_or(0, |info| info.icon)
    }

    fn options(&self) -> ShortcutOptions {
        let settings = self.client.borrow_mut().shortcut_settings().ok();
        settings.map_or(
            ShortcutOptions {
                which_set: 1,
                rows: 1,
                ..ShortcutOptions::default()
            },
            |s| ShortcutOptions {
                which_set: s.which_set,
                show_numbers: s.show_numbers,
                insta_scroll: s.insta_scroll,
                locked: s.locked,
                rows: s.height as i32,
            },
        )
    }

    fn skill_trained(&self, skill: u32) -> bool {
        self.client
            .borrow_mut()
            .is_skill_trained(skill)
            .unwrap_or(false)
    }

    /// `gmCWInterface::InqSkillTimeLeft`: the skill's total and time left, while it is on reset.
    fn skill_reset(&self, skill: u32) -> Option<(f64, f64)> {
        let time = self.client.borrow_mut().skill_time_left(skill).ok()?;
        time.ok
            .then_some((f64::from(time.total), f64::from(time.left)))
    }

    /// `gmCWInterface::IsSkillUsableBasedOnInventoryByEnum`.
    fn skill_usable(&self, skill: u32) -> bool {
        self.client
            .borrow_mut()
            .is_skill_usable_by_inventory(skill)
            .unwrap_or(true)
    }

    /// `gmCWInterface::DoIHaveItem`: in a pack the player lists or equipped.
    fn has_item(&self, item: u64) -> bool {
        self.carries(item)
    }
}

impl ClientPlayer<'_> {
    /// `gmCWInterface::DoIHaveItem`: in a pack the player lists or equipped. The player's script
    /// answers it, or the frame's mirror when the script says no: the script does not yet follow
    /// every move into a side pack (see the inventory test's Known Gaps), so either can know it.
    pub(crate) fn carries(&self, item: u64) -> bool {
        if self.client.borrow_mut().do_i_have_item(item) == Some(true) {
            return true;
        }
        let inventory = &self.frame.inventory;
        inventory.pack.items.contains(&item)
            || inventory.pack.packs.contains(&item)
            || inventory
                .containers
                .iter()
                .any(|c| c.items.contains(&item) || c.packs.contains(&item))
            || inventory.equipped.iter().any(|(_, i)| *i == item)
    }

    /// The items the player carries (`gmCWInterface::InqAllInventoryIIDList`): the script's list
    /// with the frame's items it lacks.
    pub(crate) fn carried_ids(&self) -> Vec<u64> {
        let mut items = self.client.borrow_mut().inventory_ids().unwrap_or_default();
        let inventory = &self.frame.inventory;
        let frame_items = inventory
            .pack
            .items
            .iter()
            .chain(inventory.containers.iter().flat_map(|c| c.items.iter()))
            .copied();
        for item in frame_items {
            if !items.contains(&item) {
                items.push(item);
            }
        }
        items
    }

    /// The worn items as `(location, item)` (`gmCWInterface::InqSimpleInvByLocTable`), the
    /// script's answer, else the frame's.
    pub(crate) fn equipped(&self) -> Vec<(u32, u64)> {
        if let Some(worn) = self.client.borrow_mut().equipped_by_location() {
            return worn;
        }
        self.frame.inventory.equipped.to_vec()
    }
}

impl ClientPlayer<'_> {
    /// `gmNavigationUI::AddMarker`'s tooltip (see `UiText::marker_tooltip`).
    pub(crate) fn marker_tooltip(
        &self,
        message: &ac2_net::protocol::StringInfo,
        cell: u32,
    ) -> String {
        self.ui().marker_tooltip(message, cell)
    }

    /// `ID_Inventory_TT_Durability` (see `UiText::durability_tooltip`).
    pub(crate) fn durability_tooltip(&self, current: i32, max: i32) -> Option<String> {
        self.ui().durability_tooltip(current, max)
    }

    /// `gmExaminationUI::MakeString` and `ProcessNode` (see `UiText::examination_items`).
    pub(crate) fn examination_items(
        &self,
        profile: &ac2_net::protocol::ExaminationProfile,
    ) -> Vec<ac2_ui::ExaminationItem> {
        self.ui().examination_items(profile)
    }
}
