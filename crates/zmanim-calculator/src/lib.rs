#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
//! Calculate zmanim (Jewish halachic times) using the KosherJava Zmanim API’s concepts and naming.
//!
//! This crate exposes:
//! - [`Location`]: where to calculate for.
//! - [`CalculatorConfig`]: configuration knobs (candle lighting offset, chatzos behavior, etc.).
//! - [`ZmanimCalculator`]: the engine for computing times for a given date + location.
//! - A large set of predefined [`Zman`] constants in [`zman`].
//!
//! `ZmanimCalculator::calculate()` returns `Option<DateTime<Utc>>` since some locations/dates (e.g. polar regions)
//! have no sunrise/sunset or twilight for a given definition.
#[cfg(test)]
mod java_tests;
#[cfg(test)]
mod zmanim_calendar_tests;
mod types {
    /// Configuration types for zmanim calculations.
    pub mod config;
    /// Location types for zmanim calculations.
    pub mod location;
}
mod calculator;
mod math;
mod zmanim_impl;

/// Zmanim constants and definitions.
pub mod zman;
pub use calculator::ZmanimCalculator;
pub use types::config::CalculatorConfig;
pub use types::location::Location;
