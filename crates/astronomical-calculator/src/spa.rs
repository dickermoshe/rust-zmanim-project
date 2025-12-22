use crate::tables::*;
use crate::types::*;
use core::ops::Rem;

#[allow(unused_imports)] // will be unused on std targets
use core_maths::*;

pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const M_PI: f64 = 3.14159265358979323846f64;
pub const JD0: f64 = 2451545.0f64;
pub const ETJD0: i64 = 946728000 as i64;
pub const SUN_RADIUS: f64 = 4.6542695162932789e-03f64;
pub const EARTH_R: f64 = 6378136.6f64;
pub const ABSOLUTEZERO: f64 = -273.15f64;
pub const AP0: f64 = 1010.0f64;
pub const AT0: f64 = 10.0f64;
pub const NDT: i64 = 1245 as i64;
pub const __INT_MAX__: i64 = 2147483647 as i64;
pub const INT_MAX: i64 = __INT_MAX__;
pub const INT_MIN: i64 = -__INT_MAX__ - 1 as i64;
fn acos(x: f64) -> f64 {
    x.acos()
}
fn asin(x: f64) -> f64 {
    x.asin()
}
fn atan(x: f64) -> f64 {
    x.atan()
}
fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}
fn cos(x: f64) -> f64 {
    x.cos()
}
fn sin(x: f64) -> f64 {
    x.sin()
}
fn tan(x: f64) -> f64 {
    x.tan()
}
fn fabs(x: f64) -> f64 {
    x.abs()
}
fn fmod(x: f64, y: f64) -> f64 {
    x.rem(y)
}
fn round(x: f64) -> f64 {
    x.round()
}
fn trunc(x: f64) -> f64 {
    x.trunc()
}
pub fn get_delta_t(ut: &tm) -> f64 {
    let mut imin: i64 = 0;
    let mut imax: i64 = NDT - 1;
    let dyear =
        ut.tm_year as f64 + 1900.0f64 + (ut.tm_mon as f64 + 1.0f64) / 12f64 + (ut.tm_mday as f64 - 1.0f64) / 365.0f64;
    if freespa_delta_t_table[0 as usize] > dyear {
        return 32.0f64 * ((dyear - 1820.0f64) / 100f64) * ((dyear - 1820.0f64) / 100f64) - 20f64;
    }
    if freespa_delta_t_table[(2 * imax) as usize] < dyear {
        return 32.0f64 * ((dyear - 1820.0f64) / 100f64) * ((dyear - 1820.0f64) / 100f64) - 20f64;
    }
    while imax - imin > 1 {
        let i = (imin + imax) / 2;
        if freespa_delta_t_table[(2 * i) as usize] > dyear {
            imax = i;
        } else if freespa_delta_t_table[(2 * i) as usize] < dyear {
            imin = i;
        } else {
            return freespa_delta_t_table[(2 * i + 1) as usize];
        }
    }
    return freespa_delta_t_table[(2 * imin + 1) as usize]
        + (dyear - freespa_delta_t_table[(2 * imin) as usize])
            * (freespa_delta_t_table[(2 * imax + 1) as usize] - freespa_delta_t_table[(2 * imin + 1) as usize])
            / (freespa_delta_t_table[(2 * imax) as usize] - freespa_delta_t_table[(2 * imin) as usize]);
}
pub fn MakeJulianDay(ut: &tm, delta_t: Option<f64>, delta_ut1: f64) -> JulianDay {
    let E = 0;
    let day = ut.tm_mday as f64
        + (ut.tm_hour as f64 + (ut.tm_min as f64 + (ut.tm_sec as f64 + delta_ut1) / 60.0f64) / 60.0f64) / 24f64;
    let mut month = ut.tm_mon + 1 as i64;
    let mut year = ut.tm_year + 1900 as i64;
    if month < 3 as i64 {
        month += 12 as i64;
        year -= 1;
    }
    let mut JD =
        trunc(365.25f64 * (year as f64 + 4716.0f64)) + trunc(30.6001f64 * (month as f64 + 1.0f64)) + day - 1524.5f64;
    if JD > 2299160.0f64 {
        let a = trunc(year as f64 / 100.0f64);
        JD += 2f64 - a + trunc(a / 4f64);
    }
    let dt = delta_t.unwrap_or(get_delta_t(ut));

    let JDE = JD + dt / 86400.0f64;
    let JC = (JD - JD0) / 36525.0f64;
    let JCE = (JDE - JD0) / 36525.0f64;
    let JME = JCE / 10.0f64;
    return JulianDate {
        JD,
        JDE,
        JC,
        JCE,
        JME,
        E,
    };
}

pub fn f64_to_i64(v: f64) -> Option<i64> {
    // Reject NaN and infinity first.
    if !v.is_finite() {
        return None;
    }

    // Exact bounds check using the real limits of i64.
    const MIN: f64 = i64::MIN as f64;
    const MAX: f64 = i64::MAX as f64;

    if v >= MIN && v <= MAX {
        // Safe cast: the value is in range; truncation is toward zero.
        Some(v as i64)
    } else {
        None
    }
}

