#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod java_tests;
#[cfg(test)]
mod zmanim_calendar_tests;
mod types {
    pub mod config;
    pub mod location;
    pub mod zman;
}
mod calculator;
mod math;
mod zmanim;
mod zmanim_impl;

pub use calculator::ZmanimCalculator;
pub use types::config::CalculatorConfig;
pub use types::location::Location;
pub use zmanim::*;
