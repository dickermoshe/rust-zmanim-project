use chrono::Duration;

/// Configuration options controlling how derived zmanim are calculated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CalculatorConfig {
    /// Whether `CHATZOS_ASTRONOMICAL` is computed using true astronomical transit (solar noon).
    ///
    /// When `true`, [`crate::CHATZOS_ASTRONOMICAL`] uses the astronomical calculator’s solar
    /// transit time. When `false`, you can instead use other chatzos definitions such as
    /// [`crate::CHATZOS_HALF_DAY`] or [`crate::CHATZOS_FIXED_LOCAL`].
    ///
    /// Note that many zmanim do not depend on chatzos at all; this primarily affects chatzos itself
    /// and, when combined with [`Self::use_astronomical_chatzos_for_other_zmanim`], may affect other
    /// derived zmanim.
    pub use_astronomical_chatzos: bool,

    /// Candle lighting offset (typically *before* sunset).
    ///
    /// This value is subtracted from sea-level sunset to produce [`crate::CANDLE_LIGHTING`].
    /// A typical value is `Duration::minutes(18)`.
    pub candle_lighting_offset: Duration,

    /// Whether to split the day around astronomical chatzos for certain derived zmanim.
    ///
    /// When `false`, derived zmanim like sof zman shma / mincha gedola / plag hamincha are generally
    /// computed as a fraction of the time between two endpoints (often sunrise → sunset).
    ///
    /// When `true`, zmanim that are computed in a "synchronous" way (based on sunrise/sunset) may
    /// instead be computed by splitting the day at astronomical chatzos (solar transit) and
    /// computing offsets relative to the appropriate half-day. This can change results slightly as
    /// the day lengthens/shortens through the seasons because the two halves of the day are not
    /// always equal.
    pub use_astronomical_chatzos_for_other_zmanim: bool,

    /// Ateret Torah sunset offset (typically *after* sunset).
    ///
    /// This value is added to elevation-adjusted sunset to produce the "Ateret Torah" sunset used
    /// by zmanim such as [`crate::TZAIS_ATERET_TORAH`]. A typical value is `Duration::minutes(40)`.
    pub ateret_torah_sunset_offset: Duration,
}

impl CalculatorConfig {
    /// Creates a new configuration with explicit values for all fields.
    ///
    /// No validation is performed here; values are interpreted by the zmanim that use them.
    pub fn new(
        use_astronomical_chatzos: bool,
        candle_lighting_offset: Duration,
        use_astronomical_chatzos_for_other_zmanim: bool,
        ateret_torah_sunset_offset: Duration,
    ) -> Self {
        Self {
            use_astronomical_chatzos,
            candle_lighting_offset,
            use_astronomical_chatzos_for_other_zmanim,
            ateret_torah_sunset_offset,
        }
    }
}

impl Default for CalculatorConfig {
    /// Returns a reasonable set of defaults matching common published calendars.
    ///
    /// Defaults:
    /// - `use_astronomical_chatzos = true`
    /// - `candle_lighting_offset = 18 minutes`
    /// - `use_astronomical_chatzos_for_other_zmanim = false`
    /// - `ateret_torah_sunset_offset = 40 minutes`
    fn default() -> Self {
        Self {
            use_astronomical_chatzos: true,
            candle_lighting_offset: Duration::minutes(18),
            use_astronomical_chatzos_for_other_zmanim: false,
            ateret_torah_sunset_offset: Duration::minutes(40),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CalculatorConfig {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CalculatorConfig {{ use_astronomical_chatzos: {}, candle_lighting_offset: {}, use_astronomical_chatzos_for_other_zmanim: {}, ateret_torah_sunset_offset: {} }}", self.use_astronomical_chatzos, self.candle_lighting_offset.as_seconds_f64(), self.use_astronomical_chatzos_for_other_zmanim, self.ateret_torah_sunset_offset.as_seconds_f64())
    }
}
