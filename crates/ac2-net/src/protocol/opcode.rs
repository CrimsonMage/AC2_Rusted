/// Event opcodes (`DispatchEvent`, `Proto_UI`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode;

impl Opcode {
    pub const CHAR_EXIT_GAME: u32 = 0x0001_0027;
    pub const CHARACTER_DELETION: u32 = 0x0001_0029;
    pub const CHAR_GEN_RESULT: u32 = 0x0001_002A;
    pub const ENTER_WORLD: u32 = 0x0001_002B;
    /// Data download finished (`OnEndDDD`); sent on queue 5 before the character set.
    pub const END_DDD: u32 = 0x0001_003B;
    pub const CHARACTER_SET: u32 = 0x0001_0055;
    pub const CHARACTER_ERROR: u32 = 0x0001_0056;
    pub const CHAR_GEN_VERIFICATION: u32 = 0x0001_0057;
    pub const CREATE_OBJECT: u32 = 0x0001_0058;
    pub const CREATE_PLAYER: u32 = 0x0001_0060;
    pub const PLAYER_DESCRIPTION: u32 = 0x0001_0068;
    pub const UPDATE_POSITION: u32 = 0x0001_0086;
    pub const CLIENT_POSITION: u32 = 0x0001_008E;
    pub const LOOK_AT: u32 = 0x0001_0096;
    pub const MIN_CHARACTER_SET: u32 = 0x0001_00B8;
    pub const DESTROY_OBJECT: u32 = 0x0001_00C2;
    pub const CLIENT_LOOK_AT: u32 = 0x0001_00C7;
    pub const CLIENT_LOOK: u32 = 0x0001_00C8;
    pub const LOOK: u32 = 0x0001_00C9;
    pub const SCENE_RENDERING_COMPLETE: u32 = 0x0001_00D1;
}
