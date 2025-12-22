#![allow(clippy::all)]
#![allow(warnings)]
#![allow(expect_used)]

pub const _FREESPA_DEU_OOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const _FREESPA_LON_OOR: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const _FREESPA_LAT_OOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const _FREESPA_ELE_OOR: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const _FREESPA_PRE_OOR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const _FREESPA_TEM_OOR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const _FREESPA_DIP_OOR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const _FREESPA_GMTIMEF: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const _FREESPA_SUNRISE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const _FREESPA_SUNSET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const _FREESPA_CVDAWN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const _FREESPA_CVDUSK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const _FREESPA_NADAWN: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const _FREESPA_NADUSK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const _FREESPA_ASDAWN: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const _FREESPA_ASDUSK: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const _FREESPA_EV_ERR: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const _FREESPA_EV_NA: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const _FREESPA_EV_OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const M_PI: ::core::ffi::c_double = 3.14159265358979323846f64;
pub const JD0: ::core::ffi::c_double = 2451545.0f64;
pub const ETJD0: ::core::ffi::c_int = 946728000 as ::core::ffi::c_int;
pub const SUN_RADIUS: ::core::ffi::c_double = 4.6542695162932789e-03f64;
pub const EARTH_R: ::core::ffi::c_double = 6378136.6f64;
pub const ABSOLUTEZERO: ::core::ffi::c_double = -273.15f64;
pub const AP0: ::core::ffi::c_double = 1010.0f64;
pub const AT0: ::core::ffi::c_double = 10.0f64;

pub const FRACDAYSEC: ::core::ffi::c_double = 1.1574074074074073e-05f64;
pub const MAX_FPITER: ::core::ffi::c_int = 20 as ::core::ffi::c_int;

