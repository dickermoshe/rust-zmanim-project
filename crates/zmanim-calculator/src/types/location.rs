use chrono::{TimeZone, Utc};

use crate::types::error::ZmanimError;

/// A geographic location (latitude/longitude/elevation) and an optional timezone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location<T: TimeZone = Utc> {
    /// Latitude in degrees. Valid range: `[-90.0, 90.0]` (positive = North).
    pub latitude: f64,
    /// Longitude in degrees. Valid range: `[-180.0, 180.0]` (positive = East).
    /// Must be provided with a timezone when `abs(longitude) > 150°`.
    pub longitude: f64,
    /// Elevation above sea level in meters. Must be `>= 0.0`.
    pub elevation: f64,
    /// Timezone of the location. Required when near the anti-meridian (`abs(longitude) > 150°`).
    /// Also required for calculating kiddush levena times.
    pub timezone: Option<T>,
}

impl<T: TimeZone> Location<T> {
    /// Creates a new `Location`, returning a [`ZmanimError`] if any value is out of range.
    ///
    /// # Errors
    /// - [`ZmanimError::InvalidLatitude`] — `latitude` outside `[-90.0, 90.0]`
    /// - [`ZmanimError::InvalidLongitude`] — `longitude` outside `[-180.0, 180.0]`
    /// - [`ZmanimError::InvalidElevation`] — `elevation` below `0.0`
    /// - [`ZmanimError::TimeZoneRequired`] — `timezone` is `None` and `abs(longitude) > 150°`
    pub fn new(
        latitude: f64,
        longitude: f64,
        elevation: f64,
        timezone: Option<T>,
    ) -> Result<Self, ZmanimError> {
        if timezone.is_none() && Self::near_anti_meridian(longitude) {
            return Err(ZmanimError::TimeZoneRequired);
        }

        if longitude.abs() > 180.0 || longitude.is_nan() {
            return Err(ZmanimError::InvalidLongitude);
        }
        if latitude.abs() > 90.0 || latitude.is_nan() {
            return Err(ZmanimError::InvalidLatitude);
        }
        if elevation.is_nan() || elevation < 0.0 {
            return Err(ZmanimError::InvalidElevation);
        }

        Ok(Self {
            latitude,
            longitude,
            elevation,
            timezone,
        })
    }

    pub(crate) fn near_anti_meridian(longitude: f64) -> bool {
        const ANTI_MERIDIAN_THRESHOLD: f64 = 150.0;
        longitude.abs() > ANTI_MERIDIAN_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_location_rejects_anti_meridian_without_timezone() {
        let location = Location::new(0.0, 150.1, 0.0, Option::<Utc>::None);
        assert!(location.is_err());
    }

    #[test]
    fn test_location_rejects_out_of_range_coords() {
        let bad_longitude = Location::new(0.0, 181.0, 0.0, Some(Utc));
        assert!(bad_longitude.is_err());

        let bad_latitude = Location::new(91.0, 0.0, 0.0, Some(Utc));
        assert!(bad_latitude.is_err());
    }

    #[test]
    fn test_location_rejects_negative_elevation() {
        let location = Location::new(0.0, 0.0, -1.0, Some(Utc));
        assert!(location.is_err());
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
