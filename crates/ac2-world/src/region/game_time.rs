/// One `GameTime` time-of-day boundary: start (fraction of the day), night flag and name.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TimeOfDay {
    pub(super) start: f32,
    pub(super) is_night: bool,
    pub(super) name: String,
}

/// One `GameTime` season: first day of year and name.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Season {
    pub(super) start: u32,
    pub(super) name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GameTime {
    pub(super) zero_time_of_year: f64,
    pub(super) zero_year: i32,
    pub(super) day_length: f32,
    pub(super) days_per_year: u32,
    pub(super) year_spec: String,
    pub(super) times_of_day: Vec<TimeOfDay>,
    pub(super) weekdays: Vec<String>,
    pub(super) seasons: Vec<Season>,
}

impl GameTime {
    pub(super) fn format_date_time_strings(&self, server_time: f64) -> Option<(String, String)> {
        let calendar = self.calendar_at(server_time)?;
        let season = self
            .seasons
            .iter()
            .rev()
            .find(|season| calendar.day_of_year >= season.start)
            .or_else(|| self.seasons.first())?;
        let time_of_day = self
            .times_of_day
            .iter()
            .rev()
            .find(|time| calendar.time_of_day >= f64::from(time.start))
            .or_else(|| self.times_of_day.first())?;
        let time = if time_of_day.name.len() >= 30 {
            " ".to_string()
        } else {
            time_of_day.name.clone()
        };
        let date = format!(
            "{} {}, {} {}",
            season.name,
            calendar.day_of_year.saturating_sub(season.start) + 1,
            calendar.year,
            self.year_spec
        );
        Some((
            if date.len() >= 60 {
                " ".to_string()
            } else {
                date
            },
            time,
        ))
    }

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

    pub(super) fn night_light_factor_at(&self, server_time: f64) -> Option<f32> {
        let time = self.calendar_at(server_time)?.time_of_day as f32;
        let before_index = self
            .times_of_day
            .iter()
            .rposition(|entry| entry.start <= time)
            .unwrap_or(self.times_of_day.len().checked_sub(1)?);
        let before = self.times_of_day.get(before_index)?;
        let after = self
            .times_of_day
            .get((before_index + 1) % self.times_of_day.len())?;
        let span = if before_index + 1 == self.times_of_day.len() {
            1.0 - before.start + after.start
        } else {
            after.start - before.start
        };
        let elapsed = if before_index + 1 == self.times_of_day.len() {
            (time - before.start).rem_euclid(1.0)
        } else {
            time - before.start
        };
        let ratio = if span > 0.0 {
            (elapsed / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // TimeOfDay only exposes authored named day/night boundary records. Ease between those
        // records so consumers can use it as a sunup/sundown envelope without stepping or
        // changing the separate SkyDesc world-lighting interpolation.
        let eased_ratio = ratio * ratio * (3.0 - 2.0 * ratio);
        let before_night = if before.is_night { 1.0 } else { 0.0 };
        let after_night = if after.is_night { 1.0 } else { 0.0 };
        Some(before_night + (after_night - before_night) * eased_ratio)
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
