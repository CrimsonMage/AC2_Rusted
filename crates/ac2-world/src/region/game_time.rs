/// One `GameTime` time-of-day boundary: start (fraction of the day), night flag and name (a
/// string-table id: `TimeOfDay`'s `name`, the `TIME` variable of `ID_TimeString`).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TimeOfDay {
    pub(super) start: f32,
    pub(super) is_night: bool,
    pub(super) name: u32,
}

/// One `GameTime` season: first day of year and name (a string-table id, `Season::season_token`).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Season {
    pub(super) start: u32,
    pub(super) name: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GameTime {
    pub(super) zero_time_of_year: f64,
    pub(super) zero_year: i32,
    pub(super) day_length: f32,
    pub(super) days_per_year: u32,
    /// `GameTime::year_spec`: the era's string id (the `YEARSPEC` variable).
    pub(super) year_spec: u32,
    pub(super) times_of_day: Vec<TimeOfDay>,
    pub(super) weekdays: Vec<u32>,
    pub(super) seasons: Vec<Season>,
}

impl GameTime {
    pub(super) fn server_time_at_day_fraction(
        &self,
        server_time: f64,
        day_fraction: f64,
    ) -> Option<f64> {
        if !server_time.is_finite()
            || !day_fraction.is_finite()
            || !(0.0..1.0).contains(&day_fraction)
            || !self.zero_time_of_year.is_finite()
            || !self.day_length.is_finite()
            || self.day_length <= 0.0
        {
            return None;
        }
        let day_length = f64::from(self.day_length);
        let day_index = ((server_time + self.zero_time_of_year) / day_length).floor();
        Some(day_index * day_length + day_fraction * day_length - self.zero_time_of_year)
    }

    pub(super) fn calendar_at(&self, server_time: f64) -> Option<GameCalendar> {
        if !server_time.is_finite()
            || !self.zero_time_of_year.is_finite()
            || !self.day_length.is_finite()
            || self.day_length <= 0.0
            || self.days_per_year == 0
        {
            return None;
        }
        let year_length = f64::from(self.day_length) * f64::from(self.days_per_year);
        if !year_length.is_finite() || year_length <= 0.0 {
            return None;
        }
        let total_time = server_time + self.zero_time_of_year;
        let year_index = (total_time / year_length).floor();
        let offset_in_year = total_time - year_index * year_length;
        let year = self.zero_year + year_index as i32;
        let day_of_year = ((offset_in_year / f64::from(self.day_length)).floor() as i64)
            .clamp(0, i64::from(self.days_per_year.saturating_sub(1)))
            as u32;
        let time_of_day = ((offset_in_year - f64::from(day_of_year) * f64::from(self.day_length))
            / f64::from(self.day_length))
        .clamp(0.0, 1.0);
        Some(GameCalendar {
            year,
            day_of_year,
            days_per_year: self.days_per_year,
            time_of_day,
        })
    }
}

/// The variables of `GameTime::GetDateTimeString`, from the running clock: the year (`YEAR`), the
/// day of the season, counted from 1 (`DAY`), the year's spec (`YEARSPEC`), the season's name
/// (`SEASON`) and the time of day's name (`TIME`). The names are string-table ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameDateTime {
    pub year: i32,
    pub day: i32,
    pub year_spec: u32,
    pub season: u32,
    pub time: u32,
}

/// `GameTime::GetGameTimeFromTimeStamp`'s results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeStampCalendar {
    pub year: i32,
    pub day: i32,
    pub season: u32,
    pub time_of_day: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GameCalendar {
    pub(super) year: i32,
    pub(super) day_of_year: u32,
    days_per_year: u32,
    pub(super) time_of_day: f64,
}

impl GameCalendar {
    pub(super) fn day_group_seed(self) -> u32 {
        self.day_of_year
            .wrapping_add(self.days_per_year.wrapping_mul(self.year as u32))
    }
}

