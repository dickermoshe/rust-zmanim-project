use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use icu_calendar::{
    cal::Hebrew,
    options::{DateAddOptions, Overflow},
    types::DateDuration,
    Date, Gregorian,
};

use crate::{
    abs_date_to_gregorian_date, chalakim_since_molad_tohu, HebrewHolidayCalendar, HebrewMonth,
    CHALAKIM_PER_DAY, CHALAKIM_PER_HOUR, CHALAKIM_PER_MINUTE, JEWISH_EPOCH,
};

/// A trait for calculating times related to the molad (new moon) and Kiddush Levana (blessing of the new moon).
///
/// This trait provides methods for calculating various times related to the Jewish lunar month,
/// specifically for the mitzvah of Kiddush Levana.
pub trait MoladCalendar {
    /// Returns the latest time of _Kiddush Levana_ calculated as 15 days after the molad.
    ///
    /// # Returns
    /// - `Some((DateTime, HebrewMonth))` - The time representing 15 days after the molad and the Hebrew month
    /// - `None` - If the zman will not occur on this day
    //
    fn sof_zman_kidush_levana_15_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)>;

    /// Returns the earliest time of _Kiddush Levana_ according to
    /// [Rabbeinu Yonah](https://en.wikipedia.org/wiki/Yonah_Gerondi)'s opinion that it can be
    /// said 3 days after the molad.
    ///
    /// # Returns
    /// - `Some((DateTime, HebrewMonth))` - The time representing 3 days after the molad and the Hebrew month
    /// - `None` - If the zman will not occur on this day
    fn tchilas_zman_kidush_levana_3_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)>;

    /// Returns the earliest time of _Kiddush Levana_ according to the opinions that it should
    /// not be said until 7 days after the molad.
    ///
    /// # Returns
    /// - `Some((DateTime, HebrewMonth))` - The time representing 7 days after the molad and the Hebrew month
    /// - `None` - If the zman will not occur on this day
    fn tchilas_zman_kidush_levana_7_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)>;

    /// Returns the latest time of Kiddush Levana according to the
    /// [Maharil](https://en.wikipedia.org/wiki/Yaakov_ben_Moshe_Levi_Moelin)'s opinion that it
    /// is calculated as halfway between molad and molad.
    ///
    /// This adds half the 29 days, 12 hours and 793 chalakim time between molad and molad
    /// (14 days, 18 hours, 22 minutes and 666 milliseconds) to the month's molad.
    ///
    /// # Returns
    /// - `Some((DateTime, HebrewMonth))` - The time representing halfway between molad and molad and the Hebrew month
    /// - `None` - If the zman will not occur on this day
    fn sof_zman_kidush_levana_between_moldos<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)>;

    /// Returns the time of the molad (new moon) for the current date's Hebrew month.
    ///
    /// The molad is the precise moment of the conjunction of the sun and moon.
    ///
    /// # Returns
    /// - `Some((DateTime, HebrewMonth))` - The time of the molad and the Hebrew month
    /// - `None` - If the zman will not occur on this day
    fn molad<Tz: TimeZone>(&self, tz: &Tz) -> Option<(DateTime<Tz>, HebrewMonth)>;
}

impl MoladCalendar for Date<Gregorian> {
    fn sof_zman_kidush_levana_15_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)> {
        let hebrew = self.to_calendar(Hebrew);

        if hebrew.day_of_month().0 < 11 || hebrew.day_of_month().0 > 17 {
            return None;
        }
        let molad = months_molad(&hebrew)? + chrono::Duration::hours(24 * 15);
        if is_same_day(&hebrew, &molad, tz) {
            return Some((molad.with_timezone(tz), hebrew.hebrew_month()));
        }
        None
    }

    fn tchilas_zman_kidush_levana_3_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)> {
        let hebrew = self.to_calendar(Hebrew);

        if hebrew.day_of_month().0 > 5 && hebrew.day_of_month().0 < 30 {
            return None;
        }
        let molad = months_molad(&hebrew)? + chrono::Duration::hours(72);

        if is_same_day(&hebrew, &molad, tz) {
            return Some((molad.with_timezone(tz), hebrew.hebrew_month()));
        }

        if hebrew.day_of_month().0 == 30 {
            let mut add_option = DateAddOptions::default();
            add_option.overflow = Some(Overflow::Constrain);

            let new = hebrew
                .try_added_with_options(DateDuration::for_months(1), add_option)
                .ok()?;
            let molad = months_molad(&new)? + chrono::Duration::hours(72);
            if is_same_day(&new, &molad, tz) {
                return Some((molad.with_timezone(tz), new.hebrew_month()));
            }
        }
        None
    }

    fn tchilas_zman_kidush_levana_7_days<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)> {
        let hebrew = self.to_calendar(Hebrew);

        if hebrew.day_of_month().0 < 4 || hebrew.day_of_month().0 > 9 {
            return None;
        }
        let molad = months_molad(&hebrew)? + chrono::Duration::hours(168);
        if is_same_day(&hebrew, &molad, tz) {
            return Some((molad.with_timezone(tz), hebrew.hebrew_month()));
        }
        None
    }

    fn sof_zman_kidush_levana_between_moldos<Tz: TimeZone>(
        &self,
        tz: &Tz,
    ) -> Option<(DateTime<Tz>, HebrewMonth)> {
        let hebrew = self.to_calendar(Hebrew);

        if hebrew.day_of_month().0 < 11 || hebrew.day_of_month().0 > 16 {
            return None;
        }
        let molad = months_molad(&hebrew)?
            + chrono::Duration::hours(24 * 14 + 18)
            + chrono::Duration::minutes(22)
            + chrono::Duration::seconds(1)
            + chrono::Duration::milliseconds(666);
        if is_same_day(&hebrew, &molad, tz) {
            return Some((molad.with_timezone(tz), hebrew.hebrew_month()));
        }
        None
    }

    fn molad<Tz: TimeZone>(&self, tz: &Tz) -> Option<(DateTime<Tz>, HebrewMonth)> {
        let hebrew = self.to_calendar(Hebrew);
        let day = hebrew.day_of_month().0;
        if day > 2 && day < 27 {
            return None;
        }
        let molad = months_molad(&hebrew)?;
        if !is_same_gregorian_day(&hebrew, &molad, tz) {
            let mut add_option = DateAddOptions::default();
            add_option.overflow = Some(Overflow::Constrain);
            if day > 26 {
                // Next month molad can fall on the current Gregorian day near month boundaries.
                let new = hebrew
                    .try_added_with_options(DateDuration::for_months(1), add_option)
                    .ok()?;
                let molad = months_molad(&new.to_calendar(Hebrew))?;
                if is_same_gregorian_day(&hebrew, &molad, tz) {
                    return Some((molad.with_timezone(tz), new.hebrew_month()));
                }
            }
            return None;
        }
        Some((molad.with_timezone(tz), hebrew.hebrew_month()))
    }
}

