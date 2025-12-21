#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
use crate::tables::*;
use crate::types::*;
use core::ops::Rem;
#[allow(unused_imports)] // will be unused on std targets
use core_maths::*;

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
pub const _FREESPA_DEU_OOR: i64 = 0x1 as i64;
pub const _FREESPA_LON_OOR: i64 = 0x2 as i64;
pub const _FREESPA_LAT_OOR: i64 = 0x4 as i64;
pub const _FREESPA_ELE_OOR: i64 = 0x8 as i64;
pub const _FREESPA_PRE_OOR: i64 = 0x10 as i64;
pub const _FREESPA_TEM_OOR: i64 = 0x20 as i64;
pub const _FREESPA_DIP_OOR: i64 = 0x40 as i64;
pub const _FREESPA_GMTIMEF: i64 = 0x80 as i64;
pub const _FREESPA_SUNRISE: i64 = 0x1 as i64;
pub const _FREESPA_SUNSET: i64 = 0x2 as i64;
pub const _FREESPA_CVDAWN: i64 = 0x4 as i64;
pub const _FREESPA_CVDUSK: i64 = 0x8 as i64;
pub const _FREESPA_NADAWN: i64 = 0x10 as i64;
pub const _FREESPA_NADUSK: i64 = 0x20 as i64;
pub const _FREESPA_ASDAWN: i64 = 0x40 as i64;
pub const _FREESPA_ASDUSK: i64 = 0x80 as i64;
pub const _FREESPA_EV_ERR: i64 = 20 as i64;
pub const _FREESPA_EV_NA: i64 = 10 as i64;
pub const _FREESPA_EV_OK: i64 = 0 as i64;
pub const NDT: i64 = 1245 as i64;

pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const M_PI: f64 = 3.14159265358979323846f64;
pub const JD0: f64 = 2451545.0f64;
pub const ETJD0: i64 = 946728000 as i64;
pub const SUN_RADIUS: f64 = 4.6542695162932789e-03f64;
pub const EARTH_R: f64 = 6378136.6f64;
pub const ABSOLUTEZERO: f64 = -273.15f64;
pub const AP0: f64 = 1010.0f64;
pub const AT0: f64 = 10.0f64;
#[no_mangle]
pub unsafe extern "C" fn get_delta_t(mut ut: *mut tm) -> f64 {
    let mut dyear: f64 = 0.;
    let mut imin: i64 = 0 as i64;
    let mut imax: i64 = NDT - 1 as i64;
    let mut i: i64 = 0;
    dyear = (*ut).tm_year as f64
        + 1900.0f64
        + ((*ut).tm_mon as f64 + 1.0f64) / 12f64
        + ((*ut).tm_mday as f64 - 1.0f64) / 365.0f64;
    if freespa_delta_t_table[0 as i64 as usize] > dyear {
        return 32.0f64 * ((dyear - 1820.0f64) / 100f64) * ((dyear - 1820.0f64) / 100f64) - 20f64;
    }
    if freespa_delta_t_table[(2 as i64 * imax) as usize] < dyear {
        return 32.0f64 * ((dyear - 1820.0f64) / 100f64) * ((dyear - 1820.0f64) / 100f64) - 20f64;
    }
    while imax - imin > 1 as i64 {
        i = (imin + imax) / 2 as i64;
        if freespa_delta_t_table[(2 as i64 * i) as usize] > dyear {
            imax = i;
        } else if freespa_delta_t_table[(2 as i64 * i) as usize] < dyear {
            imin = i;
        } else {
            return freespa_delta_t_table[(2 as i64 * i + 1 as i64) as usize];
        }
    }
    return freespa_delta_t_table[(2 as i64 * imin + 1 as i64) as usize]
        + (dyear - freespa_delta_t_table[(2 as i64 * imin) as usize])
            * (freespa_delta_t_table[(2 as i64 * imax + 1 as i64) as usize]
                - freespa_delta_t_table[(2 as i64 * imin + 1 as i64) as usize])
            / (freespa_delta_t_table[(2 as i64 * imax) as usize] - freespa_delta_t_table[(2 as i64 * imin) as usize]);
}
#[no_mangle]
pub unsafe extern "C" fn MakeJulianDay(mut ut: *mut tm, mut delta_t: *mut f64, mut delta_ut1: f64) -> JulianDay {
    let mut month: i64 = 0;
    let mut year: i64 = 0;
    let mut day: f64 = 0.;
    let mut a: f64 = 0.;
    let mut dt: f64 = 0.;
    let mut JD: JulianDay = JulianDate {
        JD: 0f64,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    JD.E = 0 as i64;
    day = (*ut).tm_mday as f64
        + ((*ut).tm_hour as f64 + ((*ut).tm_min as f64 + ((*ut).tm_sec as f64 + delta_ut1) / 60.0f64) / 60.0f64)
            / 24f64;
    month = (*ut).tm_mon + 1 as i64;
    year = (*ut).tm_year + 1900 as i64;
    if month < 3 as i64 {
        month += 12 as i64;
        year -= 1;
    }
    JD.JD =
        trunc(365.25f64 * (year as f64 + 4716.0f64)) + trunc(30.6001f64 * (month as f64 + 1.0f64)) + day - 1524.5f64;
    if JD.JD > 2299160.0f64 {
        a = trunc(year as f64 / 100.0f64);
        JD.JD += 2f64 - a + trunc(a / 4f64);
    }
    if !delta_t.is_null() {
        dt = *delta_t;
    } else {
        dt = get_delta_t(ut);
    }
    JD.JDE = JD.JD + dt / 86400.0f64;
    JD.JC = (JD.JD - JD0) / 36525.0f64;
    JD.JCE = (JD.JDE - JD0) / 36525.0f64;
    JD.JME = JD.JCE / 10.0f64;
    return JD;
}
#[no_mangle]
pub unsafe extern "C" fn SetIntLimits(mut v: f64, mut t: *mut i64) -> i64 {
    if v > INT_MIN as f64 && v < INT_MAX as f64 {
        *t = v as i64;
        return 0 as i64;
    }
    *t = 0 as i64;
    return 1 as i64;
}
#[no_mangle]
pub unsafe extern "C" fn JDgmtime(mut JD: JulianDay, mut ut: *mut tm) -> *mut tm {
    let mut A: f64 = 0.;
    let mut B: f64 = 0.;
    let mut C: f64 = 0.;
    let mut D: f64 = 0.;
    let mut F: f64 = 0.;
    let mut G: f64 = 0.;
    let mut I: f64 = 0.;
    let mut Z: f64 = 0.;
    let mut d: f64 = 0.;
    Z = trunc(JD.JD + 0.5f64);
    F = JD.JD - Z;
    if Z < 2299161f64 {
        A = Z;
    } else {
        B = trunc((Z - 1867216.25f64) / 36524.25f64);
        A = Z + 1f64 + B - trunc(B / 4.0f64);
    }
    C = A + 1524f64;
    D = trunc((C - 122.1f64) / 365.25f64);
    G = trunc(365.25f64 * D);
    I = trunc((C - G) / 30.6001f64);
    d = C - G - trunc(30.6001f64 * I) + F - 0.5f64;
    (*ut).tm_mday = trunc(d) as i64 + 1 as i64;
    if I < 14f64 {
        (*ut).tm_mon = (I - 2f64) as i64;
    } else {
        (*ut).tm_mon = (I - 14f64) as i64;
    }
    if (*ut).tm_mon > 1 as i64 {
        if SetIntLimits(D - 4716f64 - 1900f64, &raw mut (*ut).tm_year) != 0 {
            return ::core::ptr::null_mut::<tm>();
        }
    } else if SetIntLimits(D - 4715f64 - 1900f64, &raw mut (*ut).tm_year) != 0 {
        return ::core::ptr::null_mut::<tm>();
    }
    d -= trunc(d);
    d *= 86400f64;
    d = round(d);
    (*ut).tm_sec = d as i64 % 60 as i64;
    d -= (*ut).tm_sec as f64;
    d /= 60f64;
    (*ut).tm_min = d as i64 % 60 as i64;
    d -= (*ut).tm_min as f64;
    d /= 60f64;
    (*ut).tm_hour = d as i64 % 60 as i64;
    d -= (*ut).tm_hour as f64;
    return ut;
}
#[no_mangle]
pub unsafe extern "C" fn gmjtime_r(mut t: *mut i64, mut ut: *mut tm) -> *mut tm {
    let mut J: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    J.JD = (*t - ETJD0 as i64) as f64 / 86400.0f64 + JD0;
    return JDgmtime(J, ut);
}
#[no_mangle]
pub unsafe extern "C" fn gmjtime(mut t: *mut i64) -> *mut tm {
    static mut _tmbuf: tm = tm {
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
    return gmjtime_r(t, &raw mut _tmbuf);
}
#[no_mangle]
pub unsafe extern "C" fn mkgmjtime(mut ut: *mut tm) -> i64 {
    let mut J: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    J = MakeJulianDay(ut, ::core::ptr::null_mut::<f64>(), 0f64);
    return round((J.JD - JD0) * 86400f64) as i64 + ETJD0 as i64;
}
#[no_mangle]
pub unsafe extern "C" fn JDmkgmjtime(mut J: JulianDay) -> i64 {
    return round((J.JD - JD0) * 86400f64) as i64 + ETJD0 as i64;
}
#[no_mangle]
pub unsafe extern "C" fn MakeJulianDayEpoch(mut t: i64, mut delta_t: *mut f64, mut delta_ut1: f64) -> JulianDay {
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
    let mut p: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut JD: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    p = gmjtime_r(&raw mut t, &raw mut ut);
    if p.is_null() {
        JD.JD = 0.0f64;
        JD.JDE = 0.0f64;
        JD.JC = 0.0f64;
        JD.JCE = 0.0f64;
        JD.JME = 0.0f64;
        JD.E = _FREESPA_GMTIMEF;
    } else {
        JD = MakeJulianDay(&raw mut ut, delta_t, delta_ut1);
    }
    return JD;
}
#[no_mangle]
pub unsafe extern "C" fn SummPTerms(mut p: *const p_term, mut N: i64, mut JD: JulianDay) -> f64 {
    let mut i: i64 = 0 as i64;
    let mut s: f64 = 0f64;
    i = 0 as i64;
    while i < N {
        s += (*p.offset(i as isize)).A * cos((*p.offset(i as isize)).P + (*p.offset(i as isize)).W * JD.JME);
        i += 1;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn Heliocentric_lon(mut JD: JulianDay) -> f64 {
    let mut lon: f64 = 0.;
    let mut pp: f64 = 0.;
    lon = SummPTerms(&raw const EarthLon0 as *const p_term, N_LON0, JD);
    pp = JD.JME;
    lon += SummPTerms(&raw const EarthLon1 as *const p_term, N_LON1, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&raw const EarthLon2 as *const p_term, N_LON2, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&raw const EarthLon3 as *const p_term, N_LON3, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&raw const EarthLon4 as *const p_term, N_LON4, JD) * pp;
    pp *= JD.JME;
    lon += SummPTerms(&raw const EarthLon5 as *const p_term, N_LON5, JD) * pp;
    lon /= 1.0e8f64;
    return lon;
}
#[no_mangle]
pub unsafe extern "C" fn Heliocentric_lat(mut JD: JulianDay) -> f64 {
    let mut lat: f64 = 0.;
    lat = SummPTerms(&raw const EarthLat0 as *const p_term, N_LAT0, JD);
    lat += SummPTerms(&raw const EarthLat1 as *const p_term, N_LAT1, JD) * JD.JME;
    lat /= 1.0e8f64;
    return lat;
}
#[no_mangle]
pub unsafe extern "C" fn Heliocentric_rad(mut JD: JulianDay) -> f64 {
    let mut rad: f64 = 0.;
    let mut pp: f64 = 0.;
    rad = SummPTerms(&raw const EarthRad0 as *const p_term, N_RAD0, JD);
    pp = JD.JME;
    rad += SummPTerms(&raw const EarthRad1 as *const p_term, N_RAD1, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&raw const EarthRad2 as *const p_term, N_RAD2, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&raw const EarthRad3 as *const p_term, N_RAD3, JD) * pp;
    pp *= JD.JME;
    rad += SummPTerms(&raw const EarthRad4 as *const p_term, N_RAD4, JD) * pp;
    rad /= 1.0e8f64;
    return rad;
}
#[no_mangle]
pub unsafe extern "C" fn Geocentric_pos(mut JD: JulianDay) -> GeoCentricSolPos {
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
#[inline]
unsafe extern "C" fn poly(mut a: *const f64, mut N: i64, mut x: f64) -> f64 {
    let mut i: i64 = 0;
    let mut r: f64 = 0.;
    r = *a.offset(0 as i64 as isize);
    i = 1 as i64;
    while i < N {
        r = *a.offset(i as isize) + x * r;
        i += 1;
    }
    return r;
}
#[no_mangle]
pub static mut MEAN_ELONGATION_MOON_SUN: [f64; 4] = [
    M_PI * (1.0f64 / 189474.0f64) / 180.0f64,
    M_PI * -1.9142e-03f64 / 180.0f64,
    M_PI * 445267.11148f64 / 180.0f64,
    M_PI * 297.85036f64 / 180.0f64,
];
#[no_mangle]
pub static mut MEAN_ANOMALY_SUN: [f64; 4] = [
    M_PI * (-1.0f64 / 300000.0f64) / 180.0f64,
    M_PI * -0.0001603f64 / 180.0f64,
    M_PI * 35999.05034f64 / 180.0f64,
    M_PI * 357.52772f64 / 180.0f64,
];
#[no_mangle]
pub static mut MEAN_ANOMALY_MOON: [f64; 4] = [
    M_PI * (1.0f64 / 56250.0f64) / 180.0f64,
    M_PI * 0.0086972f64 / 180.0f64,
    M_PI * 477198.867398f64 / 180.0f64,
    M_PI * 134.96298f64 / 180.0f64,
];
#[no_mangle]
pub static mut ARG_LAT_MOON: [f64; 4] = [
    M_PI * (1.0f64 / 327270.0f64) / 180.0f64,
    M_PI * -0.0036825f64 / 180.0f64,
    M_PI * 483202.017538f64 / 180.0f64,
    M_PI * 93.27191f64 / 180.0f64,
];
#[no_mangle]
pub static mut ASC_LON_MOON: [f64; 4] = [
    M_PI * (1.0f64 / 450000.0f64) / 180.0f64,
    M_PI * 0.0020708f64 / 180.0f64,
    M_PI * -1934.136261f64 / 180.0f64,
    M_PI * 125.04452f64 / 180.0f64,
];
#[no_mangle]
pub unsafe extern "C" fn Nutation_lon_obliquity(mut JD: JulianDay, mut del_psi: *mut f64, mut del_eps: *mut f64) {
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    let mut sum: f64 = 0.;
    let mut sum_psi: f64 = 0f64;
    let mut sum_eps: f64 = 0f64;
    let mut x: [f64; 5] = [0.; 5];
    x[0 as i64 as usize] = poly(&raw const MEAN_ELONGATION_MOON_SUN as *const f64, 4 as i64, JD.JCE);
    x[1 as i64 as usize] = poly(&raw const MEAN_ANOMALY_SUN as *const f64, 4 as i64, JD.JCE);
    x[2 as i64 as usize] = poly(&raw const MEAN_ANOMALY_MOON as *const f64, 4 as i64, JD.JCE);
    x[3 as i64 as usize] = poly(&raw const ARG_LAT_MOON as *const f64, 4 as i64, JD.JCE);
    x[4 as i64 as usize] = poly(&raw const ASC_LON_MOON as *const f64, 4 as i64, JD.JCE);
    i = 0 as i64;
    while i < NY {
        sum = 0f64;
        j = 0 as i64;
        while j < 5 as i64 {
            sum += x[j as usize] * Y_Terms[i as usize][j as usize] as f64;
            j += 1;
        }
        sum_psi +=
            (PE_Terms[i as usize][0 as i64 as usize] + JD.JCE * PE_Terms[i as usize][1 as i64 as usize]) * sin(sum);
        sum_eps +=
            (PE_Terms[i as usize][2 as i64 as usize] + JD.JCE * PE_Terms[i as usize][3 as i64 as usize]) * cos(sum);
        i += 1;
    }
    *del_psi = sum_psi * (M_PI * (1f64 / 36000000.0f64) / 180.0f64);
    *del_eps = sum_eps * (M_PI * (1f64 / 36000000.0f64) / 180.0f64);
}
#[no_mangle]
pub static mut ECLIPTIC_MEAN_OBLIQUITY: [f64; 11] = [
    2.45f64,
    5.79f64,
    27.87f64,
    7.12f64,
    -39.05f64,
    -249.67f64,
    -51.38f64,
    1999.25f64,
    -1.55f64,
    -4680.93f64,
    84381.448f64,
];
#[no_mangle]
pub static mut GSTA: [f64; 4] = [
    M_PI * (-(1 as i64) as f64 / 38710000.0f64) / 180.0f64,
    M_PI * 0.000387933f64 / 180.0f64,
    0.0f64,
    M_PI * 280.46061837f64 / 180.0f64,
];
#[no_mangle]
pub unsafe extern "C" fn solpos(
    mut lon: f64,
    mut lat: f64,
    mut e: f64,
    mut JD: JulianDay,
    mut GP: GeoCentricSolPos,
) -> sol_pos {
    let mut dtau: f64 = 0.;
    let mut v: f64 = 0.;
    let mut H: f64 = 0.;
    let mut u: f64 = 0.;
    let mut x: f64 = 0.;
    let mut y: f64 = 0.;
    let mut lambda: f64 = 0.;
    let mut alpha: f64 = 0.;
    let mut delta: f64 = 0.;
    let mut xi: f64 = 0.;
    let mut delta_prime: f64 = 0.;
    let mut H_prime: f64 = 0.;
    let mut dpsi: f64 = 0.;
    let mut deps: f64 = 0.;
    let mut eps: f64 = 0.;
    let mut dalpha: f64 = 0.;
    let mut h: f64 = 0.;
    let mut P: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    dtau = M_PI * (-20.4898f64 / 3600.0f64) / 180.0f64 / GP.rad;
    Nutation_lon_obliquity(JD, &raw mut dpsi, &raw mut deps);
    eps = deps
        + poly(
            &raw const ECLIPTIC_MEAN_OBLIQUITY as *const f64,
            11 as i64,
            JD.JME / 10f64,
        ) * (M_PI * (1f64 / 3600.0f64) / 180.0f64);
    lambda = GP.lon + dpsi + dtau;
    v = poly(&raw const GSTA as *const f64, 4 as i64, JD.JC) + M_PI * 360.98564736629f64 / 180.0f64 * (JD.JD - JD0);
    v += dpsi * cos(eps);
    alpha = atan2(sin(lambda) * cos(eps) - tan(GP.lat) * sin(eps), cos(lambda));
    if alpha < 0f64 {
        alpha += 2f64 * M_PI;
    }
    delta = asin(sin(GP.lat) * cos(eps) + cos(GP.lat) * sin(eps) * sin(lambda));
    H = v + lon - alpha;
    xi = M_PI * (8.794f64 / 3600.0f64) / 180.0f64 / GP.rad;
    u = atan(0.99664719f64 * tan(lat));
    x = cos(u) + e * cos(lat) / EARTH_R;
    y = 0.99664719f64 * sin(u) + e * sin(lat) / EARTH_R;
    dalpha = atan2(-x * sin(xi) * sin(H), cos(delta) - x * sin(xi) * cos(H));
    delta_prime = atan2(
        (sin(delta) - y * sin(xi)) * cos(dalpha),
        cos(delta) - x * sin(xi) * cos(H),
    );
    H_prime = H - dalpha;
    h = asin(sin(lat) * sin(delta_prime) + cos(lat) * cos(delta_prime) * cos(H_prime));
    P.z = M_PI / 2f64 - h;
    P.a = fmod(
        M_PI + atan2(sin(H_prime), cos(H_prime) * sin(lat) - tan(delta_prime) * cos(lat)),
        2f64 * M_PI,
    );
    P.z = fmod(P.z, 2f64 * M_PI);
    if P.z < 0f64 {
        P.z = -P.z;
        P.a = fmod(P.a + M_PI, 2f64 * M_PI);
    }
    if P.z > M_PI {
        P.z = 2f64 * M_PI - P.z;
        P.a = fmod(P.a + 2f64 * M_PI, 2f64 * M_PI);
    }
    return P;
}
#[no_mangle]
pub static mut SMLON: [f64; 6] = [
    M_PI * (-(1 as i64) as f64 / 2000000.0f64) / 180.0f64,
    M_PI * (-(1 as i64) as f64 / 15300.0f64) / 180.0f64,
    M_PI * (1f64 / 49931.0f64) / 180.0f64,
    M_PI * 0.03032028f64 / 180.0f64,
    M_PI * 360007.6982779f64 / 180.0f64,
    M_PI * 280.4664567f64 / 180.0f64,
];
#[no_mangle]
pub unsafe extern "C" fn EoT(mut lat: f64, mut JD: JulianDay, mut GP: GeoCentricSolPos) -> f64 {
    let mut M: f64 = 0.;
    let mut E: f64 = 0.;
    let mut dtau: f64 = 0.;
    let mut lambda: f64 = 0.;
    let mut alpha: f64 = 0.;
    let mut eps: f64 = 0.;
    let mut deps: f64 = 0.;
    let mut dpsi: f64 = 0.;
    dtau = M_PI * (-20.4898f64 / 3600.0f64) / 180.0f64 / GP.rad;
    Nutation_lon_obliquity(JD, &raw mut dpsi, &raw mut deps);
    eps = deps
        + poly(
            &raw const ECLIPTIC_MEAN_OBLIQUITY as *const f64,
            11 as i64,
            JD.JME / 10f64,
        ) * (M_PI * (1f64 / 3600.0f64) / 180.0f64);
    lambda = GP.lon + dpsi + dtau;
    alpha = atan2(sin(lambda) * cos(eps) - tan(GP.lat) * sin(eps), cos(lambda));
    M = poly(&raw const SMLON as *const f64, 6 as i64, JD.JME);
    E = fmod(
        M - M_PI * 0.0057183f64 / 180.0f64 - alpha + dpsi * cos(eps),
        2f64 * M_PI,
    );
    if E > M_PI * 5f64 / 180.0f64 {
        E -= 2f64 * M_PI;
    }
    if E < -(M_PI * 5f64 / 180.0f64) {
        E += 2f64 * M_PI;
    }
    return E;
}
#[no_mangle]
pub unsafe extern "C" fn InputCheck(
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
    mut e: f64,
    mut p: f64,
    mut T: f64,
) -> i64 {
    let mut E: i64 = 0 as i64;
    if delta_ut1 < -(1 as i64) as f64 || delta_ut1 > 1f64 {
        E |= _FREESPA_DEU_OOR;
    }
    if lon < -M_PI || lon > M_PI {
        E |= _FREESPA_LON_OOR;
    }
    if lat < -M_PI / 2f64 || lat > M_PI / 2f64 {
        E |= _FREESPA_LAT_OOR;
    }
    if e < -EARTH_R {
        E |= _FREESPA_ELE_OOR;
    }
    if p < 0f64 || p > 5000f64 {
        E |= _FREESPA_PRE_OOR;
    }
    if T < ABSOLUTEZERO {
        E |= _FREESPA_TEM_OOR;
    }
    return E;
}
#[no_mangle]
pub unsafe extern "C" fn SPA(
    mut ut: *mut tm,
    mut delta_t: *mut f64,
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
    mut e: f64,
) -> sol_pos {
    let mut D: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    let mut G: GeoCentricSolPos = GeoCentricSolPos {
        lat: 0.,
        lon: 0.,
        rad: 0.,
    };
    let mut P: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    P.E = InputCheck(delta_ut1, lon, lat, e, 1010f64, 10f64);
    if P.E == 0 {
        D = MakeJulianDay(ut, delta_t, delta_ut1);
        if D.E != 0 {
            P.E |= D.E;
            return P;
        }
        G = Geocentric_pos(D);
        P = solpos(lon, lat, e, D, G);
    }
    return P;
}
#[inline]
unsafe extern "C" fn Refr(mut coeff: *const f64, mut p: f64, mut T: f64, mut h: f64) -> f64 {
    return p / AP0 * ((AT0 - ABSOLUTEZERO) / (T - ABSOLUTEZERO)) * *coeff.offset(0 as i64 as isize)
        / tan(h + *coeff.offset(1 as i64 as isize) / (h + *coeff.offset(2 as i64 as isize)));
}
#[inline]
unsafe extern "C" fn Bennet(mut p: f64, mut T: f64, mut h: f64) -> f64 {
    let BENNET: [f64; 3] = [
        2.9088820866572158e-04f64,
        2.2267533386408395e-03f64,
        7.6794487087750510e-02f64,
    ];
    return Refr(&raw const BENNET as *const f64, p, T, h);
}
#[inline]
unsafe extern "C" fn iBennet(mut p: f64, mut T: f64, mut h: f64) -> f64 {
    let IBENNET: [f64; 3] = [
        2.9670597283903603e-04f64,
        3.1375594238030984e-03f64,
        8.9186324776910242e-02f64,
    ];
    return Refr(&raw const IBENNET as *const f64, p, T, h);
}
#[inline]
unsafe extern "C" fn BennetNA(mut p: f64, mut T: f64, mut h: f64) -> f64 {
    let BENNET: [f64; 3] = [
        2.9088820866572158e-04f64,
        2.2297995128387070e-03f64,
        7.5398223686155036e-02f64,
    ];
    return Refr(&raw const BENNET as *const f64, p, T, h);
}
#[inline]
unsafe extern "C" fn iBennetNA(mut p: f64, mut T: f64, mut h: f64) -> f64 {
    let IBENNET: [f64; 3] = [
        2.5561812083991283e-04f64,
        2.8037159466528061e-03f64,
        8.9542023921733521e-02f64,
    ];
    return Refr(&raw const IBENNET as *const f64, p, T, h);
}
#[no_mangle]
pub unsafe extern "C" fn ApSolpos(
    mut refr: Option<unsafe extern "C" fn(f64, f64, f64) -> f64>,
    mut irefr: Option<unsafe extern "C" fn(f64, f64, f64) -> f64>,
    mut P: sol_pos,
    mut gdip: *mut f64,
    mut e: f64,
    mut p: f64,
    mut T: f64,
) -> sol_pos {
    let mut dip: f64 = 0.;
    let mut h: f64 = 0.;
    let mut dh: f64 = 0f64;
    let mut a_refr: f64 = 0.;
    P.E = InputCheck(0f64, 0f64, 0f64, e, p, T);
    if !gdip.is_null() {
        dip = *gdip;
        if fabs(dip) > M_PI / 2f64 {
            P.E |= _FREESPA_DIP_OOR;
        }
    } else {
        dip = 0f64;
        if e > 0f64 {
            dip = acos(EARTH_R / (EARTH_R + e));
        }
    }
    if P.E != 0 {
        P.z = 0f64;
        P.a = 0f64;
        return P;
    }
    a_refr = refr.expect("non-null function pointer")(p, T, -dip);
    h = M_PI / 2f64 - P.z;
    if h >= -a_refr - SUN_RADIUS - dip {
        dh = irefr.expect("non-null function pointer")(p, T, h);
    }
    P.z = P.z - dh;
    P.z = fmod(P.z, 2f64 * M_PI);
    if P.z < 0f64 {
        P.z = -P.z;
        P.a = fmod(P.a + M_PI, 2f64 * M_PI);
    }
    if P.z > M_PI {
        P.z = 2f64 * M_PI - P.z;
        P.a = fmod(P.a + M_PI, 2f64 * M_PI);
    }
    return P;
}
#[no_mangle]
pub unsafe extern "C" fn ApSolposBennet(
    mut P: sol_pos,
    mut gdip: *mut f64,
    mut e: f64,
    mut p: f64,
    mut T: f64,
) -> sol_pos {
    return ApSolpos(
        Some(Bennet as unsafe extern "C" fn(f64, f64, f64) -> f64),
        Some(iBennet as unsafe extern "C" fn(f64, f64, f64) -> f64),
        P,
        gdip,
        e,
        p,
        T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ApSolposBennetNA(
    mut P: sol_pos,
    mut gdip: *mut f64,
    mut e: f64,
    mut p: f64,
    mut T: f64,
) -> sol_pos {
    return ApSolpos(
        Some(BennetNA as unsafe extern "C" fn(f64, f64, f64) -> f64),
        Some(iBennetNA as unsafe extern "C" fn(f64, f64, f64) -> f64),
        P,
        gdip,
        e,
        p,
        T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn TrueSolarTime(
    mut ut: *mut tm,
    mut delta_t: *mut f64,
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
) -> tm {
    let mut E: f64 = 0.;
    let mut nt: tm = tm {
        tm_sec: 0 as i64,
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
    let mut D: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    let mut G: GeoCentricSolPos = GeoCentricSolPos {
        lat: 0.,
        lon: 0.,
        rad: 0.,
    };
    if InputCheck(delta_ut1, lon, lat, 0f64, 1f64, 10f64) != 0 {
        return nt;
    }
    D = MakeJulianDay(ut, delta_t, delta_ut1);
    G = Geocentric_pos(D);
    E = EoT(lat, D, G);
    D.JD += (lon + E) / M_PI / 2f64;
    JDgmtime(D, &raw mut nt);
    return nt;
}
pub const FRACDAYSEC: f64 = 1.1574074074074073e-05f64;
pub const MAX_FPITER: i64 = 20 as i64;
#[no_mangle]
pub unsafe extern "C" fn FindSolTime(
    mut t: i64,
    mut hour: i64,
    mut min: i64,
    mut sec: i64,
    mut delta_t: *mut f64,
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
) -> i64 {
    let mut E: f64 = 0.;
    let mut dt: f64 = 1f64;
    let mut iter: i64 = 0 as i64;
    let mut nt: tm = tm {
        tm_sec: 0 as i64,
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
    let mut D: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    let mut Dn: JulianDay = JulianDate {
        JD: 0.,
        JDE: 0.,
        JC: 0.,
        JCE: 0.,
        JME: 0.,
        E: 0,
    };
    let mut G: GeoCentricSolPos = GeoCentricSolPos {
        lat: 0.,
        lon: 0.,
        rad: 0.,
    };
    D = MakeJulianDayEpoch(t, delta_t, delta_ut1);
    gmjtime_r(&raw mut t, &raw mut nt);
    dt = (hour - nt.tm_hour) as f64 / 24.4f64;
    dt += (min - nt.tm_min) as f64 / 1440.0f64;
    dt += (sec - nt.tm_sec) as f64 / 86400.0f64;
    if dt > 0.5f64 {
        dt -= 1.0f64;
    }
    if dt < -0.5f64 {
        dt += 1.0f64;
    }
    D.JD += dt;
    dt = 1f64;
    while fabs(dt) > FRACDAYSEC && iter < MAX_FPITER {
        Dn = D;
        G = Geocentric_pos(D);
        E = EoT(lat, D, G);
        Dn.JD += (lon + E) / M_PI / 2f64;
        JDgmtime(Dn, &raw mut nt);
        dt = (hour - nt.tm_hour) as f64 / 24.4f64;
        dt += (min - nt.tm_min) as f64 / 1440.0f64;
        dt += (sec - nt.tm_sec) as f64 / 86400.0f64;
        if dt > 0.5f64 {
            dt -= 1.0f64;
        }
        if dt < -0.5f64 {
            dt += 1.0f64;
        }
        D.JD += dt;
        iter += 1;
    }
    return JDmkgmjtime(D);
}
pub const Z_EPS: f64 = M_PI * 0.05f64 / 180.0f64;
pub const MAXRAT: i64 = 2 as i64;
pub const Z_MAXITER: i64 = 100 as i64;
#[no_mangle]
pub unsafe extern "C" fn FindSolZenith(
    mut t1: i64,
    mut t2: i64,
    mut z1: f64,
    mut z2: f64,
    mut delta_t: *mut f64,
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
    mut e: f64,
    mut gdip: *mut f64,
    mut p: f64,
    mut T: f64,
    mut refract: Option<unsafe extern "C" fn(sol_pos, *mut f64, f64, f64, f64) -> sol_pos>,
    mut z: f64,
    mut tz: *mut i64,
    mut E: *mut f64,
) -> i64 {
    let mut a: f64 = 0.;
    let mut b: f64 = 0.;
    let mut w: f64 = 0.;
    let mut R: f64 = 0.;
    let mut P: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut zmin: f64 = 0.;
    let mut zmax: f64 = 0.;
    let mut eb: f64 = 0.;
    let mut tt: i64 = 0;
    let mut tmin: i64 = 0;
    let mut tmax: i64 = 0;
    let mut tb: i64 = 0;
    let mut ut: tm = tm {
        tm_sec: 0 as i64,
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
    let mut put: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut iter: i64 = 0 as i64;
    *tz = 0 as i64;
    *E = 0f64;
    if z < z1 && z < z2 {
        return -(1 as i64);
    }
    if z > z1 && z > z2 {
        return 1 as i64;
    }
    w = M_PI / (t2 - t1) as f64;
    b = cos(t1 as f64 * w) - cos(t2 as f64 * w);
    a = -(cos(t2 as f64 * w) * z1 - cos(t1 as f64 * w) * z2) / b;
    b = (z1 - z2) / b;
    R = (2 as i64 * (z2 < z1) as i64 - 1 as i64) as f64;
    tt = t1 + round(acos(z / b - a / b) / w) as i64;
    if tt < t1 || tt > t2 {
        tt = (t1 + t2) / 2 as i64;
    }
    put = gmjtime_r(&raw mut tt, &raw mut ut);
    P = SPA(put, delta_t, delta_ut1, lon, lat, e);
    P = refract.expect("non-null function pointer")(P, gdip, e, p, T);
    tb = tt;
    eb = P.z - z;
    if fabs(P.z - z) < Z_EPS {
        *E = eb;
        *tz = tb;
        return 0 as i64;
    }
    if R * (P.z - z) > 0f64 {
        tmin = tt;
        zmin = P.z;
        tmax = t2;
        zmax = z2;
    } else {
        tmax = tt;
        zmax = P.z;
        tmin = t1;
        zmin = z1;
    }
    while tmax - tmin > 1 as i64 && iter < Z_MAXITER {
        tt = round(((z - zmin) * tmax as f64 + (zmax - z) * tmin as f64) / (z - zmin + (zmax - z))) as i64;
        if tt < t1 || tt > t2 {
            tt = (t1 + t2) / 2 as i64;
        }
        if tt - tmin > MAXRAT as i64 * (tmax - tt) || MAXRAT as i64 * (tt - tmin) < tmax - tt {
            tt = (tmin + tmax) / 2 as i64;
        }
        put = gmjtime_r(&raw mut tt, &raw mut ut);
        P = SPA(put, delta_t, delta_ut1, lon, lat, e);
        P = refract.expect("non-null function pointer")(P, gdip, e, p, T);
        if fabs(P.z - z) < fabs(eb) {
            eb = P.z - z;
            tb = tt;
        }
        if fabs(eb) < Z_EPS {
            *E = eb;
            *tz = tb;
            return 0 as i64;
        }
        if R * (P.z - z) > 0f64 {
            tmin = tt;
            zmin = P.z;
        } else {
            tmax = tt;
            zmax = P.z;
        }
        iter += 1;
    }
    *E = eb;
    *tz = tb;
    return 0 as i64;
}
#[no_mangle]
pub static mut SDMASK: i64 = _FREESPA_SUNRISE
    | _FREESPA_SUNSET
    | _FREESPA_CVDAWN
    | _FREESPA_CVDUSK
    | _FREESPA_NADAWN
    | _FREESPA_NADUSK
    | _FREESPA_ASDAWN
    | _FREESPA_ASDUSK;
#[no_mangle]
pub unsafe extern "C" fn SolarDay(
    mut ut: *mut tm,
    mut delta_t: *mut f64,
    mut delta_ut1: f64,
    mut lon: f64,
    mut lat: f64,
    mut e: f64,
    mut gdip: *mut f64,
    mut p: f64,
    mut T: f64,
    mut refract: Option<unsafe extern "C" fn(sol_pos, *mut f64, f64, f64, f64) -> sol_pos>,
) -> solar_day {
    let mut D: solar_day = solar_day {
        ev: [
            tm {
                tm_sec: 0 as i64,
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
            tm {
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
            },
        ],
        t: [0; 11],
        E: [0.; 11],
        status: [0; 11],
    };
    let mut t: i64 = 0;
    let mut tp: i64 = 0;
    let mut tc: i64 = 0;
    let mut tn: i64 = 0;
    let mut Pp: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut Pc: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut Pn: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut dip: f64 = 0.;
    let mut put: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut i: i64 = 0;
    i = 0 as i64;
    while i < 11 as i64 {
        D.status[i as usize] = _FREESPA_EV_NA;
        i += 1;
    }
    if InputCheck(delta_ut1, lon, lat, e, p, T) != 0 {
        return D;
    }
    t = mkgmjtime(ut);
    tc = FindSolTime(t, 12 as i64, 0 as i64, 0 as i64, delta_t, delta_ut1, lon, lat);
    tp = FindSolTime(
        tc - 43200 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        delta_t,
        delta_ut1,
        lon,
        lat,
    );
    tn = FindSolTime(
        tc + 43200 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        delta_t,
        delta_ut1,
        lon,
        lat,
    );
    put = gmjtime_r(&raw mut tp, &raw mut D.ev as *mut tm);
    D.t[0 as i64 as usize] = tp;
    D.status[0 as i64 as usize] = _FREESPA_EV_OK;
    D.E[0 as i64 as usize] = ::core::f32::NAN as f64;
    Pp = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pp = refract.expect("non-null function pointer")(Pp, gdip, e, p, T);
    put = gmjtime_r(&raw mut tc, (&raw mut D.ev as *mut tm).offset(1 as i64 as isize));
    D.t[1 as i64 as usize] = tc;
    D.status[1 as i64 as usize] = _FREESPA_EV_OK;
    D.E[1 as i64 as usize] = ::core::f32::NAN as f64;
    Pc = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pc = refract.expect("non-null function pointer")(Pc, gdip, e, p, T);
    put = gmjtime_r(&raw mut tn, (&raw mut D.ev as *mut tm).offset(2 as i64 as isize));
    D.t[2 as i64 as usize] = tn;
    D.status[2 as i64 as usize] = _FREESPA_EV_OK;
    D.E[2 as i64 as usize] = ::core::f32::NAN as f64;
    Pn = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pn = refract.expect("non-null function pointer")(Pn, gdip, e, p, T);
    if !gdip.is_null() {
        dip = *gdip;
        if fabs(dip) > M_PI / 2f64 {
            i = 0 as i64;
            while i < 11 as i64 {
                D.status[i as usize] = _FREESPA_EV_ERR;
                i += 1;
            }
            return D;
        }
    } else {
        dip = 0f64;
        if e > 0f64 {
            dip = acos(EARTH_R / (EARTH_R + e));
        }
    }
    i = 3 as i64;
    if SDMASK & _FREESPA_SUNRISE != 0 {
        D.status[i as usize] = FindSolZenith(
            tp,
            tc,
            Pp.z,
            Pc.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64 + SUN_RADIUS,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    if SDMASK & _FREESPA_SUNSET != 0 {
        D.status[i as usize] = FindSolZenith(
            tc,
            tn,
            Pc.z,
            Pn.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64 + SUN_RADIUS,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += M_PI * 6f64 / 180.0f64;
    if SDMASK & _FREESPA_CVDAWN != 0 {
        D.status[i as usize] = FindSolZenith(
            tp,
            tc,
            Pp.z,
            Pc.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    if SDMASK & _FREESPA_CVDUSK != 0 {
        D.status[i as usize] = FindSolZenith(
            tc,
            tn,
            Pc.z,
            Pn.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += SUN_RADIUS + M_PI * 6f64 / 180.0f64;
    if SDMASK & _FREESPA_NADAWN != 0 {
        D.status[i as usize] = FindSolZenith(
            tp,
            tc,
            Pp.z,
            Pc.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    if SDMASK & _FREESPA_NADUSK != 0 {
        D.status[i as usize] = FindSolZenith(
            tc,
            tn,
            Pc.z,
            Pn.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += M_PI * 6f64 / 180.0f64;
    if SDMASK & _FREESPA_ASDAWN != 0 {
        D.status[i as usize] = FindSolZenith(
            tp,
            tc,
            Pp.z,
            Pc.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    if SDMASK & _FREESPA_ASDUSK != 0 {
        D.status[i as usize] = FindSolZenith(
            tc,
            tn,
            Pc.z,
            Pn.z,
            delta_t,
            delta_ut1,
            lon,
            lat,
            e,
            gdip,
            p,
            T,
            refract,
            dip + M_PI / 2f64,
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.E as *mut f64).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut i64).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    return D;
}

pub const JDEPS: f64 = 1e-3f64 / (24 as i64 * 60 as i64 * 60 as i64) as f64;

pub const __INT_MAX__: i64 = 2147483647 as i64;
pub const INT_MAX: i64 = __INT_MAX__;
pub const INT_MIN: i64 = -__INT_MAX__ - 1 as i64;