/// Retail `GameTime` as a running clock (`GameTime::UseTime`, Legions 0x005D5980, identical to
/// admin): the time of day kept from the day's cached start, recomputed by `CalcDayBegin` only
/// when the next calendar event passes. Every intermediate is a double (the x87 ran at 53-bit
/// precision); `present_time_of_day` and `present_time_in_day_unit` are stored as `float`.
#[derive(Debug, Clone, PartialEq)]
pub struct GameClock {
    zero_time_of_year: f64,
    zero_year: i32,
    day_length: f32,
    days_per_year: i32,
    year_length: f64,
    /// `TimeOfDay::begin` of each time of day, and `Season::begin` of each season.
    times_of_day: Vec<f32>,
    /// `TimeOfDay::is_night` of each time of day (the FX's night gate reads the present one).
    times_of_day_night: Vec<bool>,
    seasons: Vec<u32>,
    /// The names of the seasons and times of day, and the year's spec (string-table ids).
    season_names: Vec<u32>,
    time_names: Vec<u32>,
    year_spec: u32,
    time_dilation: f64,
    user_offset: f64,
    clock_offset: f64,
    time_zero_start_delta: f64,
    time_of_day_begin: f64,
    time_of_next_event: f64,
    present_time_of_day: f32,
    present_time_in_day_unit: f32,
    current_year: i32,
    current_day: i32,
    current_season: u32,
    current_time_of_day: u32,
    /// `GameTime::m_bNeedLightingRefresh`.
    need_lighting_refresh: bool,
}

impl GameClock {
    /// `GameTime::GameTime` then the region's `StreamPack` (`SetYearLength`: the year is
    /// `day_length * days_per_year` as a double).
    pub(super) fn new(time: &GameTime) -> Self {
        Self {
            zero_time_of_year: time.zero_time_of_year,
            zero_year: time.zero_year,
            day_length: time.day_length,
            days_per_year: time.days_per_year as i32,
            year_length: f64::from(time.day_length) * f64::from(time.days_per_year),
            times_of_day: time.times_of_day.iter().map(|t| t.start).collect(),
            times_of_day_night: time.times_of_day.iter().map(|t| t.is_night).collect(),
            seasons: time.seasons.iter().map(|s| s.start).collect(),
            season_names: time.seasons.iter().map(|s| s.name).collect(),
            time_names: time.times_of_day.iter().map(|t| t.name).collect(),
            year_spec: time.year_spec,
            time_dilation: 1.0,
            user_offset: 0.0,
            clock_offset: 0.0,
            time_zero_start_delta: 0.0,
            time_of_day_begin: -1.0,
            time_of_next_event: 0.0,
            present_time_of_day: 0.0,
            present_time_in_day_unit: 0.0,
            current_year: 0,
            current_day: 0,
            current_season: 0,
            current_time_of_day: 0,
            need_lighting_refresh: false,
        }
    }

    /// `GameTime::UseTime` at `Timer::cur_time`.
    pub fn use_time(&mut self, cur_time: f64) {
        let time = (cur_time + self.clock_offset + self.time_zero_start_delta) * self.time_dilation
            + self.user_offset;
        if self.time_of_day_begin < 0.0 {
            self.calc_day_begin(time);
        }
        let day = f64::from(self.day_length);
        let fraction = (time - self.time_of_day_begin) / day;
        self.present_time_of_day = fraction as f32;
        if self.time_of_next_event <= time {
            if fraction >= 1.0 {
                self.calc_day_begin(time);
                self.present_time_of_day = ((time - self.time_of_day_begin) / day) as f32;
            }
            self.calc_time_of_day(time);
        }
    }

    /// `GameTime::CalcDayBegin`: the year, day and season at `time`, and when the day began.
    fn calc_day_begin(&mut self, time: f64) {
        let total = time + self.zero_time_of_year;
        let years = (total / self.year_length).floor();
        self.current_year = (years as i32).wrapping_add(self.zero_year);
        let offset = total - years * self.year_length;
        let day = (offset / f64::from(self.day_length)).floor() as i32;
        self.current_day = day;
        let mut season = 0;
        if self.seasons.len() > 1 {
            while (season as usize) < self.seasons.len() - 1
                && day >= self.seasons[season as usize + 1] as i32
            {
                season += 1;
            }
        }
        self.current_season = season;
        let begin = time - (offset - f64::from(day) * f64::from(self.day_length));
        self.time_of_day_begin = begin;
        self.time_of_next_event = begin;
    }

