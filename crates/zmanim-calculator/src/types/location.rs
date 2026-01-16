use chrono::{TimeZone, Utc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location<T: TimeZone = Utc> {
    /// Latitude in degrees (positive North, negative South)
    pub latitude: f64,
    /// Longitude in degrees (positive East, negative West)
    pub longitude: f64,
    /// Elevation above sea level in meters
    pub elevation: f64,
    /// Timezone of the location.
    pub(crate) timezone: Option<T>,
}

impl<T: TimeZone> Location<T> {
    /// Creates a new Location.
    ///
    /// Returns `None` if the location is near the anti-meridian (±150° longitude)
    /// and no timezone is provided, since calculating local noon from UTC using
    /// longitude offset alone becomes unreliable near date boundaries.
    pub fn new(latitude: f64, longitude: f64, elevation: f64, timezone: Option<T>) -> Option<Self> {
        if timezone.is_none() && Self::near_anti_meridian(longitude) {
            return None;
        }

        if longitude.abs() > 180.0 || longitude.is_nan() {
            return None;
        }
        if latitude.abs() > 90.0 || latitude.is_nan() {
            return None;
        }
        if elevation.is_nan() || elevation < 0.0 {
            return None;
        }

        Some(Self {
            latitude,
            longitude,
            elevation,
            timezone,
        })
    }
    pub fn near_anti_meridian(longitude: f64) -> bool {
        const ANTI_MERIDIAN_THRESHOLD: f64 = 150.0;
        longitude.abs() > ANTI_MERIDIAN_THRESHOLD
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for Location<Utc> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Location {{ latitude: {}, longitude: {}, elevation: {}, timezone: UTC }}",
            self.latitude,
            self.longitude,
            self.elevation,
        )
    }
}
#[cfg(all(feature = "tz", feature = "defmt"))]
impl defmt::Format for Location<chrono_tz::Tz> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Location {{ latitude: {}, longitude: {}, elevation: {}, timezone: {:?} }}",
            self.latitude,
            self.longitude,
            self.elevation,
            self.timezone.map(|l| l.name())
        )
    }
}
