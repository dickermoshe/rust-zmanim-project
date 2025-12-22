//  // some error codes for the error flag in sol_pos
//  // errors are be combined with a binary OR
//  #define _FREESPA_DEU_OOR		0X01	// ΔUT1 out of range
//  #define _FREESPA_LON_OOR		0X02	// longitude out of range
//  #define _FREESPA_LAT_OOR		0X04	// latitude out of range
//  #define _FREESPA_ELE_OOR		0X08	// elevation out of range
//  #define _FREESPA_PRE_OOR		0X10	// pressure out of range
//  #define _FREESPA_TEM_OOR		0X20	// temperature out of range
//  #define _FREESPA_DIP_OOR		0X40	// geometric dip out of range
//  #define _FREESPA_GMTIMEF		0X80	// time conversion error

//  // container struct for solar position data
//  typedef struct sol_pos {
//      double z, a; // zenith, azimuth
//      int E; // error flag
//  } sol_pos;

//  typedef struct solar_day {
//      struct tm ev[11];
//      FS_TIME_T t[11];
//      double E[11];
//      int status[11];
//  } solar_day;

//  // binary masks to enable/disable computing specific solar day events
//  #define _FREESPA_SUNRISE 0X01
//  #define _FREESPA_SUNSET  0X02
//  #define _FREESPA_CVDAWN  0X04
//  #define _FREESPA_CVDUSK  0X08
//  #define _FREESPA_NADAWN  0X10
//  #define _FREESPA_NADUSK  0X20
//  #define _FREESPA_ASDAWN  0X40
//  #define _FREESPA_ASDUSK  0X80

//  // binary mask variable to configure what solar events SolarDay computes
//  // default is all (0XFF)
//  extern int SDMASK;
//  // status flags
//  #define _FREESPA_EV_ERR       20
//  #define _FREESPA_EV_NA        10
//  #define _FREESPA_EV_OK         0
//  #define _FREESPA_EV_SUNABOVE   1
//  #define _FREESPA_EV_SUNBELOW  -1
//  // compute the real solar position
//  sol_pos SPA(struct tm *ut, double *delta_t, double delta_ut1, double lon,
//              double lat, double e);

//  // correct for atmospheric refraction
//  sol_pos ApSolposBennet(sol_pos P, double *gdip, double e, double p, double T);
//  sol_pos ApSolposBennetNA(sol_pos P, double *gdip, double e, double p, double T);

//  // compute true solar time
//  struct tm TrueSolarTime(struct tm *ut, double *delta_t, double delta_ut1,
//                          double lon, double lat);

//  // compute the solar events
//  extern int SDMASK;
//  solar_day SolarDay(struct tm *ut, double *delta_t, double delta_ut1,
//                     double lon, double lat, double e, double *gdip,
//                     double p, double T,
//                     sol_pos (*refract)(sol_pos,double*,double,double,double));

//  // julian unix time routines
//  // get delta_t value from internal tables
//  double get_delta_t(struct tm *ut);
//  // populate a time struct with UTC from unix time
//  struct tm *gmjtime_r(FS_TIME_T *t, struct tm *ut);
//  struct tm *gmjtime(FS_TIME_T *t);
//  // create unix time from time struct
//  FS_TIME_T mkgmjtime(struct tm *ut);

use chrono::Datelike;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Timelike;
use core::cell::OnceCell;
use core::f64::consts::PI;
use core::ops::Rem;
#[allow(unused_imports)]
use core_maths::*;
use std::println;

use crate::tables::*;
use crate::types::*;
pub const AP0: f64 = 1010.0f64;
pub const AT0: f64 = 10.0f64;

pub const JD0: f64 = 2451545.0;
pub const SUN_RADIUS: f64 = 4.654_269_516_293_279e-3_f64;
pub const EARTH_R: f64 = 6378136.6f64;
pub const ABSOLUTEZERO: f64 = -273.15f64;
pub const ETJD0: i64 = 946728000 as i64;
pub const FRACDAYSEC: f64 = 1.1574074074074073e-05f64;
pub const MAX_FPITER: i64 = 20;
pub const NDT: i64 = 1245;
pub const Z_EPS: f64 = PI * 0.05f64 / 180.0f64;
pub const MAXRAT: i64 = 2;
pub const Z_MAXITER: i64 = 100;
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
pub struct SpaCalculator {
    ut: NaiveDateTime,
    delta_t: Option<f64>,
    delta_ut1: f64,
    lon: f64,
    lat: f64,
    e: f64,
    julian_date: OnceCell<JulianDate>,
    geocentric_position: OnceCell<GeoCentricSolPos>,
    solar_position: OnceCell<SolarPosition>,
}

