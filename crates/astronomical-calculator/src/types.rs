use thiserror::Error;

#[derive(Copy, Clone)]
pub struct p_term {
    pub A: f64,
    pub P: f64,
    pub W: f64,
}

#[derive(Copy, Clone)]
pub struct tm {
    pub tm_sec: i64,
    pub tm_min: i64,
    pub tm_hour: i64,
    pub tm_mday: i64,
    pub tm_mon: i64,
    pub tm_year: i64,
    pub tm_wday: i64,
    pub tm_yday: i64,
    pub tm_isdst: i64,
    pub tm_gmtoff: i64,
    pub tm_zone: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
pub struct solar_day {
    pub ev: [tm; 11],
    pub t: [i64; 11],
    pub E: [f64; 11],
    pub status: [i64; 11],
}
#[derive(Copy, Clone)]
pub struct JulianDate {
    pub JD: f64,
    pub JDE: f64,
    pub JC: f64,
    pub JCE: f64,
    pub JME: f64,
    pub E: i64,
}

#[derive(Error, Debug)]
pub enum SpaError {
    #[error("ΔUT1 out of range")]
    DeltaUt1OutOfRange,

    #[error("Longitude out of range")]
    LongitudeOutOfRange,

    #[error("Latitude out of range")]
    LatitudeOutOfRange,

    #[error("Elevation out of range")]
    ElevationOutOfRange,

    #[error("Pressure out of range")]
    PressureOutOfRange,

    #[error("Temperature out of range")]
    TemperatureOutOfRange,

    #[error("Geometric dip out of range")]
    GeometricDipOutOfRange,

    #[error("Time conversion error")]
    TimeConversionError,
}
