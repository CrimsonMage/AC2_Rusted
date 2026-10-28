//! The gameplay HUD's panel state, owned by [`Hud`]: the world overlays `gm3DTextUI` keeps (the
//! targeting reticle, the name plates and the floating damage texts) and the clock they run on.
//! The client owns the element tree and the GPU surfaces; this state takes the tree as an argument
//! and reads no client type, so the overlays' logic runs on plain frame data.

use ac2_net::protocol::SelectionInfo;
use ac2_world::GameDateTime;

use crate::assistance::AssistancePanel;
use crate::damage_text::DamageTexts;
use crate::element::UiTree;
use crate::name_plates::NamePlates;
use crate::reticle::{Reticle, ReticleOutcome, ReticleTarget};
use crate::skill_tree::SkillTreeUi;
use crate::vitals::VitalsUi;
use crate::{
    AllegianceUi, BackpackUi, BookUi, CatalogUi, CharacterUi, ChatOptionsUi, ConsignmentUi,
    CraftUi, EffectsUi, ExaminationUi, ExternalContainerUi, FellowshipUi, FriendsPanel, HelpPages,
    HelpWindow, IngredientUi, Items3dUi, MiniFellowUi, MoneyBagUi, NavigationUi, NetStatusUi,
    NewSaleUi, OptionsPages, OverviewMapUi, PaperDollUi, PetUi, PkStatusUi, QuestsUi, RadarUi,
    SecureTradeUi, ShopUi, SocialPanel, StackUi, StoryView, VitaeStatusUi,
};

/// The HUD's panel state (`gmGamePlayUI`'s children that the client drives each frame).
#[derive(Debug, Default)]
pub struct Hud {
    reticle: Option<Reticle>,
    pub names: Option<NamePlates>,
    pub damage: Option<DamageTexts>,
    /// The HUD's clock (`Timer::cur_time`) at the last frame: a damage text starts at it.
    pub now: f64,
    /// The selection info the reticle's meters last took.
    reticle_info: Option<SelectionInfo>,
    /// `gmSkillUI`'s `gmSkillTreeUI`.
    pub skills: Option<SkillTreeUi>,
    /// `gmCraftUI`, over `CraftPage`.
    pub craft: Option<CraftUi>,
    /// `gmIngredientUI`: the ingredient panel of a recipe.
    pub ingredient: Option<IngredientUi>,
    pub examination: Option<ExaminationUi>,
    pub allegiance: Option<AllegianceUi>,
    pub fellowship: Option<FellowshipUi>,
    /// `gmFriendsUI`: the friends page of the social toolbar.
    pub friends: Option<FriendsPanel>,
    /// `gmAssistanceUI`: the assistance page of the option toolbar.
    pub assistance: Option<AssistancePanel>,
    /// `gmSocialUI`: the social panel's buttons.
    pub social: Option<SocialPanel>,
    /// `gmMiniFellowUI`: the fellowship mini panel on the HUD.
    pub mini_fellow: Option<MiniFellowUi>,
    /// `gmPetUI`: the pet panel on the HUD.
    pub pets: Option<PetUi>,
    /// `gmQuestsUI`: the Quest panel and its quest journal.
    pub quests: Option<QuestsUi>,
    /// `gmStatusbarUI`'s `gmVitalsUI` and `gmEffectsUI`.
    pub vitals: Option<VitalsUi>,
    pub effects: Option<EffectsUi>,
    pub vitae: Option<VitaeStatusUi>,
    /// `gmPKStatusUI`, the PK indicator.
    pub pk: Option<PkStatusUi>,
    /// `gmNetStatusUI`, the net indicator.
    pub net: Option<NetStatusUi>,
    /// `gmInventoryUI`'s backpack, made once the player is in the world.
    pub backpack: Option<BackpackUi>,
    /// `gmInventoryUI`'s paper doll, made with the backpack.
    pub paper_doll: Option<PaperDollUi>,
    /// `gm3DItemsUI`, made with the backpack.
    pub items_3d: Option<Items3dUi>,
    /// `gmMoneyBagUI`, made with the backpack.
    pub money: Option<MoneyBagUi>,
    /// `gmStackUI`.
    pub stack: Option<StackUi>,
    /// `gmExternalContainerUI`.
    pub external_container: Option<ExternalContainerUi>,
    /// `gmSecureTradeUI`, hidden until a trade opens (its requests wait in the panel).
    pub secure_trade: Option<SecureTradeUi>,
    /// `gmCharacterUI`, the character options page (its requests wait in the page).
    pub character: Option<CharacterUi>,
    /// `gmChatOptionsUI`, the chat options page (its requests wait in the page).
    pub chat_options: Option<ChatOptionsUi>,
    /// `gmClientUI`, the client options page, and the effects its applies left.
    pub options: OptionsPages,
    /// `gmShopUI`, the vendor's panel (its requests wait in the panel).
    pub shop: Option<ShopUi>,
    /// `gmShopCatalogUI`, the storekeeper's catalog (its requests wait in the panel).
    pub catalog: Option<CatalogUi>,
    /// `gmShopNewSaleUI`, the consignment's new-sale dialog.
    pub new_sale: Option<NewSaleUi>,
    /// `gmShopConsignmentUI`, the storekeeper's consignment.
    pub consignment: Option<ConsignmentUi>,
    /// `gmRadarUI`'s blips, painted over the radar field each compose.
    pub radar: Option<RadarUi>,
    /// The player's cell as the radar shows it (rendered by the frameworks, which have the string
    /// tables).
    pub radar_cell: Option<u32>,
    /// The player's map coordinates as the radar shows them.
    pub radar_coordinates: Option<String>,
    /// The blips last uploaded over the radar field's layer. The field is composed and uploaded
    /// again only when these change (or the whole HUD is composed).
    pub radar_uploaded: Option<Vec<(i32, i32, [u8; 4])>>,
    /// The navigation panel's game date and time, rendered for the calendar they were rendered
    /// for (`GameTime::InqCurrentGameDateTimeString`).
    pub navigation_time: Option<(GameDateTime, String, String)>,
    /// `gmNavigationUI::AddMarker`'s tooltips, one per frame marker (rendered on the frame).
    pub navigation_tooltips: Vec<String>,
    /// `gmNavigationUI`'s macro map (its map image is set on the worker) and detail view.
    pub navigation: Option<NavigationUi>,
    /// `gmOverviewMapUI`.
    pub overview: Option<OverviewMapUi>,
    /// `gmBookUI`, shown by the book's use notice.
    pub book: Option<BookUi>,
    /// The help window (`KeyStone::OpenHelp`), hidden until a help page opens.
    pub help: Option<HelpWindow>,
    /// The help pages, read once on the HUD's worker.
    pub help_pages: Option<HelpPages>,
    /// `gmStoryUI`'s background and story fields, shown while a story holds the slot.
    pub story: Option<StoryView>,
}

