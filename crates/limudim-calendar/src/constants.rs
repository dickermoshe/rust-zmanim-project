//! Named constants for cycle lengths and other magic numbers.

/// Total number of amudim (half-pages) in the Babylonian Talmud for Dirshu
pub const BAVLI_TOTAL_AMUDIM: i32 = 5406;

/// Number of dafim in Daf Yomi Bavli cycles 1-7 (before Shekalim expansion)
pub const BAVLI_DAF_COUNT_EARLY: i32 = 2702;

/// Number of dafim in Daf Yomi Bavli cycles 8+ (after Shekalim expansion)
pub const BAVLI_DAF_COUNT_MODERN: i32 = 2711;

/// Number of dafim in the Yerushalmi Talmud
pub const YERUSHALMI_DAF_COUNT: i32 = 1554;

/// Number of days in a Mishna Yomis cycle (4192 mishnas / 2 per day)
pub const MISHNA_YOMIS_CYCLE_DAYS: i32 = 2095;

/// Cycle number at which Shekalim expanded from 13 to 22 pages
pub const SHEKALIM_EXPANSION_CYCLE: i32 = 8;