pub const Z_EPS: ::core::ffi::c_double = M_PI * 0.05f64 / 180.0f64;
pub const MAXRAT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const Z_MAXITER: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn FindSolZenith(
    mut t1: time_t,
    mut t2: time_t,
    mut z1: ::core::ffi::c_double,
    mut z2: ::core::ffi::c_double,
    mut delta_t: *mut ::core::ffi::c_double,
    mut delta_ut1: ::core::ffi::c_double,
    mut lon: ::core::ffi::c_double,
    mut lat: ::core::ffi::c_double,
    mut e: ::core::ffi::c_double,
    mut gdip: *mut ::core::ffi::c_double,
    mut p: ::core::ffi::c_double,
    mut T: ::core::ffi::c_double,
    mut refract: Option<
        unsafe extern "C" fn(
            sol_pos,
            *mut ::core::ffi::c_double,
            ::core::ffi::c_double,
            ::core::ffi::c_double,
            ::core::ffi::c_double,
        ) -> sol_pos,
    >,
    mut z: ::core::ffi::c_double,
    mut tz: *mut time_t,
    mut E: *mut ::core::ffi::c_double,
) -> ::core::ffi::c_int {
    let mut a: ::core::ffi::c_double = 0.;
    let mut b: ::core::ffi::c_double = 0.;
    let mut w: ::core::ffi::c_double = 0.;
    let mut R: ::core::ffi::c_double = 0.;
    let mut P: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut zmin: ::core::ffi::c_double = 0.;
    let mut zmax: ::core::ffi::c_double = 0.;
    let mut eb: ::core::ffi::c_double = 0.;
    let mut tt: time_t = 0;
    let mut tmin: time_t = 0;
    let mut tmax: time_t = 0;
    let mut tb: time_t = 0;
    let mut ut: tm = tm {
        tm_sec: 0 as ::core::ffi::c_int,
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
    let mut iter: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    *tz = 0 as time_t;
    *E = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
    if z < z1 && z < z2 {
        return -(1 as ::core::ffi::c_int);
    }
    if z > z1 && z > z2 {
        return 1 as ::core::ffi::c_int;
    }
    w = M_PI / (t2 - t1) as ::core::ffi::c_double;
    b = cos(t1 as ::core::ffi::c_double * w) - cos(t2 as ::core::ffi::c_double * w);
    a = -(cos(t2 as ::core::ffi::c_double * w) * z1 - cos(t1 as ::core::ffi::c_double * w) * z2) / b;
    b = (z1 - z2) / b;
    R = (2 as ::core::ffi::c_int * (z2 < z1) as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as ::core::ffi::c_double;
    tt = t1 + round(acos(z / b - a / b) / w) as time_t;
    if tt < t1 || tt > t2 {
        tt = (t1 + t2) / 2 as time_t;
    }
    put = gmjtime_r(&raw mut tt, &raw mut ut);
    P = SPA(put, delta_t, delta_ut1, lon, lat, e);
    P = refract.expect("non-null function pointer")(P, gdip, e, p, T);
    tb = tt;
    eb = P.z - z;
    if fabs(P.z - z) < Z_EPS {
        *E = eb;
        *tz = tb;
        return 0 as ::core::ffi::c_int;
    }
    if R * (P.z - z) > 0 as ::core::ffi::c_int as ::core::ffi::c_double {
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
    while tmax - tmin > 1 as time_t && iter < Z_MAXITER {
        tt = round(
            ((z - zmin) * tmax as ::core::ffi::c_double + (zmax - z) * tmin as ::core::ffi::c_double)
                / (z - zmin + (zmax - z)),
        ) as time_t;
        if tt < t1 || tt > t2 {
            tt = (t1 + t2) / 2 as time_t;
        }
        if tt - tmin > MAXRAT as time_t * (tmax - tt) || MAXRAT as time_t * (tt - tmin) < tmax - tt {
            tt = (tmin + tmax) / 2 as time_t;
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
            return 0 as ::core::ffi::c_int;
        }
        if R * (P.z - z) > 0 as ::core::ffi::c_int as ::core::ffi::c_double {
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
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub static mut SDMASK: ::core::ffi::c_int = _FREESPA_SUNRISE
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
    mut delta_t: *mut ::core::ffi::c_double,
    mut delta_ut1: ::core::ffi::c_double,
    mut lon: ::core::ffi::c_double,
    mut lat: ::core::ffi::c_double,
    mut e: ::core::ffi::c_double,
    mut gdip: *mut ::core::ffi::c_double,
    mut p: ::core::ffi::c_double,
    mut T: ::core::ffi::c_double,
    mut refract: Option<
        unsafe extern "C" fn(
            sol_pos,
            *mut ::core::ffi::c_double,
            ::core::ffi::c_double,
            ::core::ffi::c_double,
            ::core::ffi::c_double,
        ) -> sol_pos,
    >,
) -> solar_day {
    let mut D: solar_day = solar_day {
        ev: [
            tm {
                tm_sec: 0 as ::core::ffi::c_int,
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
    let mut t: time_t = 0;
    let mut tp: time_t = 0;
    let mut tc: time_t = 0;
    let mut tn: time_t = 0;
    let mut Pp: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut Pc: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut Pn: sol_pos = sol_pos { z: 0., a: 0., E: 0 };
    let mut dip: ::core::ffi::c_double = 0.;
    let mut put: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 11 as ::core::ffi::c_int {
        D.status[i as usize] = _FREESPA_EV_NA;
        i += 1;
    }
    if InputCheck(delta_ut1, lon, lat, e, p, T) != 0 {
        return D;
    }
    t = mkgmjtime(ut);
    tc = FindSolTime(
        t,
        12 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        delta_t,
        delta_ut1,
        lon,
        lat,
    );
    tp = FindSolTime(
        tc - 43200 as time_t,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        delta_t,
        delta_ut1,
        lon,
        lat,
    );
    tn = FindSolTime(
        tc + 43200 as time_t,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        delta_t,
        delta_ut1,
        lon,
        lat,
    );
    put = gmjtime_r(&raw mut tp, &raw mut D.ev as *mut tm);
    D.t[0 as ::core::ffi::c_int as usize] = tp;
    D.status[0 as ::core::ffi::c_int as usize] = _FREESPA_EV_OK;
    D.E[0 as ::core::ffi::c_int as usize] = ::core::f32::NAN as ::core::ffi::c_double;
    Pp = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pp = refract.expect("non-null function pointer")(Pp, gdip, e, p, T);
    put = gmjtime_r(
        &raw mut tc,
        (&raw mut D.ev as *mut tm).offset(1 as ::core::ffi::c_int as isize),
    );
    D.t[1 as ::core::ffi::c_int as usize] = tc;
    D.status[1 as ::core::ffi::c_int as usize] = _FREESPA_EV_OK;
    D.E[1 as ::core::ffi::c_int as usize] = ::core::f32::NAN as ::core::ffi::c_double;
    Pc = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pc = refract.expect("non-null function pointer")(Pc, gdip, e, p, T);
    put = gmjtime_r(
        &raw mut tn,
        (&raw mut D.ev as *mut tm).offset(2 as ::core::ffi::c_int as isize),
    );
    D.t[2 as ::core::ffi::c_int as usize] = tn;
    D.status[2 as ::core::ffi::c_int as usize] = _FREESPA_EV_OK;
    D.E[2 as ::core::ffi::c_int as usize] = ::core::f32::NAN as ::core::ffi::c_double;
    Pn = SPA(put, delta_t, delta_ut1, lon, lat, e);
    Pn = refract.expect("non-null function pointer")(Pn, gdip, e, p, T);
    if !gdip.is_null() {
        dip = *gdip;
        if fabs(dip) > M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double {
            i = 0 as ::core::ffi::c_int;
            while i < 11 as ::core::ffi::c_int {
                D.status[i as usize] = _FREESPA_EV_ERR;
                i += 1;
            }
            return D;
        }
    } else {
        dip = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
        if e > 0 as ::core::ffi::c_int as ::core::ffi::c_double {
            dip = acos(EARTH_R / (EARTH_R + e));
        }
    }
    i = 3 as ::core::ffi::c_int;
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double + SUN_RADIUS,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double + SUN_RADIUS,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += M_PI * 6 as ::core::ffi::c_int as ::core::ffi::c_double / 180.0f64;
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += SUN_RADIUS + M_PI * 6 as ::core::ffi::c_int as ::core::ffi::c_double / 180.0f64;
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    dip += M_PI * 6 as ::core::ffi::c_int as ::core::ffi::c_double / 180.0f64;
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
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
            dip + M_PI / 2 as ::core::ffi::c_int as ::core::ffi::c_double,
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.E as *mut ::core::ffi::c_double).offset(i as isize),
        );
        put = gmjtime_r(
            (&raw mut D.t as *mut time_t).offset(i as isize),
            (&raw mut D.ev as *mut tm).offset(i as isize),
        );
    }
    i += 1;
    return D;
}

pub const JDEPS: ::core::ffi::c_double =
    1e-3f64 / (24 as ::core::ffi::c_int * 60 as ::core::ffi::c_int * 60 as ::core::ffi::c_int) as ::core::ffi::c_double;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const INT_MIN: ::core::ffi::c_int = -__INT_MAX__ - 1 as ::core::ffi::c_int;