    /// `GameTime::CalcTimeOfDay`: the time of day `time` falls in, its progress, and when the
    /// next one starts. The last time of day runs to `day_length` (not to 1).
    fn calc_time_of_day(&mut self, time: f64) {
        let day = f64::from(self.day_length);
        let fraction = (time - self.time_of_day_begin) / day;
        let count = self.times_of_day.len();
        if count == 0 {
            self.time_of_next_event = day;
            return;
        }
        let mut index = 0;
        while index < count - 1 && fraction >= f64::from(self.times_of_day[index + 1]) {
            index += 1;
        }
        self.current_time_of_day = index as u32;
        let start = f64::from(self.times_of_day[index]);
        if index == count - 1 {
            self.time_of_next_event = day + self.time_of_day_begin;
            self.present_time_in_day_unit = ((fraction - start) / (day - start)) as f32;
        } else {
            let next = f64::from(self.times_of_day[index + 1]);
            self.time_of_next_event = next * day + self.time_of_day_begin;
            self.present_time_in_day_unit = ((fraction - start) / (next - start)) as f32;
        }
    }

    /// Whether the present time of day is a night one (`TimeOfDay::is_night` of
    /// `current_time_of_day`), the flag `FXManager::DoTargettedFX` gates a node's night part on.
    pub fn is_night(&self) -> bool {
        self.times_of_day_night
            .get(self.current_time_of_day as usize)
            .copied()
            .unwrap_or(false)
    }

    /// `GameTime::present_time_of_day`: the fraction of the day passed, `[0, 1)`.
    pub fn present_time_of_day(&self) -> f32 {
        self.present_time_of_day
    }

    /// `GameTime::GetDateTimeString` at the clock's present values: `DAY` is `current_day` less
    /// the season's first day, plus one (`GameTime.cpp` 0x618D40, admin).
    pub fn date_time(&self) -> GameDateTime {
        self.date_time_for(self.current_season, self.current_time_of_day)
    }

    /// `GameTime::GetDateTimeString` with a given season and time of day: `YEAR` and `DAY` always
    /// come from the running clock (`current_year`, `current_day` less the *current* season's
    /// first day); only `SEASON` and `TIME` follow the arguments.
    fn date_time_for(&self, season: u32, time_of_day: u32) -> GameDateTime {
        let season_start = self.seasons.get(self.current_season as usize).copied();
        GameDateTime {
            year: self.current_year,
            day: self.current_day - season_start.unwrap_or(0) as i32 + 1,
            year_spec: self.year_spec,
            season: self.season_names.get(season as usize).copied().unwrap_or(0),
            time: self
                .time_names
                .get(time_of_day as usize)
                .copied()
                .unwrap_or(0),
        }
    }

    /// `GameTime::GetTimeDilation` of this clock: the rate set by `TemporalDilation`.
    pub fn time_dilation(&self) -> f32 {
        self.time_dilation as f32
    }

    /// `GameTime::GetGameTimeFromTimeStamp` (Legions 0x005D5690): the year, day of the year,
    /// season and time of day of a server timestamp. As in retail, the season is chosen by the
    /// clock's running `current_day` and the time of day by the timestamp's offset from the start
    /// of that running day, not of the timestamp's own day.
    pub fn game_time_from_time_stamp(&self, time_stamp: f64) -> TimeStampCalendar {
        let total = time_stamp + self.zero_time_of_year;
        let years = (total / self.year_length).floor();
        let year = (years as i32).wrapping_add(self.zero_year);
        let offset = total - years * self.year_length;
        let day_length = f64::from(self.day_length);
        let day = (f64::from(offset as f32) / day_length).floor() as i32;
        let mut season = 0_u32;
        if self.seasons.len() > 1 {
            while (season as usize) < self.seasons.len() - 1
                && self.current_day >= self.seasons[season as usize + 1] as i32
            {
                season += 1;
            }
        }
        let mut time_of_day = 0_u32;
        if self.times_of_day.len() > 1 {
            let stamp = f64::from(time_stamp as f32);
            let into_day = f64::from(offset as f32) - f64::from(self.current_day) * day_length;
            let fraction = (stamp - (stamp - into_day)) / day_length;
            while (time_of_day as usize) < self.times_of_day.len() - 1
                && !(fraction < f64::from(self.times_of_day[time_of_day as usize + 1]))
            {
                time_of_day += 1;
            }
        }
        TimeStampCalendar {
            year,
            day,
            season,
            time_of_day,
        }
    }