pub fn JDgmtime(julian_day: f64, ut: &mut tm) -> Option<()> {
    let Z = trunc(julian_day + 0.5f64);
    let F = julian_day - Z;
    let A: f64;
    if Z < 2299161f64 {
        A = Z;
    } else {
        let B = trunc((Z - 1867216.25f64) / 36524.25f64);
        A = Z + 1f64 + B - trunc(B / 4.0f64);
    }
    let C = A + 1524f64;
    let D = trunc((C - 122.1f64) / 365.25f64);
    let G = trunc(365.25f64 * D);
    let I = trunc((C - G) / 30.6001f64);
    let mut d = C - G - trunc(30.6001f64 * I) + F - 0.5f64;
    ut.tm_mday = trunc(d) as i64 + 1 as i64;
    if I < 14f64 {
        ut.tm_mon = (I - 2f64) as i64;
    } else {
        ut.tm_mon = (I - 14f64) as i64;
    }

    if ut.tm_mon > 1 as i64 {
        ut.tm_year = f64_to_i64(D - 4716f64 - 1900f64)?;
    } else {
        ut.tm_year = f64_to_i64(D - 4715f64 - 1900f64)?;
    }
    d -= trunc(d);
    d *= 86400f64;
    d = round(d);
    ut.tm_sec = d as i64 % 60;
    d -= ut.tm_sec as f64;
    d /= 60f64;
    ut.tm_min = d as i64 % 60;
    d -= ut.tm_min as f64;
    d /= 60f64;
    ut.tm_hour = d as i64 % 60;
    d -= ut.tm_hour as f64;
    Some(())
}
pub fn gmjtime_r(t: i64, ut: &mut tm) -> Option<()> {
    JDgmtime((t - ETJD0 as i64) as f64 / 86400.0f64 + JD0, ut)
}

pub fn mkgmjtime(ut: &mut tm) -> i64 {
    let mut J: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    J = MakeJulianDay(ut, None, 0f64);
    return round((J.JD - JD0) * 86400f64) as i64 + ETJD0 as i64;
}

pub fn JDmkgmjtime(J: JulianDay) -> i64 {
    round((J.JD - JD0) * 86400f64) as i64 + ETJD0 as i64
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolarPositionError {
    #[error("Time conversion error")]
    TimeConversionError,
}

pub fn MakeJulianDayEpoch(t: i64, delta_t: f64, delta_ut1: f64) -> Result<JulianDay, SolarPositionError> {
    let mut ut: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };

    let mut JD: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    if gmjtime_r(t, &mut ut).is_none() {
        return Err(SolarPositionError::TimeConversionError);
    } else {
        JD = MakeJulianDay(&mut ut, Some(delta_t), delta_ut1);
    }
    Ok(JD)
}
pub fn SummPTerms(p: &[p_term], JD: JulianDay) -> f64 {
    let mut s: f64 = 0f64;
    for p in p.iter() {
        s += p.A * cos(p.P + p.W * JD.JME);
    }
    s
}
pub fn Heliocentric_lon(JD: JulianDay) -> f64 {
    let mut lon: f64 = 0.0;
    let mut pp: f64 = 0.;
    lon = SummPTerms(&EarthLon0, JD);
    pp = JD.JME;
    lon += SummPTerms(&EarthLon1, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&EarthLon2, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&EarthLon3, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&EarthLon4, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&EarthLon5, JD) * pp;
    lon /= 1.0e8f64;
    return lon;
}

pub fn Heliocentric_lat(JD: JulianDay) -> f64 {
    let mut lat: f64 = 0.;
    lat = SummPTerms(&EarthLat0, JD);
    lat += SummPTerms(&EarthLat1, JD) * JD.JME;
    lat /= 1.0e8f64;
    return lat;
}

pub fn Heliocentric_rad(JD: JulianDay) -> f64 {
    let mut rad: f64 = 0.;
    let mut pp: f64 = 0.;
    rad = SummPTerms(&EarthRad0, JD);
    pp = JD.JME;
    rad += SummPTerms(&EarthRad1, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&EarthRad2, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&EarthRad3, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&EarthRad4, JD) * pp;
    rad /= 1.0e8f64;
    return rad;
}

pub fn Geocentric_pos(JD: JulianDay) -> GeoCentricSolPos {
    let mut P: GeoCentricSolPos = GeoCentricSolPos {
        lat: 0.,
        lon: 0.,
        rad: 0.,
    };
    P.lat = fmod(-Heliocentric_lat(JD), 2f64 * M_PI);
    P.lon = fmod(Heliocentric_lon(JD) + M_PI, 2f64 * M_PI);
    if P.lon < 0f64 {
        P.lon += 2f64 * M_PI;
    }
    P.rad = Heliocentric_rad(JD);
    return P;
}
