use chrono::Duration;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CalculatorConfig {
    /// Whether to use astronomical chatzos.
    pub use_astronomical_chatzos: bool,
    /// Candle lighting offset in seconds.
    pub candle_lighting_offset: Duration,
    /// Whether to use astronomical chatzos for other zmanim.
    pub use_astronomical_chatzos_for_other_zmanim: bool,
    /// Ateret Torah sunset offset in minutes (default 40).
    pub ateret_torah_sunset_offset: Duration,
}

impl CalculatorConfig {
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