    /// `GameTime::InqGameDateTimeString`: the date and time variables for a server timestamp
    /// (`GetGameTimeFromTimeStamp`, then `GetDateTimeString`).
    pub fn date_time_at(&self, time_stamp: f64) -> GameDateTime {
        let calendar = self.game_time_from_time_stamp(time_stamp);
        self.date_time_for(calendar.season, calendar.time_of_day)
    }

    /// `current_year`, `current_day` (of the year), `current_season` and `current_time_of_day`.
    pub fn calendar(&self) -> (i32, i32, u32, u32) {
        (
            self.current_year,
            self.current_day,
            self.current_season,
            self.current_time_of_day,
        )
    }

    /// `GameTime::day_length`: seconds in a day.
    pub fn day_length(&self) -> f32 {
        self.day_length
    }

    /// `GameTime::days_per_year`.
    pub fn days_per_year(&self) -> i32 {
        self.days_per_year
    }

    /// `GameTime::GetNeedLightingRefresh`.
    pub fn need_lighting_refresh(&self) -> bool {
        self.need_lighting_refresh
    }

    /// `GameTime::NotifyLightingRefreshed`.
    pub fn lighting_refreshed(&mut self) {
        self.need_lighting_refresh = false;
    }

    /// `GameTime::AlterTimeOfDayImmediate` (Legions 0x005D5B70): moves the user offset so the
    /// day is at `fraction_of_day` now, wraps a negative offset by one day, asks for a lighting
    /// refresh and runs `UseTime` at `cur_time` with the next event reset. Every step is a double.
    pub fn alter_time_of_day_immediate(&mut self, fraction_of_day: f32, cur_time: f64) {
        self.need_lighting_refresh = true;
        self.user_offset += (f64::from(fraction_of_day) - f64::from(self.present_time_of_day))
            * f64::from(self.day_length);
        if self.user_offset < 0.0 {
            self.user_offset += f64::from(self.day_length);
        }
        self.time_of_next_event = 0.0;
        self.use_time(cur_time);
    }

    /// `GameTime::TemporalDilationInternal` (Legions 0x005D5AA0): refused (false) for a dilation
    /// of 0.0002 or less. Otherwise the dilation takes effect, the clock is refreshed at
    /// `cur_time`, and the user offset is moved back by the time the new rate added, so the day
    /// does not jump (the day count difference is taken as an unsigned word, as retail does).
    pub fn temporal_dilation(&mut self, dilation: f32, cur_time: f64) -> bool {
        if dilation <= 0.0002 {
            return false;
        }
        let old_present = f64::from(self.present_time_of_day);
        let old_total = self
            .days_per_year
            .wrapping_mul(self.current_year)
            .wrapping_add(self.current_day);
        self.time_dilation = f64::from(dilation);
        self.time_of_day_begin = -1.0;
        self.use_time(cur_time);
        let new_total = self
            .days_per_year
            .wrapping_mul(self.current_year)
            .wrapping_add(self.current_day);
        let days = f64::from(new_total.wrapping_sub(old_total) as u32);
        let advanced = days + f64::from(self.present_time_of_day) - old_present;
        self.time_of_day_begin = -1.0;
        self.user_offset -= advanced * f64::from(self.day_length);
        true
    }

