#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
//! Calculate zmanim (Jewish halachic times) using KosherJava-style concepts and naming.
//!
//! Typical usage:
//! - Build a [`Location`](crate::types::location::Location).
//! - Create a [`ZmanimCalculator`](crate::calculator::ZmanimCalculator) with a date and
//!   [`CalculatorConfig`](crate::types::config::CalculatorConfig).
//! - Compute times using ready-made definitions from [`presets`]
//!   (for example, `presets::SUNRISE` and `presets::SUNSET`).
//!
//! [`ZmanimCalculator::calculate`](crate::calculator::ZmanimCalculator::calculate) returns
//! `Result<DateTime<Utc>, ZmanimError>`.
//! In edge cases (for example high latitudes on specific dates), calculations may return an error.
//!
//! `calculate` takes `&mut self` so repeated calculations can reuse intermediate state.
//! If Rust borrow rules are awkward for your call pattern, clone the calculator and use each
//! clone independently.
#[cfg(test)]
mod java_tests;
#[cfg(test)]
mod tests;
mod types {
    /// Configuration types for zmanim calculations.
    pub mod config;
    /// Error types for zmanim calculations.
    pub mod error;
    /// Location types for zmanim calculations.
    pub mod location;
}
// Calculation logic for zmanim.
pub mod calculator;
mod duration_helper;
/// Predefined zmanim calculations built from reusable primitives.
pub mod presets;
/// Low-level zman formulas used to build higher-level presets.
pub mod primitive_zman;

#[cfg(feature = "c")]
/// C API for zmanim calculations.
pub mod c_api;
/// Re-export the most commonly used types and traits.
pub mod prelude {
    pub use crate::calculator::ZmanimCalculator;
    pub use crate::presets::ZmanPreset;
    pub use crate::primitive_zman::ZmanPrimitive;
    pub use crate::types::config::CalculatorConfig;
    pub use crate::types::error::ZmanimError;
    pub use crate::types::location::Location;
}