fn validate_delta_ut1(delta_ut1: f64) -> Result<(), SpaError> {
    if !(-1.0..=1.0).contains(&delta_ut1) {
        return Err(SpaError::DeltaUt1OutOfRange);
    }
    Ok(())
}
fn validate_lon(lon: f64) -> Result<(), SpaError> {
    if !(-PI..=PI).contains(&lon) {
        return Err(SpaError::LongitudeOutOfRange);
    }
    Ok(())
}
fn validate_lat(lat: f64) -> Result<(), SpaError> {
    if !(-PI / 2.0..=PI / 2.0).contains(&lat) {
        return Err(SpaError::LatitudeOutOfRange);
    }
    Ok(())
}
fn validate_elevation(elevation: f64) -> Result<(), SpaError> {
    if elevation < -EARTH_R {
        return Err(SpaError::ElevationOutOfRange);
    }
    Ok(())
}

fn validate_pressure(pressure: f64) -> Result<(), SpaError> {
    if pressure <= 0.0 || pressure > 5000.0 {
        return Err(SpaError::PressureOutOfRange);
    }

    Ok(())
}

fn validate_temperature(temperature: f64) -> Result<(), SpaError> {
    if temperature < ABSOLUTEZERO {
        return Err(SpaError::TemperatureOutOfRange);
    }
    Ok(())
}
impl SpaCalculator {
    pub fn new(
        ut: NaiveDateTime,
        delta_t: Option<f64>,
        delta_ut1: f64,
        lon_radians: f64,
        lat_radians: f64,
        elevation: f64,
    ) -> Result<Self, SpaError> {
        validate_delta_ut1(delta_ut1)?;
        validate_lon(lon_radians)?;
        validate_lat(lat_radians)?;
        validate_elevation(elevation)?;
        Ok(Self {
            ut,
            delta_t,
            delta_ut1,
            lon: lon_radians,
            lat: lat_radians,
            e: elevation,
            julian_date: OnceCell::new(),
            geocentric_position: OnceCell::new(),
            solar_position: OnceCell::new(),
        })
    }
    pub fn get_julian_day(&mut self) -> &JulianDate {
        self.julian_date
            .get_or_init(|| JulianDate::new(self.ut, self.delta_t, self.delta_ut1))
    }
    pub fn get_geocentric_position(&mut self) -> &GeoCentricSolPos {
        let julian_date = *self.get_julian_day();
        self.geocentric_position
            .get_or_init(|| GeoCentricSolPos::new(&julian_date))
    }
    pub fn get_solar_position(&mut self) -> &SolarPosition {
        let julian_date = *self.get_julian_day();
        let gp = *self.get_geocentric_position();
        self.solar_position.get_or_init(|| {
            let dtau = PI * (-20.4898f64 / 3600.0) / 180.0 / gp.rad;
            let (dpsi, deps) = Nutation_lon_obliquity(julian_date);
            let eps = deps + poly(&ECLIPTIC_MEAN_OBLIQUITY, julian_date.JME / 10.0) * (PI * (1.0 / 3600.0) / 180.0);
            let lambda = gp.lon + dpsi + dtau;
            let mut v = poly(&GSTA, julian_date.JC) + PI * 360.98564736629f64 / 180.0 * (julian_date.JD - JD0);
            v += dpsi * cos(eps);
            let mut alpha = atan2(sin(lambda) * cos(eps) - tan(gp.lat) * sin(eps), cos(lambda));
            if alpha < 0 as f64 {
                alpha += 2.0 * PI;
            }
            let delta = asin(sin(gp.lat) * cos(eps) + cos(gp.lat) * sin(eps) * sin(lambda));
            let H = v + self.lon - alpha;
            let xi = PI * (8.794f64 / 3600.0) / 180.0 / gp.rad;
            let u = atan(0.99664719f64 * tan(self.lat));
            let x = cos(u) + self.e * cos(self.lat) / EARTH_R;
            let y = 0.99664719f64 * sin(u) + self.e * sin(self.lat) / EARTH_R;
            let dalpha = atan2(-x * sin(xi) * sin(H), cos(delta) - x * sin(xi) * cos(H));
            let delta_prime = atan2(
                (sin(delta) - y * sin(xi)) * cos(dalpha),
                cos(delta) - x * sin(xi) * cos(H),
            );
            let h_prime = H - dalpha;
            let h = asin(sin(self.lat) * sin(delta_prime) + cos(self.lat) * cos(delta_prime) * cos(h_prime));
            let mut z = PI / 2.0 - h;
            let mut a = fmod(
                PI + atan2(
                    sin(h_prime),
                    cos(h_prime) * sin(self.lat) - tan(delta_prime) * cos(self.lat),
                ),
                2.0 * PI,
            );
            if z < 0.0 {
                z = -z;
                a = fmod(a + PI, 2.0 * PI);
            }
            if z > PI {
                z = 2.0 * PI - z;
                a = fmod(a + 2.0 * PI, 2.0 * PI);
            }
            SolarPosition { zenith: z, azimuth: a }
        })
    }
    pub fn get_solar_time(&mut self) -> Result<NaiveDateTime, SpaError> {
        let E = EoT(*self.get_julian_day(), *self.get_geocentric_position());
        JDgmtime(self.get_julian_day().JD + (self.lon + E) / PI / 2.0)
    }
}