impl Hud {
    /// The overlays the HUD's build made (`gm3DTextUI`'s reticle, name plates and damage texts).
    pub fn new(
        reticle: Option<Reticle>,
        names: Option<NamePlates>,
        damage: Option<DamageTexts>,
    ) -> Self {
        Self {
            reticle,
            names,
            damage,
            ..Self::default()
        }
    }

    /// The HUD's clock for this frame (`Timer::cur_time`, seconds).
    pub fn set_now(&mut self, now: f64) {
        self.now = now;
    }

    /// The HUD's clock (`Timer::cur_time`, seconds) as the last frame set it.
    pub fn now(&self) -> f64 {
        self.now
    }

    /// The name plates' slot, for the passes that take it (`MessagePanels::names`).
    pub fn names_slot(&mut self) -> &mut Option<NamePlates> {
        &mut self.names
    }

    /// The damage texts' slot, for the notice pass (`NoticePanels::damage`).
    pub fn damage_slot(&mut self) -> &mut Option<DamageTexts> {
        &mut self.damage
    }

    /// The targeting reticle, while the HUD has one.
    pub fn reticle_mut(&mut self) -> Option<&mut Reticle> {
        self.reticle.as_mut()
    }

    /// The name plates, while the HUD has them.
    pub fn names_mut(&mut self) -> Option<&mut NamePlates> {
        self.names.as_mut()
    }

    /// The floating damage texts, while the HUD has them.
    pub fn damage_mut(&mut self) -> Option<&mut DamageTexts> {
        self.damage.as_mut()
    }

    /// The objects with a name plate.
    pub fn named(&self) -> Vec<u64> {
        self.names.as_ref().map_or(Vec::new(), |n| n.named())
    }

    /// The objects with a floating damage text in the air.
    pub fn damage_objects(&self) -> Vec<u64> {
        self.damage.as_ref().map_or(Vec::new(), |d| d.objects())
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it is
    /// drawn, its box and projection. `info` is taken once per new selection info
    /// (`RecvNotice_UpdateSelectionInfo`).
    pub fn update_reticle(
        &mut self,
        tree: &mut UiTree,
        selection: Option<(u64, Option<&str>)>,
        info: Option<&SelectionInfo>,
        target: Option<ReticleTarget<'_>>,
    ) -> ReticleOutcome {
        let Some(reticle) = self.reticle.as_mut() else {
            return ReticleOutcome::default();
        };
        let id = selection.map(|(id, _)| id);
        if reticle.current() != id {
            reticle.selection_changed(tree, id, selection.and_then(|(_, name)| name));
            self.reticle_info = None;
        }
        if let Some(info) = info.filter(|i| self.reticle_info.as_ref() != Some(i)) {
            reticle.selection_info(tree, info);
            self.reticle_info = Some(*info);
        }
        reticle.update(tree, target, false)
    }

    /// `gm3DTextUI::UpdateDamage`: the damage texts over `places` at the HUD's clock, rising above
    /// the selection's reticle and name when `selection` is the object they are over.
    pub fn update_damage(
        &mut self,
        tree: &mut UiTree,
        places: &[crate::DamagePlace<'_>],
        selection: u64,
    ) {
        let (Some(damage), reticle) = (self.damage.as_mut(), self.reticle.as_ref()) else {
            return;
        };
        let offset = reticle.map_or(0, |r| r.selection_offset(tree));
        let precluded = reticle.map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        damage.update(
            tree,
            places,
            (selection, offset),
            precluded,
            self.now,
            false,
        );
    }

    /// `gm3DTextUI::UpdateNames` after the reticle: the plates over `places`, clear of the
    /// selection's reticle, within `range` (the radar's, squared as a float).
    pub fn update_names(
        &mut self,
        tree: &mut UiTree,
        places: &[crate::NamePlace<'_>],
        selection: u64,
        range: f32,
    ) {
        let Some(names) = self.names.as_mut() else {
            return;
        };
        let reticle = self.reticle.as_ref();
        let rect = reticle.and_then(|r| r.selection_rect());
        let precluded = reticle.map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        names.update(
            tree,
            places,
            selection,
            rect,
            range * range,
            precluded,
            false,
        );
    }
}
