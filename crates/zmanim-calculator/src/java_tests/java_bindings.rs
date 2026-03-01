//! Implementations of the core traits which delegate to the Java implementation.
//! This serves as the base of all our interop tests.
#![allow(unused)]
use chrono::{DateTime, Duration, TimeZone};
use j4rs::{Instance, InvocationArg, Jvm, Null};

use crate::{prelude::*, presets::ZmanPresetLike};

pub struct JavaTimeAndPlace {
    pub geolocation: Instance,
    pub calendar: Instance,
    pub location: Location<chrono_tz::Tz>,
    pub date_time: DateTime<chrono_tz::Tz>,
}
impl JavaTimeAndPlace {
    pub fn new(
        jvm: &Jvm,
        location: &Location<chrono_tz::Tz>,
        date_time: &DateTime<chrono_tz::Tz>,
    ) -> Option<Self> {
        let geolocation = geolocation_to_java_geolocation(jvm, location, date_time)?;
        let calendar = dt_to_java_calendar(jvm, date_time, date_time.timezone().name())?;
        Some(Self {
            geolocation,
            calendar,
            location: *location,
            date_time: *date_time,
        })
    }
}

/// Converts a Rust DateTime to a Java Date instance.
pub fn dt_to_java_date(jvm: &Jvm, date: &DateTime<chrono_tz::Tz>) -> Instance {
    jvm.create_instance(
        "java.util.Date",
        &[InvocationArg::try_from(date.timestamp_millis())
            .unwrap()
            .into_primitive()
            .unwrap()],
    )
    .unwrap()
}
/// Converts a Rust timezone to a Java TimeZone instance.
///
/// Returns None if Java cannot find a matching timezone (falls back to GMT).
pub fn tz_to_java_timezone(jvm: &Jvm, timezone_id: &str) -> Instance {
    jvm.invoke_static(
        "com.ibm.icu.util.TimeZone",
        "getTimeZone",
        &[InvocationArg::try_from(timezone_id).unwrap()],
    )
    .unwrap()
}
/// Converts a Rust DateTime to a Java Calendar instance.
///
/// Returns None if the timezone cannot be converted to Java.
pub fn dt_to_java_calendar<Tz: TimeZone>(
    jvm: &Jvm,
    date: &DateTime<Tz>,
    timezone_id: &str,
) -> Option<Instance> {
    let java_timezone = tz_to_java_timezone(jvm, timezone_id);
    let java_calendar = jvm
        .invoke_static(
            "com.ibm.icu.util.Calendar",
            "getInstance",
            InvocationArg::empty(),
        )
        .unwrap();
    jvm.invoke(
        &java_calendar,
        "setTimeZone",
        &[InvocationArg::from(java_timezone)],
    )
    .unwrap();
    jvm.invoke(
        &java_calendar,
        "setTimeInMillis",
        &[InvocationArg::try_from(date.timestamp_millis())
            .unwrap()
            .into_primitive()
            .unwrap()],
    )
    .unwrap();

    Some(java_calendar)
}

pub struct JavaZmanimCalendar<'a> {
    pub jvm: &'a Jvm,
    pub instance: Instance,
    pub location: Location<chrono_tz::Tz>,
    pub date_time: DateTime<chrono_tz::Tz>,
}
impl<'a> JavaZmanimCalendar<'a> {
    fn get_java_date_millis(&self, date_instance: &Instance) -> Option<i64> {
        //check for null
        let is_null = self
            .jvm
            .check_equals(
                date_instance,
                InvocationArg::try_from(Null::Of("java.util.Date")).unwrap(),
            )
            .unwrap();
        if is_null {
            return None;
        }
        let millis_result = self
            .jvm
            .invoke(date_instance, "getTime", InvocationArg::empty());
        if millis_result.is_err() {
            return None;
        }
        let millis = self.jvm.to_rust::<i64>(millis_result.unwrap()).ok()?;
        Some(millis)
    }

