//! Implementations of the core traits which delegate to the Java implementation.
//! This serves as the base of all our interop tests.
use chrono::{DateTime, Duration, TimeZone};
use j4rs::{Instance, InvocationArg, Jvm, Null};

use crate::{Location, Zman};

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
        let calendar = dt_to_java_calendar(jvm, &date_time, date_time.timezone().name())?;
        Some(Self {
            geolocation,
            calendar,
            location: location.clone(),
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

// impl<'a> JavaGeoLocation<'a> {
//     pub fn new(jvm: &'a Jvm, latitude: f64, longitude: f64, elevation: f64, timezone_id: &'a str) -> Option<Self> {
//         let java_timezone = tz_to_java_timezone(jvm, timezone_id);

//         let instance = jvm
//             .create_instance(
//                 "com.kosherjava.zmanim.util.GeoLocation",
//                 &[
//                     InvocationArg::try_from("Name").unwrap(),
//                     InvocationArg::try_from(latitude).unwrap().into_primitive().unwrap(),
//                     InvocationArg::try_from(longitude).unwrap().into_primitive().unwrap(),
//                     InvocationArg::try_from(elevation).unwrap().into_primitive().unwrap(),
//                     InvocationArg::from(java_timezone),
//                 ],
//             )
//             .ok();
//         // DIFF: Java will throw an exception if it is unable to create a GeoLocation
//         // However we will return None if this is the case
//         instance.as_ref()?;
//         let instance = instance.unwrap();
//         Some(Self {
//             jvm,
//             instance,
//             timezone_id,
//         })
//     }
// }

// impl<'a> GeoLocationTrait for JavaGeoLocation<'a> {
//     fn get_latitude(&self) -> f64 {
//         self.jvm
//             .to_rust::<f64>(
//                 self.jvm
//                     .invoke(&self.instance, "getLatitude", InvocationArg::empty())
//                     .unwrap(),
//             )
//             .unwrap()
//     }

//     fn get_longitude(&self) -> f64 {
//         self.jvm
//             .to_rust::<f64>(
//                 self.jvm
//                     .invoke(&self.instance, "getLongitude", InvocationArg::empty())
//                     .unwrap(),
//             )
//             .unwrap()
//     }

//     fn get_elevation(&self) -> f64 {
//         self.jvm
//             .to_rust::<f64>(
//                 self.jvm
//                     .invoke(&self.instance, "getElevation", InvocationArg::empty())
//                     .unwrap(),
//             )
//             .unwrap()
//     }

//     fn get_rhumb_line_distance(&self, location: &JavaGeoLocation<'_>) -> f64 {
//         let cloned_instance = self.jvm.clone_instance(&location.instance).unwrap();
//         self.jvm
//             .to_rust::<f64>(
//                 self.jvm
//                     .invoke(
//                         &self.instance,
//                         "getRhumbLineDistance",
//                         &[InvocationArg::from(cloned_instance)],
//                     )
//                     .unwrap(),
//             )
//             .unwrap()
//     }

//     fn get_rhumb_line_bearing(&self, location: &JavaGeoLocation<'_>) -> f64 {
//         let cloned_instance = self.jvm.clone_instance(&location.instance).unwrap();
//         self.jvm
//             .to_rust::<f64>(
//                 self.jvm
//                     .invoke(
//                         &self.instance,
//                         "getRhumbLineBearing",
//                         &[InvocationArg::from(cloned_instance)],
//                     )
//                     .unwrap(),
//             )
//             .unwrap()
//     }

//     fn get_geodesic_initial_bearing(&self, location: &JavaGeoLocation<'_>) -> Option<f64> {
//         let cloned_instance = self.jvm.clone_instance(&location.instance).unwrap();
//         let result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "getGeodesicInitialBearing",
//                 &[InvocationArg::from(cloned_instance)],
//             )
//             .unwrap();

//         let result = self.jvm.to_rust::<f64>(result).ok();
//         result.filter(|&value| !value.is_nan())
//     }

//     fn get_geodesic_final_bearing(&self, location: &JavaGeoLocation<'_>) -> Option<f64> {
//         let cloned_instance = self.jvm.clone_instance(&location.instance).unwrap();
//         let result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "getGeodesicFinalBearing",
//                 &[InvocationArg::from(cloned_instance)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<f64>(result).ok().filter(|&value| !value.is_nan())
//     }

//     fn get_geodesic_distance(&self, location: &JavaGeoLocation<'_>) -> Option<f64> {
//         let cloned_instance = self.jvm.clone_instance(&location.instance).unwrap();
//         let result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "getGeodesicDistance",
//                 &[InvocationArg::from(cloned_instance)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<f64>(result).ok().filter(|&value| !value.is_nan())
//     }

//     fn get_local_mean_time_offset<Tz: chrono::TimeZone>(&self, date: &chrono::DateTime<chrono_tz::Tz>) -> chrono::Duration {
//         let java_date = dt_to_java_calendar(self.jvm, date, self.timezone_id).unwrap();
//         let result = self
//             .jvm
//             .to_rust::<i64>(
//                 self.jvm
//                     .invoke(
//                         &self.instance,
//                         "getLocalMeanTimeOffset",
//                         &[InvocationArg::from(java_date)],
//                     )
//                     .unwrap(),
//             )
//             .unwrap();
//         Duration::milliseconds(result)
//     }

//     fn get_antimeridian_adjustment<Tz: chrono::TimeZone>(&self, date: &chrono::DateTime<chrono_tz::Tz>) -> i8 {
//         let java_date = dt_to_java_calendar(self.jvm, date, self.timezone_id).unwrap();
//         self.jvm
//             .to_rust::<i32>(
//                 self.jvm
//                     .invoke(
//                         &self.instance,
//                         "getAntimeridianAdjustment",
//                         &[InvocationArg::from(java_date)],
//                     )
//                     .unwrap(),
//             )
//             .unwrap() as i8
//     }
// }

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
        use_astronomical_chatzos: bool,
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
        jvm.invoke(
            &java_zmanim_calendar,
            "setUseAstronomicalChatzos",
            &[InvocationArg::try_from(use_astronomical_chatzos)
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

    pub fn get_zman(&self, zman: &Zman) -> Option<DateTime<chrono_tz::Tz>> {
        let method_name = match zman {
            Zman::AlosHashachar => "getAlosHashachar",
            Zman::Alos72 => "getAlos72",
            Zman::Chatzos => "getChatzos",
            Zman::ChatzosAsHalfDay => "getChatzosAsHalfDay",
            Zman::MinchaGedola => "getMinchaGedola",
            Zman::MinchaKetana => "getMinchaKetana",
            Zman::PlagHamincha => "getPlagHamincha",
            Zman::SofZmanShmaGRA => "getSofZmanShmaGRA",
            Zman::SofZmanShmaMGA => "getSofZmanShmaMGA",
            Zman::SofZmanTfilaGRA => "getSofZmanTfilaGRA",
            Zman::SofZmanTfilaMGA => "getSofZmanTfilaMGA",
            Zman::Tzais => "getTzais",
            Zman::Tzais72 => "getTzais72",
            Zman::CandleLighting => "getCandleLighting",
            Zman::Sunrise => "getSunrise",
            Zman::Sunset => "getSunset",
            Zman::SeaLevelSunrise => "getSeaLevelSunrise",
            Zman::SeaLevelSunset => "getSeaLevelSunset",
        };
        let java_result = self
            .jvm
            .invoke(&self.instance, method_name, InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }
}

// /// Very sketchy function to convert a JewishCalendarTrait to a JavaJewishCalendar instance
// /// This is acceptable for testing purposes
// fn java_jewish_calendar_trait_to_java_instance<T: JewishCalendarTrait>(jvm: &Jvm, jewish_calendar: &T) -> Instance {
//     unsafe {
//         let java_calendar = &*(jewish_calendar as *const T as *const JavaJewishCalendar);
//         jvm.clone_instance(&java_calendar.instance).ok().unwrap()
//     }
// }

// impl<'a> TefilaRulesTrait<JavaJewishCalendar<'a>> for JavaTefilaRules<'a> {
//     fn is_tachanun_recited_shacharis(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isTachanunRecitedShacharis",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_tachanun_recited_mincha(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> Option<bool> {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isTachanunRecitedMincha",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         let result: bool = self.jvm.to_rust(java_result).unwrap();
//         Some(result)
//     }

//     fn is_hallel_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "isHallelRecited", &[InvocationArg::from(java_calendar)])
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_hallel_shalem_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isHallelShalemRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_al_hanissim_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isAlHanissimRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_yaaleh_veyavo_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isYaalehVeyavoRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_mizmor_lesoda_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isMizmorLesodaRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_vesein_tal_umatar_start_date(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isVeseinTalUmatarStartDate",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_vesein_tal_umatar_starting_tonight(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isVeseinTalUmatarStartingTonight",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_vesein_tal_umatar_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isVeseinTalUmatarRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_vesein_beracha_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isVeseinBerachaRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_mashiv_haruach_start_date(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isMashivHaruachStartDate",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_mashiv_haruach_end_date(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> bool {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isMashivHaruachEndDate",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_mashiv_haruach_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> Option<bool> {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isMashivHaruachRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         let result: bool = self.jvm.to_rust(java_result).unwrap();
//         Some(result)
//     }

//     fn is_morid_hatal_recited(&self, jewish_calendar: &JavaJewishCalendar<'a>) -> Option<bool> {
//         let java_calendar = java_jewish_calendar_trait_to_java_instance(self.jvm, jewish_calendar);
//         let java_result = self
//             .jvm
//             .invoke(
//                 &self.instance,
//                 "isMoridHatalRecited",
//                 &[InvocationArg::from(java_calendar)],
//             )
//             .unwrap();
//         let result: bool = self.jvm.to_rust(java_result).unwrap();
//         Some(result)
//     }
// }

// pub struct JavaJewishCalendar<'a> {
//     pub jvm: &'a Jvm,
//     pub instance: Instance,
//     pub in_israel: bool,
//     pub is_mukaf_choma: bool,
//     pub use_modern_holidays: bool,
// }

// impl<'a> JavaJewishCalendar<'a> {
//     pub fn from_gregorian_date(
//         jvm: &'a Jvm,
//         year: i32,
//         month: i32,
//         day: i32,
//         in_israel: bool,
//         is_mukaf_choma: bool,
//         use_modern_holidays: bool,
//     ) -> Option<Self> {
//         let year_arg = InvocationArg::try_from(year).unwrap().into_primitive().unwrap();
//         let month_arg = InvocationArg::try_from(month).unwrap().into_primitive().unwrap();
//         let day_arg = InvocationArg::try_from(day).unwrap().into_primitive().unwrap();
//         let local_date = jvm
//             .invoke_static("java.time.LocalDate", "of", &[year_arg, month_arg, day_arg])
//             .unwrap();
//         let instance = jvm
//             .create_instance(
//                 "com.kosherjava.zmanim.hebrewcalendar.JewishCalendar",
//                 &[InvocationArg::from(local_date)],
//             )
//             .ok();
//         instance.as_ref()?;
//         let instance = instance.unwrap();
//         let self_ = Self {
//             jvm,
//             instance,
//             in_israel,
//             is_mukaf_choma,
//             use_modern_holidays,
//         };
//         self_.set_in_israel(in_israel);
//         self_.set_is_mukaf_choma(is_mukaf_choma);
//         self_.set_use_modern_holidays(use_modern_holidays);
//         Some(self_)
//     }
//     pub fn from_jewish_date(
//         jvm: &'a Jvm,
//         year: i32,
//         month: JewishMonth,
//         day: i32,
//         in_israel: bool,
//         is_mukaf_choma: bool,
//         use_modern_holidays: bool,
//     ) -> Option<Self> {
//         let year_arg = InvocationArg::try_from(year).unwrap().into_primitive().unwrap();
//         let month_arg = InvocationArg::try_from(month as i32).unwrap().into_primitive().unwrap();
//         let day_arg = InvocationArg::try_from(day).unwrap().into_primitive().unwrap();
//         let instance = jvm
//             .create_instance(
//                 "com.kosherjava.zmanim.hebrewcalendar.JewishCalendar",
//                 &[year_arg, month_arg, day_arg],
//             )
//             .ok();
//         instance.as_ref()?;
//         let instance = instance.unwrap();
//         let self_ = Self {
//             jvm,
//             instance,
//             in_israel,
//             is_mukaf_choma,
//             use_modern_holidays,
//         };
//         self_.set_in_israel(in_israel);
//         self_.set_is_mukaf_choma(is_mukaf_choma);
//         self_.set_use_modern_holidays(use_modern_holidays);
//         Some(self_)
//     }
//     fn set_is_mukaf_choma(&self, is_mukaf_choma: bool) {
//         self.jvm
//             .invoke(
//                 &self.instance,
//                 "setIsMukafChoma",
//                 &[InvocationArg::try_from(is_mukaf_choma)
//                     .unwrap()
//                     .into_primitive()
//                     .unwrap()],
//             )
//             .unwrap();
//     }

//     fn invoke_bool(&self, method: &str) -> bool {
//         let java_result = self.jvm.invoke(&self.instance, method, InvocationArg::empty()).unwrap();
//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn invoke_i64(&self, method: &str) -> i64 {
//         let java_result = self.jvm.invoke(&self.instance, method, InvocationArg::empty()).unwrap();
//         self.jvm.to_rust::<i64>(java_result).unwrap()
//     }
//     #[allow(unused)]
//     pub fn set_in_israel(&self, in_israel: bool) {
//         self.jvm
//             .invoke(
//                 &self.instance,
//                 "setInIsrael",
//                 &[InvocationArg::try_from(in_israel).unwrap().into_primitive().unwrap()],
//             )
//             .unwrap();
//     }

//     #[allow(unused)]
//     pub fn set_mukaf_choma(&self, mukaf_choma: bool) {
//         self.jvm
//             .invoke(
//                 &self.instance,
//                 "setIsMukafChoma",
//                 &[InvocationArg::try_from(mukaf_choma).unwrap().into_primitive().unwrap()],
//             )
//             .unwrap();
//     }

//     #[allow(unused)]
//     pub fn set_use_modern_holidays(&self, use_modern_holidays: bool) {
//         self.jvm
//             .invoke(
//                 &self.instance,
//                 "setUseModernHolidays",
//                 &[InvocationArg::try_from(use_modern_holidays)
//                     .unwrap()
//                     .into_primitive()
//                     .unwrap()],
//             )
//             .unwrap();
//     }

//     fn java_date_to_rust_datetime(&self, java_date: &Instance) -> Option<DateTime<Utc>> {
//         let is_null = self
//             .jvm
//             .check_equals(java_date, InvocationArg::try_from(Null::Of("java.util.Date")).unwrap())
//             .unwrap();
//         if is_null {
//             return None;
//         }

//         let millis = self
//             .jvm
//             .to_rust::<i64>(self.jvm.invoke(java_date, "getTime", InvocationArg::empty()).unwrap())
//             .unwrap();

//         DateTime::<Utc>::from_timestamp_millis(millis)
//     }

//     fn invoke_date(&self, method: &str) -> Option<DateTime<Utc>> {
//         let java_result = self.jvm.invoke(&self.instance, method, InvocationArg::empty()).ok()?;
//         self.java_date_to_rust_datetime(&java_result)
//     }

//     fn parsha_from_java(&self, method: &str) -> Option<Parsha> {
//         let java_parsha = self.jvm.invoke(&self.instance, method, InvocationArg::empty()).ok()?;
//         let ordinal_instance = self.jvm.invoke(&java_parsha, "ordinal", InvocationArg::empty()).ok()?;
//         let ordinal = self.jvm.to_rust::<i32>(ordinal_instance).ok()?;
//         if ordinal == 0 {
//             None
//         } else {
//             Parsha::try_from((ordinal - 1) as u8).ok()
//         }
//     }
// }

// impl<'a> JewishCalendarTrait for JavaJewishCalendar<'a> {
//     fn get_jewish_year(&self) -> i32 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getJewishYear", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<i32>(java_result).unwrap()
//     }

//     fn get_jewish_month(&self) -> JewishMonth {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getJewishMonth", InvocationArg::empty())
//             .unwrap();
//         let java_result = self.jvm.to_rust::<u8>(java_result).unwrap();
//         java_result.try_into().unwrap()
//     }
//     fn get_jewish_day_of_month(&self) -> u8 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getJewishDayOfMonth", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<u8>(java_result).unwrap()
//     }
//     fn get_gregorian_year(&self) -> i32 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getGregorianYear", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<i32>(java_result).unwrap()
//     }
//     fn get_gregorian_month(&self) -> u8 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getGregorianMonth", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<u8>(java_result).unwrap()
//     }

//     fn get_gregorian_day_of_month(&self) -> u8 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getGregorianDayOfMonth", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<u8>(java_result).unwrap()
//     }

//     fn get_day_of_week(&self) -> Weekday {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDayOfWeek", InvocationArg::empty())
//             .unwrap();
//         let java_result = self.jvm.to_rust::<u8>(java_result).unwrap();
//         match java_result {
//             1 => Weekday::Sun,
//             2 => Weekday::Mon,
//             3 => Weekday::Tue,
//             4 => Weekday::Wed,
//             5 => Weekday::Thu,
//             6 => Weekday::Fri,
//             7 => Weekday::Sat,
//             _ => unreachable!(),
//         }
//     }

//     fn is_jewish_leap_year(&self) -> bool {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "isJewishLeapYear", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn get_days_in_jewish_year(&self) -> i32 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDaysInJewishYear", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<i32>(java_result).unwrap()
//     }

//     fn get_days_in_jewish_month(&self) -> u8 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDaysInJewishMonth", InvocationArg::empty())
//             .unwrap();
//         self.jvm.to_rust::<u8>(java_result).unwrap()
//     }

//     fn is_cheshvan_long(&self) -> bool {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "isCheshvanLong", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn is_kislev_short(&self) -> bool {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "isKislevShort", InvocationArg::empty())
//             .unwrap();

//         self.jvm.to_rust::<bool>(java_result).unwrap()
//     }

//     fn get_cheshvan_kislev_kviah(&self) -> crate::constants::YearLengthType {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getCheshvanKislevKviah", InvocationArg::empty())
//             .unwrap();
//         let java_result = self.jvm.to_rust::<u8>(java_result).unwrap();
//         java_result.try_into().unwrap()
//     }

//     fn get_days_since_start_of_jewish_year(&self) -> i32 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDaysSinceStartOfJewishYear", InvocationArg::empty())
//             .unwrap();
//         self.jvm.to_rust::<i32>(java_result).unwrap()
//     }

//     fn get_chalakim_since_molad_tohu(&self) -> i64 {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getChalakimSinceMoladTohu", InvocationArg::empty())
//             .unwrap();
//         self.jvm.to_rust::<i64>(java_result).unwrap()
//     }
//     #[allow(refining_impl_trait)]
//     fn get_molad_as_calendar(&self) -> Option<JavaJewishCalendar<'a>> {
//         let java_molad = self
//             .jvm
//             .invoke(&self.instance, "getMolad", InvocationArg::empty())
//             .unwrap();
//         let is_null = self
//             .jvm
//             .check_equals(
//                 &java_molad,
//                 InvocationArg::try_from(Null::Of("java.util.Date")).unwrap(),
//             )
//             .unwrap();
//         if is_null {
//             return None;
//         }

//         let java_year = self
//             .jvm
//             .invoke(&java_molad, "getJewishYear", InvocationArg::empty())
//             .unwrap();
//         let java_year = self.jvm.to_rust::<i32>(java_year).unwrap();

//         let java_month = self
//             .jvm
//             .invoke(&java_molad, "getJewishMonth", InvocationArg::empty())
//             .unwrap();
//         let java_month = self.jvm.to_rust::<u8>(java_month).unwrap();

//         let java_day = self
//             .jvm
//             .invoke(&java_molad, "getJewishDayOfMonth", InvocationArg::empty())
//             .unwrap();
//         let java_day = self.jvm.to_rust::<i32>(java_day).unwrap();
//         Some(
//             JavaJewishCalendar::from_jewish_date(
//                 self.jvm,
//                 java_year,
//                 java_month.try_into().unwrap(),
//                 java_day,
//                 self.in_israel,
//                 self.is_mukaf_choma,
//                 self.use_modern_holidays,
//             )
//             .unwrap(),
//         )
//     }

//     fn get_molad(&self) -> Option<MoladData> {
//         let java_molad = self
//             .jvm
//             .invoke(&self.instance, "getMolad", InvocationArg::empty())
//             .unwrap();
//         let is_null = self
//             .jvm
//             .check_equals(
//                 &java_molad,
//                 InvocationArg::try_from(Null::Of("java.util.Date")).unwrap(),
//             )
//             .unwrap();
//         if is_null {
//             return None;
//         }

//         let java_hours = self
//             .jvm
//             .invoke(&java_molad, "getMoladHours", InvocationArg::empty())
//             .unwrap();
//         let java_hours = self.jvm.to_rust::<i64>(java_hours).unwrap();

//         let java_minutes = self
//             .jvm
//             .invoke(&java_molad, "getMoladMinutes", InvocationArg::empty())
//             .unwrap();
//         let java_minutes = self.jvm.to_rust::<i64>(java_minutes).unwrap();

//         let java_chalakim = self
//             .jvm
//             .invoke(&java_molad, "getMoladChalakim", InvocationArg::empty())
//             .unwrap();
//         let java_chalakim = self.jvm.to_rust::<i64>(java_chalakim).unwrap();
//         Some(MoladData {
//             hours: java_hours,
//             minutes: java_minutes,
//             chalakim: java_chalakim,
//         })
//     }
//     fn get_yom_tov_index(&self) -> Option<crate::constants::JewishHoliday> {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getYomTovIndex", InvocationArg::empty())
//             .unwrap();
//         let index = self.jvm.to_rust::<i32>(java_result).unwrap();
//         if index == -1 {
//             None
//         } else {
//             JewishHoliday::try_from(index as u8).ok()
//         }
//     }

//     fn is_yom_tov(&self) -> bool {
//         self.invoke_bool("isYomTov")
//     }

//     fn is_yom_tov_assur_bemelacha(&self) -> bool {
//         self.invoke_bool("isYomTovAssurBemelacha")
//     }

//     fn is_assur_bemelacha(&self) -> bool {
//         self.invoke_bool("isAssurBemelacha")
//     }

//     fn has_candle_lighting(&self) -> bool {
//         self.invoke_bool("hasCandleLighting")
//     }

//     fn is_tomorrow_shabbos_or_yom_tov(&self) -> bool {
//         self.invoke_bool("isTomorrowShabbosOrYomTov")
//     }

//     fn is_erev_yom_tov_sheni(&self) -> bool {
//         self.invoke_bool("isErevYomTovSheni")
//     }

//     fn is_aseres_yemei_teshuva(&self) -> bool {
//         self.invoke_bool("isAseresYemeiTeshuva")
//     }

//     fn is_pesach(&self) -> bool {
//         self.invoke_bool("isPesach")
//     }

//     fn is_chol_hamoed_pesach(&self) -> bool {
//         self.invoke_bool("isCholHamoedPesach")
//     }

//     fn is_shavuos(&self) -> bool {
//         self.invoke_bool("isShavuos")
//     }

//     fn is_rosh_hashana(&self) -> bool {
//         self.invoke_bool("isRoshHashana")
//     }

//     fn is_yom_kippur(&self) -> bool {
//         self.invoke_bool("isYomKippur")
//     }

//     fn is_succos(&self) -> bool {
//         self.invoke_bool("isSuccos")
//     }

//     fn is_hoshana_rabba(&self) -> bool {
//         self.invoke_bool("isHoshanaRabba")
//     }

//     fn is_shemini_atzeres(&self) -> bool {
//         self.invoke_bool("isShminiAtzeres")
//     }

//     fn is_simchas_torah(&self) -> bool {
//         self.invoke_bool("isSimchasTorah")
//     }

//     fn is_chol_hamoed_succos(&self) -> bool {
//         self.invoke_bool("isCholHamoedSuccos")
//     }

//     fn is_chol_hamoed(&self) -> bool {
//         self.invoke_bool("isCholHamoed")
//     }

//     fn is_erev_yom_tov(&self) -> bool {
//         self.invoke_bool("isErevYomTov")
//     }

//     fn is_rosh_chodesh(&self) -> bool {
//         self.invoke_bool("isRoshChodesh")
//     }

//     fn is_isru_chag(&self) -> bool {
//         self.invoke_bool("isIsruChag")
//     }

//     fn is_taanis(&self) -> bool {
//         self.invoke_bool("isTaanis")
//     }

//     fn is_taanis_bechoros(&self) -> bool {
//         self.invoke_bool("isTaanisBechoros")
//     }

//     fn get_day_of_chanukah(&self) -> Option<u8> {
//         let result = self.invoke_i64("getDayOfChanukah");
//         if result == -1 { None } else { Some(result as u8) }
//     }

//     fn is_chanukah(&self) -> bool {
//         self.invoke_bool("isChanukah")
//     }

//     fn is_purim(&self) -> bool {
//         self.invoke_bool("isPurim")
//     }

//     fn get_day_of_omer(&self) -> Option<u8> {
//         let result = self.invoke_i64("getDayOfOmer");
//         if result == -1 { None } else { Some(result as u8) }
//     }

//     fn is_tisha_beav(&self) -> bool {
//         self.invoke_bool("isTishaBav")
//     }

//     fn get_parshah(&self) -> Option<crate::constants::Parsha> {
//         self.parsha_from_java("getParshah")
//     }

//     fn get_daf_yomi_bavli(&self) -> Option<crate::daf::BavliDaf> {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDafYomiBavli", InvocationArg::empty())
//             .ok()?;
//         let is_null = self
//             .jvm
//             .check_equals(
//                 &java_result,
//                 InvocationArg::try_from(Null::Of("com.kosherjava.zmanim.hebrewcalendar.Daf")).unwrap(),
//             )
//             .unwrap();
//         if is_null {
//             return None;
//         }

//         let masechta = self
//             .jvm
//             .invoke(&java_result, "getMasechtaNumber", InvocationArg::empty())
//             .ok()
//             .and_then(|m| self.jvm.to_rust::<i32>(m).ok())?;
//         let daf_index = self
//             .jvm
//             .invoke(&java_result, "getDaf", InvocationArg::empty())
//             .ok()
//             .and_then(|d| self.jvm.to_rust::<i32>(d).ok())?;

//         let tractate = crate::constants::BavliTractate::try_from(masechta as u8).ok()?;

//         Some(BavliDaf {
//             tractate,
//             daf_index: daf_index as i64,
//         })
//     }

//     fn get_daf_yomi_yerushalmi(&self) -> Option<crate::daf::YerushalmiDaf> {
//         let java_result = self
//             .jvm
//             .invoke(&self.instance, "getDafYomiYerushalmi", InvocationArg::empty())
//             .ok()?;
//         let is_null = self
//             .jvm
//             .check_equals(
//                 &java_result,
//                 InvocationArg::try_from(Null::Of("com.kosherjava.zmanim.hebrewcalendar.Daf")).unwrap(),
//             )
//             .unwrap();
//         if is_null {
//             return None;
//         }

//         let masechta = self
//             .jvm
//             .invoke(&java_result, "getMasechtaNumber", InvocationArg::empty())
//             .ok()
//             .and_then(|m| self.jvm.to_rust::<i32>(m).ok())?;

//         // The Java implementation uses 39 (No Daf Today) as a sentinel.
//         if masechta >= 39 {
//             return None;
//         }

//         let daf_index = self
//             .jvm
//             .invoke(&java_result, "getDaf", InvocationArg::empty())
//             .ok()
//             .and_then(|d| self.jvm.to_rust::<i32>(d).ok())?;

//         let tractate = crate::constants::YerushalmiTractate::try_from(masechta as i64).ok()?;

//         Some(YerushalmiDaf {
//             tractate,
//             daf_index: daf_index as i64,
//         })
//     }

//     fn is_birkas_hachamah(&self) -> bool {
//         self.invoke_bool("isBirkasHachamah")
//     }

//     fn is_erev_rosh_chodesh(&self) -> bool {
//         self.invoke_bool("isErevRoshChodesh")
//     }

//     fn is_yom_kippur_katan(&self) -> bool {
//         self.invoke_bool("isYomKippurKatan")
//     }

//     fn is_be_hab(&self) -> bool {
//         self.invoke_bool("isBeHaB")
//     }

//     fn is_machar_chodesh(&self) -> bool {
//         self.invoke_bool("isMacharChodesh")
//     }

//     fn is_shabbos_mevorchim(&self) -> bool {
//         self.invoke_bool("isShabbosMevorchim")
//     }

//     fn get_upcoming_parshah(&self) -> Option<crate::constants::Parsha> {
//         self.parsha_from_java("getUpcomingParshah")
//     }

//     fn get_special_shabbos(&self) -> Option<crate::constants::Parsha> {
//         self.parsha_from_java("getSpecialShabbos")
//     }

//     fn get_molad_as_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//         self.invoke_date("getMoladAsDate")
//     }

//     fn get_tchilaszman_kidush_levana_3_days(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//         self.invoke_date("getTchilasZmanKidushLevana3Days")
//     }

//     fn get_tchilaszman_kidush_levana_7_days(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//         self.invoke_date("getTchilasZmanKidushLevana7Days")
//     }

//     fn get_sof_zman_kidush_levana_between_moldos(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//         self.invoke_date("getSofZmanKidushLevanaBetweenMoldos")
//     }

//     fn get_sof_zman_kidush_levana_15_days(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//         self.invoke_date("getSofZmanKidushLevana15Days")
//     }

//     fn get_tekufas_tishrei_elapsed_days(&self) -> i64 {
//         self.invoke_i64("getTekufasTishreiElapsedDays")
//     }

//     fn is_vesein_tal_umatar_start_date(&self) -> bool {
//         self.invoke_bool("isVeseinTalUmatarStartDate")
//     }

//     fn is_vesein_tal_umatar_starting_tonight(&self) -> bool {
//         self.invoke_bool("isVeseinTalUmatarStartingTonight")
//     }

//     fn is_vesein_tal_umatar_recited(&self) -> bool {
//         self.invoke_bool("isVeseinTalUmatarRecited")
//     }

//     fn is_vesein_beracha_recited(&self) -> bool {
//         self.invoke_bool("isVeseinBerachaRecited")
//     }

//     fn is_mashiv_haruach_start_date(&self) -> bool {
//         self.invoke_bool("isMashivHaruachStartDate")
//     }

//     fn is_mashiv_haruach_end_date(&self) -> bool {
//         self.invoke_bool("isMashivHaruachEndDate")
//     }

//     fn is_mashiv_haruach_recited(&self) -> Option<bool> {
//         Some(self.invoke_bool("isMashivHaruachRecited"))
//     }

//     fn is_morid_hatal_recited(&self) -> Option<bool> {
//         Some(self.invoke_bool("isMoridHatalRecited"))
//     }
// }

// pub struct JavaTefilaRules<'a> {
//     pub jvm: &'a Jvm,
//     pub instance: Instance,
// }
// fn bool_to_invocation_arg(bool: bool) -> InvocationArg {
//     InvocationArg::try_from(bool).unwrap().into_primitive().unwrap()
// }
// impl<'a> JavaTefilaRules<'a> {
//     #[allow(clippy::too_many_arguments)]
//     pub fn new(
//         jvm: &'a Jvm,
//         tachanun_recited_end_of_tishrei: bool,
//         tachanun_recited_week_after_shavuos: bool,
//         tachanun_recited_13_sivan_out_of_israel: bool,
//         tachanun_recited_pesach_sheni: bool,
//         tachanun_recited_15_iyar_out_of_israel: bool,
//         tachanun_recited_mincha_erev_lag_baomer: bool,
//         tachanun_recited_shivas_yemei_hamiluim: bool,
//         tachanun_recited_week_of_hod: bool,
//         tachanun_recited_week_of_purim: bool,
//         tachanun_recited_fridays: bool,
//         tachanun_recited_sundays: bool,
//         tachanun_recited_mincha_all_year: bool,
//         mizmor_lesoda_recited_erev_yom_kippur_and_pesach: bool,
//     ) -> Self {
//         let instance = jvm
//             .create_instance(
//                 "com.kosherjava.zmanim.hebrewcalendar.TefilaRules",
//                 InvocationArg::empty(),
//             )
//             .unwrap();
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedWeekOfPurim",
//             &[bool_to_invocation_arg(tachanun_recited_week_of_purim)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedWeekOfHod",
//             &[bool_to_invocation_arg(tachanun_recited_week_of_hod)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedEndOfTishrei",
//             &[bool_to_invocation_arg(tachanun_recited_end_of_tishrei)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedWeekAfterShavuos",
//             &[bool_to_invocation_arg(tachanun_recited_week_after_shavuos)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecited13SivanOutOfIsrael",
//             &[bool_to_invocation_arg(tachanun_recited_13_sivan_out_of_israel)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedPesachSheni",
//             &[bool_to_invocation_arg(tachanun_recited_pesach_sheni)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecited15IyarOutOfIsrael",
//             &[bool_to_invocation_arg(tachanun_recited_15_iyar_out_of_israel)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedMinchaErevLagBaomer",
//             &[bool_to_invocation_arg(tachanun_recited_mincha_erev_lag_baomer)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedShivasYemeiHamiluim",
//             &[bool_to_invocation_arg(tachanun_recited_shivas_yemei_hamiluim)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedFridays",
//             &[bool_to_invocation_arg(tachanun_recited_fridays)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedSundays",
//             &[bool_to_invocation_arg(tachanun_recited_sundays)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setTachanunRecitedMinchaAllYear",
//             &[bool_to_invocation_arg(tachanun_recited_mincha_all_year)],
//         );
//         let _ = jvm.invoke(
//             &instance,
//             "setMizmorLesodaRecitedErevYomKippurAndPesach",
//             &[bool_to_invocation_arg(mizmor_lesoda_recited_erev_yom_kippur_and_pesach)],
//         );
//         Self { jvm, instance }
//     }
// }
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
