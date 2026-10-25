//! A native stand-in for the player's weenie scripts (`CPlayer`, `CExaminationSystem` in the
//! Legions script library): the selection and examination state those scripts keep, the
//! messages they send, and the `CM_*::SendNotice_*` notices they raise for the UI.
//!
//! Retail runs these as WSL script; each method names the script function it mirrors
//! (package and function index in WLib 0x56000005) and the IFun the native UI calls it
//! through. Once the script VM runs `CPlayer`, the VM glue raises the same notices and this
//! stand-in is retired.

use std::collections::HashMap;

use ac2_net::protocol::{
    ExaminationProfile, ExaminationRequest, ScriptMessage, ScriptRequest, SelectionInfo,
};

/// "Nothing selected": `Player::GetSelection` returns 0 (`CPlayer::SetSelection` compares
/// against a pushed 0).
pub const NO_SELECTION: u64 = 0;

/// A `CM_*::SendNotice_*` notice, as plain data, in the order the scripts raise them.
#[derive(Debug, Clone, PartialEq)]
pub enum GameplayNotice {
    /// `CM_Character::SendNotice_SelectionChanged`: the new selection (0 for none).
    SelectionChanged(u64),
    /// `CM_Character::SendNotice_UpdateSelectionInfo`: the selected object's health and vigor.
    UpdateSelectionInfo { id: u64, info: SelectionInfo },
    /// `CM_Examination::SendNotice_ExaminationRequestChanged`.
    ExaminationRequestChanged(ExaminationRequest),
    /// `CM_Examination::SendNotice_UpdateExaminationProfile`.
    UpdateExaminationProfile(ExaminationProfile),
}

/// What one script call did: messages for the server, notices for the UI.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScriptOutput {
    pub requests: Vec<ScriptRequest>,
    pub notices: Vec<GameplayNotice>,
}

/// The player's script state.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerScript {
    /// `Player` slot 26 (`GetSelection`, `Player` 0x3D).
    selection: u64,
    /// `CPlayer` slot 39 (`CPlayer` 0x8B stores the previous selection; `GetLastSelection`).
    last_selection: u64,
    /// `CPlayer` slot 29: the examination request (`Player` 0x9 returns it).
    examination: ExaminationRequest,
    /// `CPlayer` slot 56: the examination panel is showing (`StartExamination` /
    /// `StopExamination`).
    examining: bool,
    /// `CPlayer` slots 45 and 58: the latest selection info and the object it described.
    selection_info: Option<(u64, SelectionInfo)>,
    /// `gmCEntity` 0x6 / 0xD: the examination profile each object's weenie keeps.
    profiles: HashMap<u64, ExaminationProfile>,
    /// `GameplayOptionsProfile::GetShowPKDamage` (`CPlayer` 0x1A8).
    show_pk_damage: bool,
}

impl Default for PlayerScript {
    fn default() -> Self {
        Self {
            selection: NO_SELECTION,
            last_selection: NO_SELECTION,
            examination: ExaminationRequest::default(),
            examining: false,
            selection_info: None,
            profiles: HashMap::new(),
            show_pk_damage: false,
        }
    }
}

impl PlayerScript {
    /// `GetSelection`: the selected object, if any.
    pub fn selection(&self) -> Option<u64> {
        (self.selection != NO_SELECTION).then_some(self.selection)
    }

    /// `GetLastSelection`.
    pub fn last_selection(&self) -> Option<u64> {
        (self.last_selection != NO_SELECTION).then_some(self.last_selection)
    }

    /// The selection info the server last sent for the current selection.
    pub fn selection_info(&self) -> Option<(u64, SelectionInfo)> {
        self.selection_info
    }

    /// `Player` 0x9: the current examination request.
    pub fn examination_request(&self) -> ExaminationRequest {
        self.examination
    }

    pub fn is_examining(&self) -> bool {
        self.examining
    }

    /// The gameplay option `CPlayer` 0x1A8 reads (`GameplayOptionsProfile::GetShowPKDamage`).
    pub fn set_show_pk_damage(&mut self, show: bool) {
        self.show_pk_damage = show;
    }

