//! `gmAllegianceUI`: the social panel's allegiance page. It shows one level of the hierarchy at a
//! time, around a target (the player at first): the allegiance's name, size and monarch, the
//! target with its patron (`MemberElement`), its vassals (`VassalElement`, or `NoVassalsElement`)
//! and the message of the day, in `MemberListBox`. Home, Up and a vassal's Down button move the
//! target (`FocusTree`), which asks the server for that member's profile (`SendRequest`; the page
//! is `pending` until it comes). Swear swears to the selected player; Break breaks with the target
//! after a confirmation; a would-be vassal's request is a confirmation too. While the page is not
//! shown the profile is asked for nobody (`UpdateShowState`).
//!
//! The panel turns these into [`AllegianceRequest`]s for the client
//! (`gmCWInterface::QueryAllegianceProfile`, `SwearAllegiance`, `BreakAllegiance`, `AcceptVassal`,
//! `DeclineVassal`).
//!
//! Not modelled yet: tooltips. The page follows its tab and field once a frame rather than on
//! their `opened` / `closed` / `set_show_state` messages.

use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_weenie::{AllegianceMember, AllegianceProfile};

use crate::dialog::{DialogFactory, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::panel::PAGE_OPEN;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, STATELESS};
use crate::ui_string::{UiString, UiStrings};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `INVALID_IID`.
const INVALID: u64 = 0;

/// What the page asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllegianceRequest {
    /// `QueryAllegianceProfile(target)` (`INVALID_IID` while the page is hidden).
    Query(u64),
    /// `SwearAllegiance` to the selection.
    Swear,
    /// `BreakAllegiance(target)`.
    Break(u64),
    /// `AcceptVassal`; `DeclineVassal(0x1585)`.
    AcceptVassal,
    DeclineVassal,
}

/// What the page reads from the player.
pub trait AllegianceSource: UiStrings {
    fn player(&self) -> u64;
    /// A string the server sent, rendered.
    fn render_wire(&self, info: &StringInfo) -> String;
}

/// `IIDUtils::VerifyPlayer`: the id's type byte marks a player.
fn is_player(iid: u64) -> bool {
    let kind = (iid >> 56) as u8;
    kind & 0x80 == 0 && kind & 0x40 == 0 && kind & 0x20 != 0
}

/// `gmAllegianceUI`'s state.
#[derive(Debug, Clone)]
pub struct AllegianceUi {
    layout: Arc<LayoutDesc>,
    /// `_allegianceHierachyPage` (the `AllegiancePage`).
    page: Option<usize>,
    swear: Option<usize>,
    breaker: Option<usize>,
    player: u64,
    cur_target: u64,
    cur_patron: u64,
    patron: u64,
    monarch: u64,
    selection: u64,
    cur_name: StringInfo,
    officers: Vec<u64>,
    in_allegiance: bool,
    player_is_vassal_of_target: bool,
    accept_swear_context: u32,
    break_context: u32,
    shown: Option<bool>,
    /// The vassal rows' Down buttons and their ids (retail's `iid` attribute).
    downs: Vec<(u64, usize)>,
    requests: Vec<AllegianceRequest>,
}

