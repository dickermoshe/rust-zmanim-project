use chrono::Duration;

/// Parameters that control how zmanim are calculated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CalculatorConfig {
    /// Offset subtracted from sea-level sunset to produce [`crate::presets::CANDLE_LIGHTING`]. Default: 18 min.
    pub candle_lighting_offset: Duration,

    /// When `true`, derived zmanim (sof zman shma, mincha gedola, etc.) are computed
    /// relative to solar noon rather than as a fraction of the sunrise–sunset interval. Default: `false`.
    pub use_astronomical_chatzos_for_other_zmanim: bool,

    /// Offset added to elevation-adjusted sunset for the Ateret Torah opinion (see [`crate::presets::TZAIS_ATERET_TORAH`]). Default: 40 min.
    pub ateret_torah_sunset_offset: Duration,
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            candle_lighting_offset: Duration::minutes(18),
            use_astronomical_chatzos_for_other_zmanim: false,
            ateret_torah_sunset_offset: Duration::minutes(40),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CalculatorConfig {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CalculatorConfig {{ candle_lighting_offset: {}, use_astronomical_chatzos_for_other_zmanim: {}, ateret_torah_sunset_offset: {} }}", self.candle_lighting_offset.as_seconds_f64(), self.use_astronomical_chatzos_for_other_zmanim, self.ateret_torah_sunset_offset.as_seconds_f64())
    }
}