    /// `CPlayer` 0x6A (`SetSelection`, IFun 0xB40002). `combat_target` is
    /// `CCombatSystem` 0x1 for the new selection (Legions examines combat targets with the
    /// extended request type 0x4000000D).
    ///
    /// Not mirrored: `CUsageSystem` 0x3 (a multi-item use collecting its targets takes the
    /// click instead) and `CCombatSystem` 0xD (the combat system's own target).
    pub fn set_selection(&mut self, id: u64, combat_target: bool) -> ScriptOutput {
        let mut out = ScriptOutput::default();
        if self.selection == id {
            return out;
        }
        self.last_selection = self.selection;
        self.selection = id;
        out.requests.push(ScriptRequest::SetTarget(id));
        out.notices.push(GameplayNotice::SelectionChanged(id));
        if id != NO_SELECTION {
            let mut request = ExaminationRequest::object(id);
            if combat_target {
                request.kind = ExaminationRequest::OBJECT_EXTENDED;
            }
            self.set_examination_request(request, &mut out);
        } else if matches!(
            self.examination.kind,
            ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
        ) {
            self.set_examination_request(ExaminationRequest::default(), &mut out);
        }
        out
    }

    /// `CPlayer` 0xE6 (`SetExaminationRequest`, IFun `SetExaminationRequest`).
    pub fn examine(&mut self, request: ExaminationRequest) -> ScriptOutput {
        let mut out = ScriptOutput::default();
        self.set_examination_request(request, &mut out);
        out
    }

    fn set_examination_request(&mut self, mut request: ExaminationRequest, out: &mut ScriptOutput) {
        if request == self.examination {
            return;
        }
        if self.examining {
            self.stop_examining_object(out);
        }
        // The Legions word at +0x28 carries over (EFuns 0x9A0014 get, 0x9A0013 set).
        request.extra[2] = self.examination.extra[2];
        self.examination = request;
        out.notices
            .push(GameplayNotice::ExaminationRequestChanged(request));
        if self.examining {
            self.start_examining(out);
        }
    }

    /// `CPlayer` 0x114 (`StartExamination`, the examination panel opened): the panel's word
    /// goes into the request (EFun 0x9A0013), then `CExaminationSystem` 0xE.
    pub fn start_examination(&mut self, context: u32) -> ScriptOutput {
        let mut out = ScriptOutput::default();
        self.examination.extra[2] = context;
        self.start_examining(&mut out);
        self.examining = true;
        out
    }

    /// `CPlayer` 0xF6 (`StopExamination`, the panel closed): `CExaminationSystem` 0xC, then
    /// the selection info is forgotten.
    pub fn stop_examination(&mut self) -> ScriptOutput {
        let mut out = ScriptOutput::default();
        self.stop_examining_object(&mut out);
        self.examining = false;
        self.selection_info = None;
        out
    }

    /// `CExaminationSystem` 0xC: stop the server examining an object (a cleared request).
    fn stop_examining_object(&mut self, out: &mut ScriptOutput) {
        if matches!(
            self.examination.kind,
            ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
        ) {
            out.requests
                .push(ScriptRequest::Examine(ExaminationRequest::default()));
        }
    }

    /// `CExaminationSystem` 0xE for an object request (`CExaminationSystem` 0xD): show the
    /// profile the object's weenie already holds, then ask the server. The other request
    /// types build their profiles locally from data (`CExaminationSystem` 0xB, 0xA, ...),
    /// which is not mirrored yet.
    fn start_examining(&mut self, out: &mut ScriptOutput) {
        if !matches!(
            self.examination.kind,
            ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
        ) {
            return;
        }
        if let Some(profile) = self.profiles.get(&self.examination.iid) {
            out.notices
                .push(GameplayNotice::UpdateExaminationProfile(profile.clone()));
        }
        out.requests.push(ScriptRequest::Examine(self.examination));
    }

    /// A server message for the player's scripts.
    pub fn receive(&mut self, message: ScriptMessage) -> ScriptOutput {
        let mut out = ScriptOutput::default();
        match message {
            // `CPlayer` 0x34: only for the current selection; PK figures are cleared unless
            // the player shows PK damage.
            ScriptMessage::UpdateSelectionInfo { mut info, id } => {
                if id != self.selection {
                    return out;
                }
                if !self.show_pk_damage {
                    info.pk_damage = 0;
                    info.pk_vigor_loss = 0;
                }
                self.selection_info = Some((id, info));
                out.notices
                    .push(GameplayNotice::UpdateSelectionInfo { id, info });
            }
            // `CExaminationSystem` 0x2: kept on the examined object's weenie, then shown.
            ScriptMessage::ExaminationProfile(profile) => {
                if matches!(
                    profile.request.kind,
                    ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
                ) {
                    self.profiles.insert(profile.request.iid, profile.clone());
                }
                out.notices
                    .push(GameplayNotice::UpdateExaminationProfile(profile));
            }
        }
        out
    }

