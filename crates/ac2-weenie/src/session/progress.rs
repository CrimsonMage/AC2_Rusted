//! The session's skill, shortcut and social entry points: the `WM_Skill` answers and the login
//! options kept in [`Progress`], and the shortcut IFun (`CPlayer` 0x87, `SetShortcut`).

use ac2_net::protocol::{unpack_values, NativeObject, PackValue, ScriptCall, ShortcutInfo};

use super::gameplay::player_qualities;
use super::{OnlineSession, SessionEvent};
use crate::notice::GameplayNotice;
use crate::progress::{Progress, SkillRequest};

/// `WM_Player` 0x1A (`SetShortcut`'s stub).
const SET_SHORTCUT: u32 = 0x8212_0018;

impl OnlineSession {
    /// Hands `functions` (server script calls) to the script VM: the session's stand-ins leave
    /// them alone and pass them on raw as [`SessionEvent::Interp`].
    pub fn set_script_functions(&mut self, functions: &[u32]) {
        self.script_functions = functions.iter().copied().collect();
    }

    /// Whether a stand-in handles server call `function` (its notices come from the session),
    /// unless it was handed to the script VM.
    pub fn stand_in_handles(&self, function: u32) -> bool {
        !self.script_functions.contains(&function)
            && (function == ac2_net::protocol::script_function::INIT_PLAYER
                || function >> 16 == 0x8214
                || ac2_net::protocol::ScriptMessage::decode(function, &[]).is_some())
    }

    /// Skills, shortcuts, fellowship and allegiance as the panels read them.
    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    /// The UI's view of the notices (the session's and the script VM's): the selection,
    /// fellowship and allegiance are kept in [`Progress`]. The client applies each frame's
    /// notices once.
    pub fn apply_notices(&mut self, notices: &[GameplayNotice]) {
        for notice in notices {
            self.progress.social.apply(notice);
            self.progress.selection.apply(notice);
        }
    }

    /// `CPlayer` 0x6A's state without its output: when the script VM runs `SetSelection`, the
    /// stand-ins that still read the selection (use, combat checks) follow it.
    pub fn mirror_selection(&mut self, id: u64) {
        self.script.mirror_selection(id);
    }

    /// `gmCWInterface::SetShortcut` -> `CPlayer` 0x87: the shortcut into slot `index` of the
    /// bar (`GameplayOptionsProfile::SetShortcut`, refused past the end), then `WM_Player`
    /// 0x1A (the shortcut, the index). Returns whether it took.
    pub fn set_shortcut(&mut self, index: u32, shortcut: ShortcutInfo) -> bool {
        if !self.progress.set_shortcut(index, shortcut.clone()) {
            return false;
        }
        let call = ScriptCall::new(
            SET_SHORTCUT,
            vec![
                PackValue::native(NativeObject::ShortcutInfo(shortcut)),
                PackValue::Int(index),
            ],
        )
        .expect("SetShortcut's layout");
        self.send(call.to_event());
        true
    }

    /// A skill panel request (`gmCWInterface::RequestTrainSkill` and kin): its `WM_Skill`
    /// call, as the stub packs it.
    pub fn request_skill(&mut self, request: SkillRequest) {
        self.send(request.call().to_event());
    }

    /// A `WM_Skill` answer to the player; `None` for other functions.
    pub(super) fn receive_skill(
        &mut self,
        function: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        if function >> 16 != 0x8214 {
            return None;
        }
        let values = unpack_values(arguments).ok()?;
        let q = player_qualities(&self.world);
        let notices = self.progress.receive_skill(function, &values, q)?;
        Some(notices.into_iter().map(SessionEvent::Notice).collect())
    }

    /// `CPlayer` 0x65's options (`WM_Player` 0x27 argument 4).
    pub(super) fn init_player_options(&mut self, arguments: &[u8]) -> Vec<SessionEvent> {
        let Ok(values) = unpack_values(arguments) else {
            return Vec::new();
        };
        if let Some(NativeObject::GameplayOptionsProfile(profile)) =
            values.get(3).and_then(|options| options.as_native())
        {
            self.chat_options(profile);
        }
        values
            .get(3)
            .and_then(|options| self.progress.login_options(options))
            .map(SessionEvent::Notice)
            .into_iter()
            .collect()
    }

    pub(super) fn progress_quality_changed(
        &self,
        table: u32,
        quality: u32,
    ) -> Option<SessionEvent> {
        let q = player_qualities(&self.world);
        self.progress
            .quality_changed(q, table, quality)
            .map(SessionEvent::Notice)
    }
}
