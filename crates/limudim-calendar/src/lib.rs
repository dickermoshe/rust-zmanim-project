//! # Limudim - Jewish Learning Schedule Calculator
//!
//! This library provides calculators for various Jewish learning schedules including:
//! - Daf Yomi Bavli (Babylonian Talmud daily page)
//! - Daf Yomi Yerushalmi (Jerusalem Talmud daily page)
//! - Amud Yomi Bavli Dirshu (Babylonian Talmud daily column - Dirshu schedule)
//! - Daf Hashavua Bavli (Babylonian Talmud weekly page)
//! - Mishna Yomis (Daily Mishna)
//! - Pirkei Avos (Ethics of the Fathers - seasonal schedule)
//! - Tehillim Monthly (Monthly Psalms reading)
//!
//! ## Example
//! ```
//! use limudim_calendar::{DafYomiBavli, LimudCalculator};
//! use icu_calendar::{cal::Hebrew, Date};
//! use limudim_calendar::LimudCalendar;
//!
//! let date = Date::try_new_iso(2020, 1, 5).unwrap().to_calendar(Hebrew);
//! let daf = date.limud(DafYomiBavli::default());
//! ```

#![no_std]
#![warn(missing_docs)]

use icu_calendar::{cal::Hebrew, Date};

mod amud_yomi_bavli_dirshu;
mod constants;
mod cycle;
mod daf_hashavua_bavli;
mod daf_yomi_bavli;
mod daf_yomi_yerushalmi;
mod date;
mod interval;
mod limud_calculator;
mod mishna_yomis;
mod pirkei_avos;
mod tehillim_monthly;
mod units;

/// Extension trait for Hebrew dates to calculate limud schedules.
///
/// This trait extends `Date<Hebrew>` with the ability to calculate
/// what should be learned on any given date according to various
/// Jewish learning schedules.
pub trait LimudCalendar {
    /// Calculate the limud (learning unit) for this date using the given calculator.
    ///
    /// # Arguments
    /// * `limud_calculator` - A calculator implementing the `LimudCalculator` trait
    ///
    /// # Returns
    /// The learning unit for this date, or `None` if no learning is scheduled
    fn limud<T>(&self, limud_calculator: impl LimudCalculator<T>) -> Option<T>;
}
impl LimudCalendar for Date<Hebrew> {
    fn limud<T>(&self, limud_calculator: impl LimudCalculator<T>) -> Option<T> {
        limud_calculator.limud(*self)
    }
}
// Calculators
pub use amud_yomi_bavli_dirshu::AmudYomiBavliDirshu;
pub use daf_hashavua_bavli::DafHashavuaBavli;
pub use daf_yomi_bavli::DafYomiBavli;
pub use daf_yomi_yerushalmi::DafYomiYerushalmiVilna;
pub use mishna_yomis::{MishnaYomis, Mishnas};
pub use pirkei_avos::{PirkeiAvos, PirkeiAvosUnit};
pub use tehillim_monthly::{TehillimMonthly, TehillimUnit};

// Unit types
pub use units::{Amud, Daf, Mishna, Side, Tractate};

// Traits
pub use limud_calculator::LimudCalculator;