    fn java_date_to_rust_datetime(
        &self,
        date_instance: &Instance,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let millis = self.get_java_date_millis(date_instance)?;
        Some(
            self.date_time
                .timezone()
                .timestamp_millis_opt(millis)
                .unwrap(),
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        jvm: &'a Jvm,
        java_time_and_place: JavaTimeAndPlace,
        candle_lighting_offset: Duration,
        use_astronomical_chatzos_for_other_zmanim: bool,
        ateret_torah_sunset_offset: Duration,
    ) -> Option<Self> {
        let java_zmanim_calendar = jvm
            .create_instance(
                "com.kosherjava.zmanim.ComplexZmanimCalendar",
                &[InvocationArg::from(java_time_and_place.geolocation)],
            )
            .ok()?;
        jvm.invoke(
            &java_zmanim_calendar,
            "setCalendar",
            &[InvocationArg::from(java_time_and_place.calendar)],
        )
        .ok()?;

        jvm.invoke(
            &java_zmanim_calendar,
            "setUseElevation",
            &[InvocationArg::try_from(true)
                .unwrap()
                .into_primitive()
                .unwrap()],
        )
        .ok()?;

        // We test SolarTransit against GetChatzos. This configures the calendar to use solar transit/astronomical midday.
        jvm.invoke(
            &java_zmanim_calendar,
            "setUseAstronomicalChatzos",
            &[InvocationArg::try_from(true)
                .unwrap()
                .into_primitive()
                .unwrap()],
        )
        .ok()?;
        jvm.invoke(
            &java_zmanim_calendar,
            "setUseAstronomicalChatzosForOtherZmanim",
            &[
                InvocationArg::try_from(use_astronomical_chatzos_for_other_zmanim)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
            ],
        )
        .ok()?;
        jvm.invoke(
            &java_zmanim_calendar,
            "setAteretTorahSunsetOffset",
            &[
                InvocationArg::try_from(ateret_torah_sunset_offset.as_seconds_f64() / 60.0)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
            ],
        )
        .ok()?;

        jvm.invoke(
            &java_zmanim_calendar,
            "setCandleLightingOffset",
            &[
                InvocationArg::try_from(candle_lighting_offset.as_seconds_f64() / 60.0)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
            ],
        )
        .ok()?;

        Some(Self {
            jvm,
            instance: java_zmanim_calendar,
            #[allow(clippy::clone_on_copy)]
            location: java_time_and_place.location.clone(),
            date_time: java_time_and_place.date_time,
        })
    }

    fn get_java_duration_millis(&self, duration_instance: Instance) -> Option<i64> {
        let millis = self.jvm.to_rust::<i64>(duration_instance).ok()?;
        // DIFF: Java returns Long.MIN_VALUE (-9223372036854775808) to indicate null/None
        if millis == -9223372036854775808i64 {
            None
        } else {
            Some(millis)
        }
    }

    pub fn get_sunrise(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getSunrise", InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    pub fn get_sea_level_sunrise(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getSeaLevelSunrise", InvocationArg::empty())
            .ok()?;

        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_begin_civil_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getBeginCivilTwilight",
                InvocationArg::empty(),
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_begin_nautical_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getBeginNauticalTwilight",
                InvocationArg::empty(),
            )
            .unwrap();
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_begin_astronomical_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getBeginAstronomicalTwilight",
                InvocationArg::empty(),
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sunset(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getSunset", InvocationArg::empty())
            .ok()?;

        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sea_level_sunset(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getSeaLevelSunset", InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_end_civil_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getEndCivilTwilight",
                InvocationArg::empty(),
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_end_nautical_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getEndNauticalTwilight",
                InvocationArg::empty(),
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_end_astronomical_twilight(&self) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getEndAstronomicalTwilight",
                InvocationArg::empty(),
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sunrise_offset_by_degrees(&self, offset_zenith: f64) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSunriseOffsetByDegrees",
                &[InvocationArg::try_from(offset_zenith)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sunset_offset_by_degrees(&self, offset_zenith: f64) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSunsetOffsetByDegrees",
                &[InvocationArg::try_from(offset_zenith)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_temporal_hour(&self) -> Option<Duration> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getTemporalHour", InvocationArg::empty())
            .ok()?;
        let millis = self.jvm.to_rust::<i64>(java_result).ok()?;
        // DIFF: Java returns Long.MIN_VALUE (-9223372036854775808) to indicate null/None
        if millis == -9223372036854775808i64 {
            None
        } else {
            Some(Duration::milliseconds(millis))
        }
    }

    fn get_temporal_hour_from_times(
        &self,
        start_of_day: &DateTime<chrono_tz::Tz>,
        end_of_day: &DateTime<chrono_tz::Tz>,
    ) -> Option<Duration> {
        let java_start = self
            .jvm
            .create_instance(
                "java.util.Date",
                &[InvocationArg::try_from(start_of_day.timestamp_millis())
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .ok()?;
        let java_end = self
            .jvm
            .create_instance(
                "java.util.Date",
                &[InvocationArg::try_from(end_of_day.timestamp_millis())
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .ok()?;
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getTemporalHour",
                &[
                    InvocationArg::from(java_start),
                    InvocationArg::from(java_end),
                ],
            )
            .ok()?;
        let millis = self.jvm.to_rust::<i64>(java_result).ok()?;
        // DIFF: Java returns Long.MIN_VALUE (-9223372036854775808) to indicate null/None
        if millis == -9223372036854775808i64 {
            None
        } else {
            Some(Duration::milliseconds(millis))
        }
    }

    fn get_sun_transit_from_times(
        &self,
        start_of_day: &DateTime<chrono_tz::Tz>,
        end_of_day: &DateTime<chrono_tz::Tz>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = dt_to_java_date(self.jvm, start_of_day);
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSunTransit",
                &[
                    InvocationArg::from(java_start),
                    InvocationArg::from(java_end),
                ],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_local_mean_time(&self, hours: f64) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getLocalMeanTime",
                &[InvocationArg::try_from(hours)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }
    fn get_percent_of_shaah_zmanis_from_degrees(&self, degrees: f64, sunset: bool) -> Option<f64> {
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getPercentOfShaahZmanisFromDegrees",
                &[
                    InvocationArg::try_from(degrees)
                        .unwrap()
                        .into_primitive()
                        .unwrap(),
                    InvocationArg::try_from(sunset)
                        .unwrap()
                        .into_primitive()
                        .unwrap(),
                ],
            )
            .ok()?;
        let result = self.jvm.to_rust::<f64>(java_result).ok()?;
        if result == 5e-324 {
            None
        } else {
            Some(result)
        }
    }

    fn get_shaah_zmanis_gra(&self) -> Option<Duration> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getShaahZmanisGra", InvocationArg::empty())
            .ok()?;
        self.get_java_duration_millis(java_result)
            .map(Duration::milliseconds)
    }

    fn get_shaah_zmanis_mga(&self) -> Option<Duration> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getShaahZmanisMGA", InvocationArg::empty())
            .ok()?;
        self.get_java_duration_millis(java_result)
            .map(Duration::milliseconds)
    }

    fn get_half_day_based_zman_from_times(
        &self,
        start_of_half_day: &DateTime<chrono_tz::Tz>,
        end_of_half_day: &DateTime<chrono_tz::Tz>,
        hours: f64,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = dt_to_java_date(self.jvm, start_of_half_day);
        let java_end = dt_to_java_date(self.jvm, end_of_half_day);
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getHalfDayBasedZman",
                &[
                    InvocationArg::from(java_start),
                    InvocationArg::from(java_end),
                    InvocationArg::try_from(hours)
                        .unwrap()
                        .into_primitive()
                        .unwrap(),
                ],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_half_day_based_shaah_zmanis_from_times(
        &self,
        start_of_half_day: &DateTime<chrono_tz::Tz>,
        end_of_half_day: &DateTime<chrono_tz::Tz>,
    ) -> Option<Duration> {
        let java_start = dt_to_java_date(self.jvm, start_of_half_day);
        let java_end = dt_to_java_date(self.jvm, end_of_half_day);
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getHalfDayBasedShaahZmanis",
                &[
                    InvocationArg::from(java_start),
                    InvocationArg::from(java_end),
                ],
            )
            .ok()?;
        self.get_java_duration_millis(java_result)
            .map(Duration::milliseconds)
    }

    fn get_shaah_zmanis_based_zman_from_times(
        &self,
        start_of_day: &DateTime<chrono_tz::Tz>,
        end_of_day: &DateTime<chrono_tz::Tz>,
        hours: f64,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = dt_to_java_date(self.jvm, start_of_day);
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getShaahZmanisBasedZman",
                &[
                    InvocationArg::from(java_start),
                    InvocationArg::from(java_end),
                    InvocationArg::try_from(hours)
                        .unwrap()
                        .into_primitive()
                        .unwrap(),
                ],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sof_zman_shma_from_times(
        &self,
        start_of_day: &DateTime<chrono_tz::Tz>,
        end_of_day: Option<&DateTime<chrono_tz::Tz>>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = dt_to_java_date(self.jvm, start_of_day);
        let java_end = if let Some(end_of_day) = end_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, end_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSofZmanShma",
                &[InvocationArg::from(java_start), java_end, java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_mincha_ketana_from_times(
        &self,
        start_of_day: Option<&DateTime<chrono_tz::Tz>>,
        end_of_day: &DateTime<chrono_tz::Tz>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = if let Some(start_of_day) = start_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, start_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getMinchaKetana",
                &[java_start, InvocationArg::from(java_end), java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sof_zman_tfila_from_times(
        &self,
        start_of_day: &DateTime<chrono_tz::Tz>,
        end_of_day: Option<&DateTime<chrono_tz::Tz>>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = dt_to_java_date(self.jvm, start_of_day);
        let java_end = if let Some(end_of_day) = end_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, end_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSofZmanTfila",
                &[InvocationArg::from(java_start), java_end, java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_mincha_gedola_from_times(
        &self,
        start_of_day: Option<&DateTime<chrono_tz::Tz>>,
        end_of_day: &DateTime<chrono_tz::Tz>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = if let Some(start_of_day) = start_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, start_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getMinchaGedola",
                &[java_start, InvocationArg::from(java_end), java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_plag_hamincha_from_times(
        &self,
        start_of_day: Option<&DateTime<chrono_tz::Tz>>,
        end_of_day: &DateTime<chrono_tz::Tz>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = if let Some(start_of_day) = start_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, start_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getPlagHamincha",
                &[java_start, InvocationArg::from(java_end), java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_samuch_le_mincha_ketana_from_times(
        &self,
        start_of_day: Option<&DateTime<chrono_tz::Tz>>,
        end_of_day: &DateTime<chrono_tz::Tz>,
        synchronous: bool,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_start = if let Some(start_of_day) = start_of_day {
            InvocationArg::from(dt_to_java_date(self.jvm, start_of_day))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_end = dt_to_java_date(self.jvm, end_of_day);
        let java_synchronous = InvocationArg::try_from(synchronous)
            .unwrap()
            .into_primitive()
            .unwrap();
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSamuchLeMinchaKetana",
                &[java_start, InvocationArg::from(java_end), java_synchronous],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sof_zman_kidush_levana_15_days_from_times(
        &self,
        alos: Option<&DateTime<chrono_tz::Tz>>,
        tzais: Option<&DateTime<chrono_tz::Tz>>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_alos = if let Some(alos) = alos {
            InvocationArg::from(dt_to_java_date(self.jvm, alos))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_tzais = if let Some(tzais) = tzais {
            InvocationArg::from(dt_to_java_date(self.jvm, tzais))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSofZmanKidushLevana15Days",
                &[java_alos, java_tzais],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_sof_zman_kidush_levana_between_moldos_from_times(
        &self,
        alos: Option<&DateTime<chrono_tz::Tz>>,
        tzais: Option<&DateTime<chrono_tz::Tz>>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_alos = if let Some(alos) = alos {
            InvocationArg::from(dt_to_java_date(self.jvm, alos))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_tzais = if let Some(tzais) = tzais {
            InvocationArg::from(dt_to_java_date(self.jvm, tzais))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getSofZmanKidushLevanaBetweenMoldos",
                &[java_alos, java_tzais],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_tchilas_zman_kidush_levana_3_days_from_times(
        &self,
        alos: Option<&DateTime<chrono_tz::Tz>>,
        tzais: Option<&DateTime<chrono_tz::Tz>>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_alos = if let Some(alos) = alos {
            InvocationArg::from(dt_to_java_date(self.jvm, alos))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_tzais = if let Some(tzais) = tzais {
            InvocationArg::from(dt_to_java_date(self.jvm, tzais))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getTchilasZmanKidushLevana3Days",
                &[java_alos, java_tzais],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn get_tchilas_zman_kidush_levana_7_days_from_times(
        &self,
        alos: Option<&DateTime<chrono_tz::Tz>>,
        tzais: Option<&DateTime<chrono_tz::Tz>>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_alos = if let Some(alos) = alos {
            InvocationArg::from(dt_to_java_date(self.jvm, alos))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_tzais = if let Some(tzais) = tzais {
            InvocationArg::from(dt_to_java_date(self.jvm, tzais))
        } else {
            InvocationArg::try_from(Null::Of("java.util.Date")).unwrap()
        };
        let java_result = self
            .jvm
            .invoke(
                &self.instance,
                "getTchilasZmanKidushLevana7Days",
                &[java_alos, java_tzais],
            )
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    pub fn get_zman(
        &self,
        zman: &dyn ZmanPresetLike<chrono_tz::Tz>,
    ) -> Option<DateTime<chrono_tz::Tz>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, zman.name(), InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }
}

/// Converts a Rust GeoLocation to a Java GeoLocation instance.
///
/// Returns None if the timezone cannot be converted to Java.
pub fn geolocation_to_java_geolocation(
    jvm: &Jvm,
    location: &Location<chrono_tz::Tz>,
    date_time: &DateTime<chrono_tz::Tz>,
) -> Option<Instance> {
    let timezone_id = date_time.timezone().name();
    let java_timezone = tz_to_java_timezone(jvm, timezone_id);

    let instance = jvm
        .create_instance(
            "com.kosherjava.zmanim.util.GeoLocation",
            &[
                InvocationArg::try_from("Name").unwrap(),
                InvocationArg::try_from(location.latitude)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
                InvocationArg::try_from(location.longitude)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
                InvocationArg::try_from(location.elevation)
                    .unwrap()
                    .into_primitive()
                    .unwrap(),
                InvocationArg::from(java_timezone),
            ],
        )
        .ok();
    // DIFF: Java will throw an exception if it is unable to create a GeoLocation
    // However we will return None if this is the case
    instance.as_ref()?;
    let instance = instance.unwrap();
    Some(instance)
}
