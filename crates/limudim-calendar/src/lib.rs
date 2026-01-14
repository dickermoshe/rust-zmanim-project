#![no_std]

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

pub trait LimudCalendar {
    fn limud<T: LimudCalculator<T>>(&self, limud_calculator: T) -> Option<T>;
}
impl LimudCalendar for Date<Hebrew> {
    fn limud<T: LimudCalculator<T>>(&self, limud_calculator: T) -> Option<T> {
        limud_calculator.limud(*self)
    }
}
// Calculators
pub use amud_yomi_bavli_dirshu::AmudYomiBavliDirshu;
pub use daf_hashavua_bavli::DafHashavuaBavli;
pub use daf_yomi_bavli::DafYomiBavli;
pub use daf_yomi_yerushalmi::DafYomiYerushalmi;
pub use mishna_yomis::{MishnaYomis, Mishnas};
pub use pirkei_avos::{PirkeiAvos, PirkeiAvosUnit};
pub use tehillim_monthly::{TehillimMonthly, TehillimUnit};

// Unit types
pub use units::{Amud, Daf, Mishna, Side, Tractate};

// Traits
pub use limud_calculator::LimudCalculator;