    /// `present_time_in_day_unit`: progress through the current time of day.
    pub fn present_time_in_day_unit(&self) -> f32 {
        self.present_time_in_day_unit
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;

    fn time() -> GameTime {
        let time_of_day = |start: f32, name: u32| TimeOfDay {
            start,
            is_night: false,
            name,
        };
        let season = |start: u32, name: u32| Season { start, name };
        GameTime {
            zero_time_of_year: 50.0,
            zero_year: 10,
            day_length: 100.0,
            days_per_year: 4,
            year_spec: 0x5000_0001,
            times_of_day: vec![
                time_of_day(0.0, 0x5000_0010),
                time_of_day(0.25, 0x5000_0011),
                time_of_day(0.75, 0x5000_0012),
            ],
            weekdays: Vec::new(),
            seasons: vec![season(0, 0x5000_0020), season(2, 0x5000_0021)],
        }
    }

    #[test]
    fn the_clock_keeps_the_day_start_until_the_next_event() {
        let mut clock = GameClock::new(&time());
        // 1000 + 50 = 1050: year 2 (10 + 2), 250 into it = day 2 (second season), 0.5 of a day.
        clock.use_time(1000.0);
        assert_eq!(clock.calendar(), (12, 2, 1, 1));
        assert_eq!(clock.present_time_of_day(), 0.5);
        assert_eq!(clock.present_time_in_day_unit(), 0.5);
        // The last time of day runs to `day_length`, retail's quirk.
        clock.use_time(1030.0);
        assert_eq!(clock.calendar().3, 2);
        assert_eq!(
            clock.present_time_in_day_unit(),
            ((0.8 - 0.75) / (100.0 - 0.75)) as f32
        );
        // The next day starts at 1050.
        clock.use_time(1060.0);
        assert_eq!(clock.calendar(), (12, 3, 1, 0));
        assert_eq!(clock.present_time_of_day(), 0.1);
    }

    #[test]
    fn the_date_and_time_names_follow_the_calendar() {
        let mut clock = GameClock::new(&time());
        // 1000 + 50 = 1050: year 12 (10 + 2), day 2 of the year, in season 1 (from day 2), so
        // `DAY` is 2 - 2 + 1 = 1; time of day 1 (from 0.25) is `TIME`.
        clock.use_time(1000.0);
        assert_eq!(
            clock.date_time(),
            GameDateTime {
                year: 12,
                day: 1,
                year_spec: 0x5000_0001,
                season: 0x5000_0021,
                time: 0x5000_0011,
            }
        );
        // The next day starts at 1050: day 3 of the year, so `DAY` is 3 - 2 + 1 = 2; time 0.
        clock.use_time(1060.0);
        assert_eq!(clock.date_time().day, 2);
        assert_eq!(clock.date_time().time, 0x5000_0010);
    }

    #[test]
    fn a_time_stamp_reads_season_and_time_of_day_against_the_running_day() {
        let mut clock = GameClock::new(&time());
        assert_eq!(clock.time_dilation(), 1.0);
        clock.use_time(1000.0);
        // Running day 2 (season 1). Stamp 1230 + 50 = 1280: year 13, offset 80 = day 0 of that
        // year, but retail picks the season from the running day and measures the time of day
        // from the running day's start, so the offset (80 - 200) / 100 falls before every start.
        let calendar = clock.game_time_from_time_stamp(1230.0);
        assert_eq!(
            calendar,
            TimeStampCalendar {
                year: 13,
                day: 0,
                season: 1,
                time_of_day: 0
            }
        );
        // Stamp 1000 + 50 = 1050: offset 250 is day 2, 0.5 into it, time of day 1.
        let calendar = clock.game_time_from_time_stamp(1000.0);
        assert_eq!((calendar.year, calendar.day), (12, 2));
        assert_eq!((calendar.season, calendar.time_of_day), (1, 1));
        // `InqGameDateTimeString`: the names follow, `YEAR` and `DAY` stay the running clock's.
        assert_eq!(clock.date_time_at(1000.0), clock.date_time());
        assert_eq!(clock.date_time_at(1230.0).time, 0x5000_0010);
        assert_eq!(clock.date_time_at(1230.0).year, 12);
        assert!(clock.temporal_dilation(2.0, 1000.0));
        assert_eq!(clock.time_dilation(), 2.0);
    }

    #[test]
    fn altering_the_time_of_day_moves_the_day_to_the_fraction() {
        let mut clock = GameClock::new(&time());
        clock.use_time(1000.0);
        assert_eq!(clock.present_time_of_day(), 0.5);
        clock.alter_time_of_day_immediate(0.75, 1000.0);
        assert_eq!(clock.present_time_of_day(), 0.75);
        assert!(clock.need_lighting_refresh());
        clock.lighting_refreshed();
        assert!(!clock.need_lighting_refresh());
        // A move back past the start of the offset wraps by one day.
        clock.alter_time_of_day_immediate(0.25, 1000.0);
        assert_eq!(clock.present_time_of_day(), 0.25);
    }

    #[test]
    fn temporal_dilation_keeps_the_time_of_day_and_refuses_a_tiny_rate() {
        let mut clock = GameClock::new(&time());
        clock.use_time(1000.0);
        assert!(!clock.temporal_dilation(0.0002, 1000.0));
        assert!(clock.temporal_dilation(2.0, 1000.0));
        assert_eq!(clock.present_time_of_day(), 0.5);
        // The day now passes twice as fast: 20 seconds are 0.4 of a day.
        clock.use_time(1020.0);
        assert!((clock.present_time_of_day() - 0.9).abs() < 1e-5);
    }
}