impl AllegianceUi {
    /// `gmAllegianceUI::gmAllegianceUI`: the page and its Swear and Break buttons; the target is
    /// the player.
    pub fn create(tree: &UiTree, layout: &Arc<LayoutDesc>, player: u64) -> Self {
        let page = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("AllegiancePage")));
        let child = |name: &str| page.and_then(|p| tree.child_recursive(p, id(name)));
        Self {
            layout: Arc::clone(layout),
            page,
            swear: child("SwearButton"),
            breaker: child("BreakButton"),
            player,
            cur_target: player,
            cur_patron: INVALID,
            patron: INVALID,
            monarch: INVALID,
            selection: INVALID,
            cur_name: StringInfo::default(),
            officers: Vec::new(),
            in_allegiance: false,
            player_is_vassal_of_target: false,
            accept_swear_context: 0,
            break_context: 0,
            shown: None,
            downs: Vec::new(),
            requests: Vec::new(),
        }
    }

    /// The player is in the world (`m_initData.m_iid`): the target, then `UpdateShowState`.
    pub fn set_player(&mut self, tree: &mut UiTree, player: u64) {
        self.player = player;
        self.cur_target = player;
        self.shown = None;
        self.follow_show_state(tree);
    }

    pub fn take_requests(&mut self) -> Vec<AllegianceRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The member list's rows.
    pub fn rows(&self, tree: &UiTree) -> Vec<usize> {
        self.page_child(tree, "MemberListBox")
            .map(|list| tree.list_items(list).to_vec())
            .unwrap_or_default()
    }

    fn page_child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        self.page.and_then(|p| tree.child_recursive(p, id(name)))
    }

    /// `gmAllegianceUI::SendRequest`.
    fn send_request(&mut self, tree: &mut UiTree, target: u64) {
        self.requests.push(AllegianceRequest::Query(target));
        if let (true, Some(page)) = (target != INVALID, self.page) {
            tree.set_state(page, id("pending"));
        }
    }

    /// `gmAllegianceUI::UpdateButtons`.
    fn update_buttons(&self, tree: &mut UiTree) {
        let (Some(swear), Some(breaker)) = (self.swear, self.breaker) else {
            return;
        };
        let no_break = self.cur_target == self.player
            || self.break_context != 0
            || self.accept_swear_context != 0
            || self.cur_target == INVALID
            || !self.in_allegiance
            || (self.cur_target != self.patron
                && self.cur_patron != self.player
                && !self.officers.contains(&self.player));
        let can_swear = self.break_context == 0
            && self.accept_swear_context == 0
            && self.patron == INVALID
            && self.selection != INVALID
            && self.selection != self.player
            && is_player(self.selection);
        tree.set_bool(swear, BUTTON_GHOSTED, !can_swear);
        tree.set_bool(breaker, BUTTON_GHOSTED, no_break);
    }

    /// `gmAllegianceUI::FocusTree`.
    fn focus_tree(&mut self, tree: &mut UiTree, target: u64) {
        if self.break_context == 0 {
            self.send_request(tree, target);
            self.update_buttons(tree);
        }
    }

    /// `gmAllegianceUI::UpdateShowState`, as the allegiance tab opens or the social field shows
    /// or hides: the target's profile while shown, nobody's otherwise.
    pub fn follow_show_state(&mut self, tree: &mut UiTree) {
        let Some(page) = self.page else {
            return;
        };
        let tab_open = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("AllegianceTab")))
            .and_then(|t| tree.get(t))
            .is_some_and(|t| t.state() == PAGE_OPEN);
        let field_shown = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("SocialField")))
            .and_then(|f| tree.get(f))
            .is_some_and(|f| f.visible);
        let shown = tab_open && field_shown && tree.get(page).is_some();
        if self.shown == Some(shown) {
            return;
        }
        self.shown = Some(shown);
        let target = if shown { self.cur_target } else { INVALID };
        self.send_request(tree, target);
        self.update_buttons(tree);
    }

    /// `RecvNotice_SelectionChanged`.
    pub fn selection_changed(&mut self, tree: &mut UiTree, selection: u64) {
        self.selection = selection;
        self.update_buttons(tree);
    }

    fn template(&self, tree: &mut UiTree, name: &str) -> Option<usize> {
        let desc = self.layout.element(id(name))?.clone();
        tree.create_element(Arc::clone(&self.layout), desc, false)
    }

    /// `AllegianceDataAdaptor::GetSpeciesGenderString`: table `0x4100000B`'s race and sex, or
    /// an empty literal.
    fn species_gender(member: &AllegianceMember) -> UiString {
        let name = match (member.gender, member.species) {
            (0, _) => None,
            (gender, 1) => Some(if gender != 0x1000 {
                "ID_RaceAndSex_HumanFemale"
            } else {
                "ID_RaceAndSex_HumanMale"
            }),
            (gender, 2) => Some(if gender != 0x1000 {
                "ID_RaceAndSex_LugianFemale"
            } else {
                "ID_RaceAndSex_LugianMale"
            }),
            (gender, 4) => Some(if gender != 0x1000 {
                "ID_RaceAndSex_TumerokFemale"
            } else {
                "ID_RaceAndSex_TumerokMale"
            }),
            _ => None,
        };
        let mut string = UiString::new(name.map_or(0, id), 0x4100_000B, Vec::new());
        if name.is_none() {
            string.literal = Some(String::new());
        }
        string
    }

    /// `gmAllegianceUI::MakeCurrent`: the target's row with its patron.
    fn make_current(
        &mut self,
        tree: &mut UiTree,
        s: &dyn AllegianceSource,
        p: &AllegianceProfile,
        row: usize,
    ) {
        let current = &p.member;
        let Some(name_text) = tree.child_recursive(row, id("MemberNameText")) else {
            return;
        };
        // One StringInfo throughout, as retail reuses it.
        let mut vars = vec![
            (id("title"), SkillVar::Wire(current.title.clone())),
            (id("value"), SkillVar::Wire(current.name.clone())),
            (id("level"), SkillVar::UInt(current.level.into())),
            (
                id("speciesgender"),
                SkillVar::Info(Box::new(Self::species_gender(current))),
            ),
        ];
        let string_id = id(if current.level == 0 {
            "ID_Allegiance_TitleAndName"
        } else {
            "ID_Allegiance_TitleAndNameLevelSpeciesGender"
        });
        let state = if current.id == self.player {
            "self"
        } else if p.member_online {
            "activememberonline"
        } else {
            "activememberoffline"
        };
        tree.set_state(name_text, id(state));
        tree.set_text(
            name_text,
            &s.render_string(&UiString::ui(string_id, vars.clone())),
        );
        let Some(misc) = tree.child_recursive(row, id("MemberMiscText")) else {
            return;
        };
        let string_id = if p.patron.id == INVALID {
            id("ID_Allegiance_NoPatron")
        } else {
            vars[1] = (id("value"), SkillVar::Wire(p.patron.name.clone()));
            id("ID_Allegiance_PatronName")
        };
        let state = if p.patron.id == self.player {
            id("self")
        } else {
            STATELESS
        };
        tree.set_state(misc, state);
        tree.set_text(misc, &s.render_string(&UiString::ui(string_id, vars)));
        let Some(home) = tree.child_recursive(row, id("HomeButton")) else {
            return;
        };
        self.cur_patron = p.patron.id;
        let at_home = current.id == self.player;
        if at_home {
            self.patron = self.cur_patron;
        }
        tree.set_bool(home, BUTTON_GHOSTED, at_home);
        if let Some(up) = tree.child_recursive(row, id("UpButton")) {
            tree.set_bool(up, BUTTON_GHOSTED, self.cur_patron == INVALID);
        }
    }

    /// `gmAllegianceUI::MakeVassal`: a vassal's row; its Down button carries its id.
    fn make_vassal(
        &mut self,
        tree: &mut UiTree,
        s: &dyn AllegianceSource,
        vassal: &AllegianceMember,
        online: bool,
        row: usize,
    ) -> Option<usize> {
        let name_text = tree.child_recursive(row, id("MemberNameText"))?;
        let mut vars = vec![
            (id("title"), SkillVar::Wire(vassal.title.clone())),
            (id("value"), SkillVar::Wire(vassal.name.clone())),
        ];
        tree.set_text(
            name_text,
            &s.render_string(&UiString::ui(
                id("ID_Allegiance_TitleAndName"),
                vars.clone(),
            )),
        );
        let state = if vassal.id == self.player {
            self.player_is_vassal_of_target = true;
            "self"
        } else if online {
            "online"
        } else {
            "offline"
        };
        tree.set_state(name_text, id(state));
        let down = tree.child_recursive(row, id("DownButton"))?;
        let misc = tree.child_recursive(row, id("MemberMiscText"))?;
        vars[1] = (id("value"), SkillVar::UInt(vassal.xp_passed));
        vars.push((id("level"), SkillVar::UInt(vassal.level.into())));
        vars.push((
            id("speciesgender"),
            SkillVar::Info(Box::new(Self::species_gender(vassal))),
        ));
        let string_id = id(if vassal.level == 0 {
            "ID_Allegiance_XpUp"
        } else {
            "ID_Allegiance_VassalLevelSpeciesGenderXp"
        });
        tree.set_text(misc, &s.render_string(&UiString::ui(string_id, vars)));
        Some(down)
    }

    /// `RecvNotice_UpdateAllegianceProfile` (`gmAllegianceUI::Update`, then the show state:
    /// a hidden page asks for nobody's profile again).
    pub fn update_profile(
        &mut self,
        tree: &mut UiTree,
        s: &dyn AllegianceSource,
        p: &AllegianceProfile,
    ) {
        self.downs = self.update(tree, s, p);
        if self.shown != Some(true) {
            self.send_request(tree, INVALID);
        }
    }

    /// `gmAllegianceUI::Update`; the vassals' Down buttons and ids.
    fn update(
        &mut self,
        tree: &mut UiTree,
        s: &dyn AllegianceSource,
        p: &AllegianceProfile,
    ) -> Vec<(u64, usize)> {
        let mut downs = Vec::new();
        let Some(page) = self.page else {
            return downs;
        };
        self.cur_patron = INVALID;
        self.cur_name = StringInfo::default();
        tree.set_state(page, STATELESS);
        self.officers.clear();
        if p.member.id == INVALID {
            self.in_allegiance = false;
            self.patron = INVALID;
            tree.set_state(page, id("noallegiance"));
            self.update_buttons(tree);
            return downs;
        }
        self.officers = p.officers.clone();
        tree.set_state(page, id("pending"));
        self.in_allegiance = true;
        self.cur_target = p.member.id;
        self.cur_name = p.member.name.clone();
        tree.set_state(page, id("allegiance"));
        // `AllegianceProfileAdaptor::InqRealMonarch`: the monarch, else the patron's, else the
        // member's slot.
        self.monarch = p.monarch.id;
        let monarch = if p.monarch.id != INVALID {
            &p.monarch
        } else if p.patron.id != INVALID {
            &p.patron
        } else {
            &p.member
        };
        let Some(name) = self.page_child(tree, "AllegianceName") else {
            return downs;
        };
        tree.set_text(name, &s.render_wire(&p.name));
        let size = UiString::ui(
            id("ID_Allegiance_Size"),
            vec![(id("value"), SkillVar::UInt(p.total.into()))],
        );
        let Some(size_text) = self.page_child(tree, "AllegianceSize") else {
            return downs;
        };
        tree.set_text(size_text, &s.render_string(&size));
        let Some(monarch_text) = self.page_child(tree, "MonarchName") else {
            return downs;
        };
        // Retail adds the name to the size's StringInfo.
        let mut monarch_string = size;
        monarch_string.string_id = id("ID_Allegiance_MonarchName");
        monarch_string.vars[0] = (id("value"), SkillVar::Wire(monarch.name.clone()));
        let state = if self.monarch == self.player {
            id("self")
        } else {
            STATELESS
        };
        tree.set_state(monarch_text, state);
        tree.set_text(monarch_text, &s.render_string(&monarch_string));
        let Some(list) = self.page_child(tree, "MemberListBox") else {
            return downs;
        };
        tree.list_multi_op_start(list);
        tree.list_flush(list);
        let Some(row) = self.template(tree, "MemberElement") else {
            return downs;
        };
        self.make_current(tree, s, p, row);
        tree.list_add_item(list, row);
        self.player_is_vassal_of_target = false;
        if p.vassals.is_empty() {
            if let Some(row) = self.template(tree, "NoVassalsElement") {
                tree.list_add_item(list, row);
            }
        } else {
            let mut last = None;
            for (vassal, online) in &p.vassals {
                let Some(row) = self.template(tree, "VassalElement") else {
                    continue;
                };
                if let Some(down) = self.make_vassal(tree, s, vassal, *online, row) {
                    downs.push((vassal.id, down));
                }
                tree.list_add_item(list, row);
                last = Some(row);
            }
            if let Some(graphic) = last.and_then(|r| tree.child_recursive(r, id("ConnectGraphic")))
            {
                tree.set_state(graphic, id("end"));
            }
        }
        if let Some(motd) = self.template(tree, "MotDTextElement") {
            let valid = p.motd.literal.is_some() || p.motd.string_id != 0;
            let text = if valid {
                UiString::new(
                    id("ID_MotD"),
                    0x4100_0001,
                    vec![(id("MESSAGE"), SkillVar::Wire(p.motd.clone()))],
                )
            } else {
                UiString::new(id("ID_NoMotD"), 0x4100_0001, Vec::new())
            };
            tree.set_text(motd, &s.render_string(&text));
            let width = tree.get(motd).map_or(0, |e| e.rect.width());
            if let Some(height) = tree.text_paper_height(motd) {
                tree.resize_to(motd, width, height);
            }
            tree.list_add_item(list, motd);
        }
        tree.list_multi_op_complete(list);
        tree.list_home(list);
        self.update_buttons(tree);
        downs
    }

    /// `RecvNotice_RequestVassal` (`MakeAcceptSwearDialog`).
    pub fn request_vassal(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn AllegianceSource,
        vassal: &StringInfo,
    ) {
        if self.accept_swear_context != 0 {
            return;
        }
        let name = s.render_wire(vassal);
        let text = UiString::ui(
            id("ID_Allegiance_SwearText"),
            vec![(id("value"), SkillVar::Text(name))],
        );
        self.accept_swear_context =
            dialogs.make_dialog(tree, &s.render_string(&text), DIALOG_CONFIRMATION, false);
        self.update_buttons(tree);
    }

    /// `RecvNotice_Patron_SwearDone`.
    pub fn patron_swear_done(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        dialogs.close_dialog(tree, self.accept_swear_context);
        self.accept_swear_context = 0;
        self.update_buttons(tree);
    }

    /// `gmAllegianceUI::MakeBreakDialog`.
    fn make_break_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn AllegianceSource,
    ) {
        if self.break_context != 0 {
            return;
        }
        let name = s.render_wire(&self.cur_name);
        let text = UiString::ui(
            id("ID_Allegiance_BreakText"),
            vec![(id("value"), SkillVar::Text(name))],
        );
        self.break_context =
            dialogs.make_dialog(tree, &s.render_string(&text), DIALOG_CONFIRMATION, false);
        self.update_buttons(tree);
    }

    /// `RecvNotice_CloseConfirmationDialog`: the vassal's request answered, or `DoBreak`.
    pub fn close_confirmation(&mut self, tree: &mut UiTree, context: u32, yes: bool) {
        if context != 0 && context == self.accept_swear_context {
            self.accept_swear_context = 0;
            self.update_buttons(tree);
            self.requests.push(if yes {
                AllegianceRequest::AcceptVassal
            } else {
                AllegianceRequest::DeclineVassal
            });
        }
        if context != 0 && context == self.break_context {
            // `DoBreak`.
            self.break_context = 0;
            self.update_buttons(tree);
            if yes {
                self.requests
                    .push(AllegianceRequest::Break(self.cur_target));
                let target = if self.player_is_vassal_of_target {
                    self.player
                } else {
                    self.cur_patron
                };
                self.send_request(tree, target);
                self.update_buttons(tree);
            }
        }
    }

    /// `gmAllegianceUI::ListenToMessage`.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn AllegianceSource,
        message: &UiMessage,
    ) {
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        match message.element_id {
            e if e == id("SwearButton") => self.requests.push(AllegianceRequest::Swear),
            e if e == id("BreakButton") => self.make_break_dialog(tree, dialogs, s),
            e if e == id("HomeButton") => self.focus_tree(tree, self.player),
            e if e == id("UpButton") => self.focus_tree(tree, self.cur_patron),
            e if e == id("DownButton") => {
                let target = self.downs.iter().find(|(_, d)| Some(*d) == message.element);
                if let Some(&(target, _)) = target {
                    self.focus_tree(tree, target);
                }
            }
            _ => {}
        }
    }
}