fn is_same_day<Tz: TimeZone>(hdate: &Date<Hebrew>, gdate: &DateTime<Utc>, tz: &Tz) -> bool {
    let gdate_tz = tz.from_utc_datetime(&gdate.naive_utc());

    // Convert the Gregorian datetime to a Hebrew date
    let gregorian_date = match Date::try_new_gregorian(
        gdate_tz.year(),
        gdate_tz.month() as u8,
        gdate_tz.day() as u8,
    ) {
        Ok(date) => date,
        Err(_) => return false,
    };

    let hebrew_date_from_gdate = gregorian_date.to_calendar(Hebrew);

    // Compare Hebrew date components
    hdate.day_of_month().0 == hebrew_date_from_gdate.day_of_month().0
        && hdate.hebrew_month() == hebrew_date_from_gdate.hebrew_month()
        && hdate.extended_year() == hebrew_date_from_gdate.extended_year()
}

fn is_same_gregorian_day<Tz: TimeZone>(
    hdate: &Date<Hebrew>,
    gdate: &DateTime<Utc>,
    tz: &Tz,
) -> bool {
    let gdate_local = tz.from_utc_datetime(&gdate.naive_utc()).date_naive();
    let gregorian_date = hdate.gregorian_date();
    let hdate_greg = NaiveDate::from_ymd_opt(
        gregorian_date.extended_year(),
        gregorian_date.month().month_number() as u32,
        gregorian_date.day_of_month().0 as u32,
    );

    Some(gdate_local) == hdate_greg
}
struct MoladData {
    pub hours: i64,
    pub minutes: i64,
    pub chalakim: i64,
}
fn _get_molad(date: &Date<Hebrew>) -> Option<(Date<Gregorian>, MoladData)> {
    let chalakim_since_molad_tohu =
        chalakim_since_molad_tohu(date.extended_year(), date.hebrew_month());
    let abs_date = JEWISH_EPOCH + (chalakim_since_molad_tohu / CHALAKIM_PER_DAY);
    let mut gregorian_date = abs_date_to_gregorian_date(abs_date)?;
    let conjunction_day = chalakim_since_molad_tohu / CHALAKIM_PER_DAY;
    let conjunction_parts = chalakim_since_molad_tohu - conjunction_day * CHALAKIM_PER_DAY;
    let mut hours = conjunction_parts / CHALAKIM_PER_HOUR;
    let adjusted_conjunction_parts = conjunction_parts - (hours * CHALAKIM_PER_HOUR);
    let minutes = adjusted_conjunction_parts / CHALAKIM_PER_MINUTE;
    let chalakim = adjusted_conjunction_parts - (minutes * CHALAKIM_PER_MINUTE);
    if hours >= 6 {
        gregorian_date
            .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
            .ok()?;
    }
    hours = (hours + 18) % 24;
    Some((
        gregorian_date,
        MoladData {
            hours,
            minutes,
            chalakim,
        },
    ))
}
// Molad and Kiddush Levana
pub(crate) fn months_molad(date: &Date<Hebrew>) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;

    let (molad, molad_data) = _get_molad(date)?;

    // Get the Gregorian date components from molad JewishCalendar
    let year = molad.extended_year();
    let month = molad.month().month_number(); // Convert from 0-based to 1-based
    let day = molad.day_of_month().0 as u32;

    let molad_seconds = molad_data.chalakim as f64 * 10.0 / 3.0;
    let seconds = molad_seconds as u32;
    let millis = ((molad_seconds - seconds as f64) * 1000.0) as u32;

    let naive_datetime = chrono::NaiveDate::from_ymd_opt(year, month as u32, day)?
        .and_hms_milli_opt(
            molad_data.hours as u32,
            molad_data.minutes as u32,
            seconds,
            millis,
        )?;

    // Molad is in Jerusalem standard time (GMT+2)
    let jerusalem_offset = chrono::FixedOffset::east_opt(2 * 3600)?;
    let datetime_jerusalem = jerusalem_offset
        .from_local_datetime(&naive_datetime)
        .single()?;

    // Subtract local mean time offset (20.94 minutes = 1256.4 seconds)
    // Longitude of Har Habayis: 35.2354°
    // 35.2354° away from 35° (GMT+2 +  20 minutes) = 0.2354° = ~0.94 minutes
    // Total: 20 minutes 56.496 seconds ≈ 1256.496 seconds
    Some(datetime_jerusalem.to_utc() - chrono::Duration::milliseconds(1256496))
}