    /// The object's weenie went away (`CObjectMaint::DestroyObjectInternal`): its profile goes
    /// with it.
    pub fn forget_object(&mut self, id: u64) {
        self.profiles.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selecting_sends_the_target_and_changes_the_examination_request() {
        let mut script = PlayerScript::default();
        let out = script.set_selection(7, false);
        assert_eq!(out.requests, vec![ScriptRequest::SetTarget(7)]);
        let request = ExaminationRequest::object(7);
        assert_eq!(
            out.notices,
            vec![
                GameplayNotice::SelectionChanged(7),
                GameplayNotice::ExaminationRequestChanged(request)
            ]
        );
        assert_eq!(script.selection(), Some(7));
        // The same selection again does nothing.
        assert_eq!(script.set_selection(7, false), ScriptOutput::default());
        // Clearing it clears the object request too.
        let out = script.set_selection(NO_SELECTION, false);
        assert_eq!(out.requests, vec![ScriptRequest::SetTarget(0)]);
        assert_eq!(
            out.notices[1],
            GameplayNotice::ExaminationRequestChanged(ExaminationRequest::default())
        );
        assert_eq!(script.last_selection(), Some(7));
        // A combat target gets the Legions extended request.
        let out = script.set_selection(9, true);
        assert!(matches!(
            out.notices[1],
            GameplayNotice::ExaminationRequestChanged(ExaminationRequest {
                kind: ExaminationRequest::OBJECT_EXTENDED,
                iid: 9,
                ..
            })
        ));
    }

    #[test]
    fn the_server_is_asked_only_while_the_examination_panel_shows() {
        let mut script = PlayerScript::default();
        script.set_selection(7, false);
        let out = script.start_examination(0x203);
        assert_eq!(
            out.requests,
            vec![ScriptRequest::Examine(ExaminationRequest::object(7))]
        );
        // Selecting another object stops the first and asks for the second.
        let out = script.set_selection(8, false);
        assert_eq!(
            out.requests,
            vec![
                ScriptRequest::SetTarget(8),
                ScriptRequest::Examine(ExaminationRequest::default()),
                ScriptRequest::Examine(ExaminationRequest::object(8)),
            ]
        );
        // The profile is kept for the object and shown again next time.
        let profile = ExaminationProfile {
            request: ExaminationRequest::object(8),
            nodes: vec![],
        };
        let out = script.receive(ScriptMessage::ExaminationProfile(profile.clone()));
        assert_eq!(
            out.notices,
            vec![GameplayNotice::UpdateExaminationProfile(profile.clone())]
        );
        script.stop_examination();
        script.set_selection(NO_SELECTION, false);
        script.set_selection(8, false);
        let out = script.start_examination(0x203);
        assert_eq!(
            out.notices,
            vec![GameplayNotice::UpdateExaminationProfile(profile)]
        );
    }

    #[test]
    fn selection_info_counts_only_for_the_selection() {
        let mut script = PlayerScript::default();
        script.set_selection(7, false);
        let info = SelectionInfo {
            kind: SelectionInfo::CREATURE,
            cur_health: 3,
            pk_damage: 2,
            max_health: 4,
            ..SelectionInfo::default()
        };
        assert_eq!(
            script.receive(ScriptMessage::UpdateSelectionInfo { info, id: 8 }),
            ScriptOutput::default()
        );
        let out = script.receive(ScriptMessage::UpdateSelectionInfo { info, id: 7 });
        let shown = SelectionInfo {
            pk_damage: 0,
            ..info
        };
        assert_eq!(
            out.notices,
            vec![GameplayNotice::UpdateSelectionInfo { id: 7, info: shown }]
        );
        assert_eq!(script.selection_info(), Some((7, shown)));
    }
}
