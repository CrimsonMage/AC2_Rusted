//! The client's game clock as the scripts see it (`GameTime::current_game_time`, read by
//! `gGameTime`'s efuns), and the changes they ask of it. The clock itself belongs to the client
//! (`ac2_world::GameClock`); each frame the client hands the host a [`GameTimeState`] and applies
//! the [`GameTimeRequest`]s the scripts made.

use super::ScriptHost;

/// `GameTime::GetDateTimeString`'s variables at the running clock: the year, the day of the
/// season counted from 1, and the year spec, season and time-of-day names (string ids in the
/// game-time table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameDate {
    pub year: i32,
    pub day: i32,
    pub year_spec: u32,
    pub season: u32,
    pub time: u32,
}

/// The fields of `GameTime::current_game_time` the efuns read.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GameTimeState {
    /// `TimeOfDay::is_night` of `current_time_of_day`.
    pub night: bool,
    /// `present_time_of_day`: the fraction of the day passed.
    pub time_of_day: f32,
    /// `day_length`, in seconds.
    pub day_length: f32,
    /// `days_per_year`.
    pub days_per_year: i32,
    /// `current_year`.
    pub year: i32,
    /// `current_day`: the day of the year.
    pub day_of_year: i32,
    /// The date and time strings' variables.
    pub date: GameDate,
}

/// A change a script asked of the game clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameTimeRequest {
    /// `GameTime::AlterTimeOfDayImmediate(fraction of day)`.
    AlterTimeOfDay(f32),
    /// `GameTime::TemporalDilation(dilation)`.
    TemporalDilation(f32),
}

impl ScriptHost {
    /// `GameTime::current_game_time`, as the client's clock stands this frame; `None` while the
    /// client has no game time (the `gGameTime` efuns then answer 0).
    pub fn set_game_time(&mut self, state: Option<GameTimeState>) {
        self.vm.game_time = state;
    }

    /// The clock changes the scripts asked for since the last call, in order.
    pub fn take_game_time_requests(&mut self) -> Vec<GameTimeRequest> {
        std::mem::take(&mut self.vm.game_time_requests)
    }
}
