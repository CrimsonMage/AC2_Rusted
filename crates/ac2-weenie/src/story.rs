//! The story scenes a script hands the UI (`gmSceneInfo`, `gmKeyframe`, `List<gmSceneInfo>`):
//! plain data, built by the script VM's glue and read by the gameplay story slot.

use ac2_net::protocol::StringInfo;

/// `gmKeyframe::m_definedTracks` bits: the video (`SetVideoTrack`), audio (`SetAudioTrack`),
/// voice (`SetVoiceTrack`), movie (`SetMovieTrack`) and subtitle (`SetSubtitle`) tracks.
pub const TRACK_VIDEO: u32 = 0x1;
pub const TRACK_AUDIO: u32 = 0x2;
pub const TRACK_VOICE: u32 = 0x4;
pub const TRACK_MOVIE: u32 = 0x8;
pub const TRACK_SUBTITLE: u32 = 0x10;

/// `gmKeyframe`'s duration when none is set (`HasDurationDefined` is `m_ttDuration != -1.0`).
pub const UNDEFINED_DURATION: f64 = -1.0;

/// One `gmKeyframe`: the tracks a story shows for a moment of a scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    /// `m_sceneID`: the scene the keyframe belongs to (set when it is added to a scene).
    pub scene_id: u32,
    /// `m_uiFrameNum`: its place in the scene, from 1 (set when it is added).
    pub frame_num: u32,
    /// `m_ttDuration` in seconds; [`UNDEFINED_DURATION`] when not set.
    pub duration: f64,
    /// `m_didVideoFragment`: the image's DID.
    pub video: u32,
    /// `m_didAudioFragment`: the music's DID.
    pub audio: u32,
    /// `m_didVoiceFragment`: the voice's DID.
    pub voice: u32,
    /// `m_siTextFragment`: the subtitle.
    pub subtitle: StringInfo,
    /// `m_definedTracks`: the `TRACK_*` bits set.
    pub defined_tracks: u32,
}

impl Default for Keyframe {
    /// `gmKeyframe::gmKeyframe(this)` with the duration left undefined.
    fn default() -> Self {
        Self {
            scene_id: 0,
            frame_num: 0,
            duration: UNDEFINED_DURATION,
            video: 0,
            audio: 0,
            voice: 0,
            subtitle: StringInfo::default(),
            defined_tracks: 0,
        }
    }
}

impl Keyframe {
    /// `gmKeyframe::HasVideoTrack`.
    pub fn has_video(&self) -> bool {
        self.defined_tracks & TRACK_VIDEO != 0
    }

    /// `gmKeyframe::HasAudioTrack`.
    pub fn has_audio(&self) -> bool {
        self.defined_tracks & TRACK_AUDIO != 0
    }

    /// `gmKeyframe::HasVoiceTrack`.
    pub fn has_voice(&self) -> bool {
        self.defined_tracks & TRACK_VOICE != 0
    }

    /// `gmKeyframe::HasSubtitle`.
    pub fn has_subtitle(&self) -> bool {
        self.defined_tracks & TRACK_SUBTITLE != 0
    }

    /// `gmKeyframe::HasDurationDefined`.
    pub fn has_duration(&self) -> bool {
        self.duration != UNDEFINED_DURATION
    }
}

/// One `gmSceneInfo`: a scene of a play sequence or a portal scene, and its keyframes in order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneInfo {
    /// `_ID`: the scene's id, reported back when the scene has played.
    pub id: u32,
    /// `_siName`.
    pub name: StringInfo,
    /// `_actID`: the act the scene belongs to.
    pub act_id: u32,
    /// `_uiSceneNum`: the scene's order, which `insert_sort` keeps.
    pub scene_num: u32,
    /// `_bIsHidden`.
    pub hidden: bool,
    /// `_bIsPlayable`: only a playable scene's keyframes play.
    pub playable: bool,
    /// `_keyframeList`.
    pub keyframes: Vec<Keyframe>,
}
