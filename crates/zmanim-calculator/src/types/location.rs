use chrono::{TimeZone, Utc};

/// A geographic location (latitude/longitude/elevation) and an optional timezone.
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
    /// # Allowed values
    ///
    /// - `latitude` must be a finite number in `[-90.0, 90.0]` (degrees; positive = North).
    /// - `longitude` must be a finite number in `[-180.0, 180.0]` (degrees; positive = East).
    /// - `elevation` must be a finite number `>= 0.0` (meters above mean sea level).
    /// - `timezone` is required for locations near the anti-meridian.
    ///   Calculations for Kiddush Levana times require a timezone. Ommitting a timezone will result in None being returned.
    ///
    /// If any of the values are invalid, this returns `None`.
    ///
    /// # Anti-meridian behavior
    ///
    /// If `timezone` is `None` and `longitude` is near the anti-meridian (currently `abs(longitude) > 150°`),
    /// this returns `None`.
    ///
    /// This mirrors the real-world situation where longitude alone does not uniquely determine the civil day
    /// near the International Date Line; providing a timezone disambiguates the intended date.
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
        assert!(location.is_none());
    }

    #[test]
    fn test_location_rejects_out_of_range_coords() {
        let bad_longitude = Location::new(0.0, 181.0, 0.0, Some(Utc));
        assert!(bad_longitude.is_none());

        let bad_latitude = Location::new(91.0, 0.0, 0.0, Some(Utc));
        assert!(bad_latitude.is_none());
    }

    #[test]
    fn test_location_rejects_negative_elevation() {
        let location = Location::new(0.0, 0.0, -1.0, Some(Utc));
        assert!(location.is_none());
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
