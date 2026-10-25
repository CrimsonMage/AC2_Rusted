//! The running `GameTime` clock's calendar on the Legions DAT (`AC2_DAT_DIR`): the year, day,
//! season and time-of-day names, against the `GameTime::CalcDayBegin` and `CalcTimeOfDay`
//! formulas written out from the record's fields.

use ac2_db::ResourceStore;

use super::ac2_region::load_ac2_region;

#[test]
fn the_clock_names_follow_the_gametime_formula_on_the_dataset() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let region = load_ac2_region(&store)
        .expect("region decodes")
        .expect("region exists");
    let time = &region.game_time;
    let day_length = f64::from(time.day_length);
    let year_length = day_length * f64::from(time.days_per_year);
    let mut clock = region.game_clock();
    for server_time in [0.0, 1234.5, 9000.0, 54_321.0, 1.0e6] {
        clock.use_time(server_time);
        // CalcDayBegin: the year and the day of the year from the zero of the year.
        let total = server_time + time.zero_time_of_year;
        let years = (total / year_length).floor();
        let offset = total - years * year_length;
        let day = (offset / day_length).floor() as i64;
        // The season is the last one that begins on or before the day.
        let season = time
            .seasons
            .iter()
            .rposition(|season| i64::from(season.start) <= day)
            .unwrap_or(0);
        // CalcTimeOfDay: the fraction of the day, and the last time of day that begins by it.
        let fraction = (offset - day as f64 * day_length) / day_length;
        let tod = time
            .times_of_day
            .iter()
            .rposition(|tod| f64::from(tod.start) <= fraction)
            .unwrap_or(0);
        let date_time = clock.date_time();
        assert_eq!(
            date_time.year,
            time.zero_year + years as i32,
            "year at {server_time}"
        );
        assert_eq!(
            date_time.day,
            (day - i64::from(time.seasons[season].start) + 1) as i32,
            "day at {server_time}"
        );
        assert_eq!(
            date_time.season, time.seasons[season].name,
            "season at {server_time}"
        );
        assert_eq!(
            date_time.time, time.times_of_day[tod].name,
            "time at {server_time}"
        );
        assert_eq!(date_time.year_spec, time.year_spec);
    }
}