#[derive(Copy, Clone)]
pub struct SolarPosition {
    pub zenith: f64,
    pub azimuth: f64,
}

impl SolarPosition {
    pub fn ApSolposBennet(&self, gdip: Option<f64>, e: f64, p: f64, T: f64) -> Result<SolarPosition, SpaError> {
        ApSolpos(Bennet, iBennet, self.clone(), gdip, e, p, T)
    }

    pub fn ApSolposBennetNA(&self, gdip: Option<f64>, e: f64, p: f64, T: f64) -> Result<SolarPosition, SpaError> {
        ApSolpos(BennetNA, iBennetNA, self.clone(), gdip, e, p, T)
    }
}

pub fn get_delta_t(ut: &NaiveDateTime) -> f64 {
    let mut imin: i64 = 0;
    let mut imax: i64 = NDT - 1;
    let dyear = ut.year() as f64 + ut.month() as f64 / 12.0 + (ut.day() as f64 - 1.0) / 365.0;
    if freespa_delta_t_table[0_usize] > dyear {
        return 32.0 * ((dyear - 1820.0) / 100f64) * ((dyear - 1820.0) / 100f64) - 20f64;
    }
    if freespa_delta_t_table[(2 * imax) as usize] < dyear {
        return 32.0 * ((dyear - 1820.0) / 100f64) * ((dyear - 1820.0) / 100f64) - 20f64;
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
    freespa_delta_t_table[(2 * imin + 1) as usize]
        + (dyear - freespa_delta_t_table[(2 * imin) as usize])
            * (freespa_delta_t_table[(2 * imax + 1) as usize] - freespa_delta_t_table[(2 * imin + 1) as usize])
            / (freespa_delta_t_table[(2 * imax) as usize] - freespa_delta_t_table[(2 * imin) as usize])
}

fn poly(a: &[f64], x: f64) -> f64 {
    let mut r = a[0];
    for i in a.iter().skip(1) {
        r = i + x * r;
    }
    r
}

fn EoT(JD: JulianDate, GP: GeoCentricSolPos) -> f64 {
    let dtau = PI * (-20.4898f64 / 3600.0f64) / 180.0f64 / GP.rad;
    let (dpsi, deps) = Nutation_lon_obliquity(JD);
    let eps = deps + poly(&ECLIPTIC_MEAN_OBLIQUITY, JD.JME / 10.0) * (PI * (1.0 / 3600.0) / 180.0);
    let lambda = GP.lon + dpsi + dtau;
    let alpha = atan2(sin(lambda) * cos(eps) - tan(GP.lat) * sin(eps), cos(lambda));
    let M = poly(&SMLON, JD.JME);
    let mut E = fmod(M - PI * 0.0057183f64 / 180.0f64 - alpha + dpsi * cos(eps), 2.0 * PI);
    if E > PI * 5.0 / 180.0 {
        E -= 2.0 * PI;
    }
    if E < -(PI * 5.0 / 180.0f64) {
        E += 2.0 * PI;
    }
    E
}

fn Nutation_lon_obliquity(JD: JulianDate) -> (f64, f64) {
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    let mut sum: f64 = 0.;
    let mut sum_psi: f64 = 0.0;
    let mut sum_eps: f64 = 0.0;
    let mut x: [f64; 5] = [0.; 5];
    x[0] = poly(&MEAN_ELONGATION_MOON_SUN, JD.JCE);
    x[1] = poly(&MEAN_ANOMALY_SUN, JD.JCE);
    x[2] = poly(&MEAN_ANOMALY_MOON, JD.JCE);
    x[3] = poly(&ARG_LAT_MOON, JD.JCE);
    x[4] = poly(&ASC_LON_MOON, JD.JCE);
    i = 0_i64;
    while i < NY {
        sum = 0.0;
        j = 0_i64;
        while j < 5_i64 {
            sum += x[j as usize] * Y_Terms[i as usize][j as usize] as f64;
            j += 1;
        }
        sum_psi += (PE_Terms[i as usize][0_i64 as usize] + JD.JCE * PE_Terms[i as usize][1_i64 as usize]) * sin(sum);
        sum_eps += (PE_Terms[i as usize][2_i64 as usize] + JD.JCE * PE_Terms[i as usize][3_i64 as usize]) * cos(sum);
        i += 1;
    }

    (
        sum_psi * (PI * (1.0 / 36000000.0) / 180.0),
        sum_eps * (PI * (1.0 / 36000000.0) / 180.0),
    )
}

fn SummPTerms(p: &[p_term], JD: &JulianDate) -> f64 {
    let mut s: f64 = 0f64;
    for p in p.iter() {
        s += p.A * cos(p.P + p.W * JD.JME);
    }
    s
}
fn Heliocentric_lon(JD: &JulianDate) -> f64 {
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
    lon
}

fn Heliocentric_lat(JD: &JulianDate) -> f64 {
    let mut lat: f64 = 0.;
    lat = SummPTerms(&EarthLat0, JD);
    lat += SummPTerms(&EarthLat1, JD) * JD.JME;
    lat /= 1.0e8f64;
    lat
}

fn Heliocentric_rad(JD: &JulianDate) -> f64 {
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
    rad
}

fn f64_to_i64(v: f64) -> Option<i32> {
    // Reject NaN and infinity first.
    if !v.is_finite() {
        return None;
    }

    // Exact bounds check using the real limits of i64.
    const MIN: f64 = i32::MIN as f64;
    const MAX: f64 = i32::MAX as f64;

    if v >= MIN && v <= MAX {
        // Safe cast: the value is in range; truncation is toward zero.
        Some(v as i32)
    } else {
        None
    }
}

fn JDgmtime(julian_day: f64) -> Result<NaiveDateTime, SpaError> {
    let z = trunc(julian_day + 0.5f64);
    let f = julian_day - z;

    let a = if z < 2299161f64 {
        z
    } else {
        let b = trunc((z - 1867216.25f64) / 36524.25f64);
        z + 1f64 + b - trunc(b / 4.0f64)
    };
    let c = a + 1524f64;
    let j = trunc((c - 122.1f64) / 365.25f64);
    let g = trunc(365.25f64 * j);
    let i = trunc((c - g) / 30.6001f64);
    let mut d = c - g - trunc(30.6001f64 * i) + f - 0.5f64;
    let tm_mday = trunc(d) as i64 + 1;
    let tm_mon = if i < 14f64 {
        (i - 2f64) as i64
    } else {
        (i - 14f64) as i64
    };
    let tm_year = if tm_mon > 1 {
        f64_to_i64(j - 4716f64)
    } else {
        f64_to_i64(j - 4715f64)
    };
    if let Some(tm_year) = tm_year {
        d -= trunc(d);
        d *= 86400f64;
        d = round(d);
        let tm_sec = d as i64 % 60;
        d -= tm_sec as f64;
        d /= 60f64;
        let tm_min = d as i64 % 60;
        d -= tm_min as f64;
        d /= 60f64;
        let tm_hour = d as i64 % 60;
        d -= tm_hour as f64;

        let result = NaiveDate::from_ymd_opt(tm_year, tm_mon as u32 + 1, tm_mday as u32)
            .and_then(|i| i.and_hms_opt(tm_hour as u32, tm_min as u32, tm_sec as u32));
        result.ok_or(SpaError::TimeConversionError)
    } else {
        Err(SpaError::TimeConversionError)
    }
}
fn gmjtime_r(t: i64) -> Result<NaiveDateTime, SpaError> {
    JDgmtime((t - ETJD0) as f64 / 86400.0f64 + JD0)
}

fn Refr(coeff: &[f64], p: f64, T: f64, h: f64) -> f64 {
    return p / AP0 * ((AT0 - ABSOLUTEZERO) / (T - ABSOLUTEZERO)) * coeff[0] / tan(h + coeff[1] / (h + coeff[2]));
}

fn Bennet(p: f64, T: f64, h: f64) -> f64 {
    return Refr(&BENNET, p, T, h);
}
fn iBennet(p: f64, T: f64, h: f64) -> f64 {
    return Refr(&IBENNET, p, T, h);
}
fn BennetNA(p: f64, T: f64, h: f64) -> f64 {
    return Refr(&BENNETNA, p, T, h);
}
fn iBennetNA(p: f64, T: f64, h: f64) -> f64 {
    return Refr(&IBENNETNA, p, T, h);
}

fn ApSolpos(
    refr: fn(f64, f64, f64) -> f64,
    irefr: fn(f64, f64, f64) -> f64,
    mut P: SolarPosition,
    gdip: Option<f64>,
    e: f64,
    p: f64,
    T: f64,
) -> Result<SolarPosition, SpaError> {
    validate_elevation(e)?;
    validate_pressure(p)?;
    validate_temperature(T)?;

    let dip = if let Some(gdip) = gdip {
        if fabs(gdip) > PI / 2.0 {
            return Err(SpaError::GeometricDipOutOfRange);
        }
        gdip
    } else if e > 0.0 {
        acos(EARTH_R / (EARTH_R + e))
    } else {
        0.0
    };

    let a_refr = refr(p, T, -dip);
    let h = PI / 2.0 - P.zenith;
    let dh = if h >= -a_refr - SUN_RADIUS - dip {
        irefr(p, T, h)
    } else {
        0.0
    };
    P.zenith -= dh;
    P.zenith = fmod(P.zenith, 2.0 * PI);
    if P.zenith < 0.0 {
        P.zenith = -P.zenith;
        P.azimuth = fmod(P.azimuth + PI, 2.0 * PI);
    }
    if P.zenith > PI {
        P.zenith = 2.0 * PI - P.zenith;
        P.azimuth = fmod(P.azimuth + PI, 2.0 * PI);
    }
    Ok(P)
}

impl JulianDate {
    pub fn new(ut: NaiveDateTime, delta_t: Option<f64>, delta_ut1: f64) -> Self {
        let day = ut.day() as f64
            + (ut.hour() as f64 + (ut.minute() as f64 + (ut.second() as f64 + delta_ut1) / 60.0) / 60.0) / 24.0;
        let mut month = ut.month();
        let mut year = ut.year();
        if month < 3 {
            month += 12;
            year -= 1;
        }
        let mut JD =
            trunc(365.25f64 * (year as f64 + 4716.0)) + trunc(30.6001f64 * (month + 1) as f64) + day - 1524.5f64;
        if JD > 2299160.0 {
            let a = trunc(year as f64 / 100.0);
            JD += 2.0 - a + trunc(a / 4.0);
        }
        let JDE = JD + delta_t.unwrap_or_else(|| get_delta_t(&ut)) / 86400.0;
        let JC = (JD - JD0) / 36525.0;
        let JCE = (JDE - JD0) / 36525.0;
        let JME = JCE / 10.0;
        JulianDate {
            JDE,
            JD,
            JC,
            JCE,
            JME,
            E: 0,
        }
    }

    pub fn from_unix_time(unix_time: i64, delta_t: Option<f64>, delta_ut1: f64) -> Result<Self, SpaError> {
        gmjtime_r(unix_time).map(|ut| Self::new(ut, delta_t, delta_ut1))
    }
}

#[derive(Copy, Clone)]
pub struct GeoCentricSolPos {
    pub lat: f64,
    pub lon: f64,
    pub rad: f64,
}

impl GeoCentricSolPos {
    pub fn new(jd: &JulianDate) -> Self {
        let lat = fmod(-Heliocentric_lat(jd), 2f64 * PI);
        let mut lon = fmod(Heliocentric_lon(jd) + PI, 2f64 * PI);
        if lon < 0f64 {
            lon += 2f64 * PI;
        }
        let rad = Heliocentric_rad(jd);
        GeoCentricSolPos { lat, lon, rad }
    }
}

pub fn FindSolTime(
    t: i64,
    hour: i64,
    min: i64,
    sec: i64,
    delta_t: Option<f64>,
    delta_ut1: f64,
    lon: f64,
) -> Result<i64, SpaError> {
    let mut D = JulianDate::from_unix_time(t, delta_t, delta_ut1)?;
    let mut nt = gmjtime_r(t)?;
    let mut dt = (hour - nt.hour() as i64) as f64 / 24.4f64;
    dt += (min - nt.minute() as i64) as f64 / 1440.0f64;
    dt += (sec - nt.second() as i64) as f64 / 86400.0f64;
    if dt > 0.5f64 {
        dt -= 1.0f64;
    }
    if dt < -0.5f64 {
        dt += 1.0f64;
    }
    D.JD += dt;
    dt = 1.0;
    let mut iter = 0;
    while fabs(dt) > FRACDAYSEC && iter < MAX_FPITER {
        let mut Dn = D;
        let G = GeoCentricSolPos::new(&D);
        let E = EoT(D, G);
        Dn.JD += (lon + E) / PI / 2.0;
        nt = JDgmtime(Dn.JD)?;
        dt = (hour - nt.hour() as i64) as f64 / 24.4f64;
        dt += (min - nt.minute() as i64) as f64 / 1440.0f64;
        dt += (sec - nt.second() as i64) as f64 / 86400.0f64;
        if dt > 0.5f64 {
            dt -= 1.0f64;
        }
        if dt < -0.5f64 {
            dt += 1.0f64;
        }
        D.JD += dt;
        iter += 1;
    }
    Ok(JDmkgmjtime(&D))
}

pub fn JDmkgmjtime(J: &JulianDate) -> i64 {
    round((J.JD - JD0) * 86400.0) as i64 + ETJD0
}

enum SolarZenith {
    AlwaysBelow,
    AlwaysAbove,
    BetweenHorizon(i64, f64),
}

fn FindSolZenith(
    t1: i64,
    t2: i64,
    z1: f64,
    z2: f64,
    delta_t: Option<f64>,
    delta_ut1: f64,
    lon: f64,
    lat: f64,
    e: f64,
    gdip: Option<f64>,
    p: f64,
    T: f64,
    refract: fn(SolarPosition, Option<f64>, f64, f64, f64) -> SolarPosition,
    z: f64,
) -> Result<SolarZenith, SpaError> {
    if z < z1 && z < z2 {
        return Ok(SolarZenith::AlwaysAbove);
    }
    if z > z1 && z > z2 {
        return Ok(SolarZenith::AlwaysBelow);
    }

    let w = PI / (t2 - t1) as f64;
    let b_denom = cos(t1 as f64 * w) - cos(t2 as f64 * w);
    let a = -(cos(t2 as f64 * w) * z1 - cos(t1 as f64 * w) * z2) / b_denom;
    let b = (z1 - z2) / b_denom;
    let R = if z2 < z1 { 1.0 } else { -1.0 };

    let mut tt = t1 + round(acos(z / b - a / b) / w) as i64;
    if tt < t1 || tt > t2 {
        tt = (t1 + t2) / 2;
    }

    let ut = gmjtime_r(tt)?;
    let mut calculator = SpaCalculator::new(ut, delta_t, delta_ut1, lon, lat, e)?;
    let mut P = refract(*calculator.get_solar_position(), gdip, e, p, T);

    let mut tb = tt;
    let mut eb = P.zenith - z;

    if fabs(P.zenith - z) < Z_EPS {
        return Ok(SolarZenith::BetweenHorizon(tb, eb));
    }

    let (mut tmin, mut zmin, mut tmax, mut zmax) = if R * (P.zenith - z) > 0.0 {
        (tt, P.zenith, t2, z2)
    } else {
        (t1, z1, tt, P.zenith)
    };

    let mut iter = 0;
    while tmax - tmin > 1 && iter < Z_MAXITER {
        tt = round(((z - zmin) * tmax as f64 + (zmax - z) * tmin as f64) / (z - zmin + (zmax - z))) as i64;
        if tt < t1 || tt > t2 {
            tt = (t1 + t2) / 2;
        }
        if tt - tmin > MAXRAT * (tmax - tt) || MAXRAT * (tt - tmin) < tmax - tt {
            tt = (tmin + tmax) / 2;
        }

        let ut = gmjtime_r(tt)?;
        let mut calculator = SpaCalculator::new(ut, delta_t, delta_ut1, lon, lat, e)?;
        P = refract(*calculator.get_solar_position(), gdip, e, p, T);

        if fabs(P.zenith - z) < fabs(eb) {
            eb = P.zenith - z;
            tb = tt;
        }
        if fabs(eb) < Z_EPS {
            return Ok(SolarZenith::BetweenHorizon(tb, eb));
        }
        if R * (P.zenith - z) > 0.0 {
            tmin = tt;
            zmin = P.zenith;
        } else {
            tmax = tt;
            zmax = P.zenith;
        }
        iter += 1;
    }
    Ok(SolarZenith::BetweenHorizon(tb, eb))
}
