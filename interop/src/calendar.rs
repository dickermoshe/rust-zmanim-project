//! Calendar FFI bridge.

#[diplomat::bridge]
mod ffi {
    use icu_calendar::cal::Hebrew;
    use kosher_rust::calendar::{HebrewCalendar, HebrewHolidayCalendar};

    use crate::common::{
        civil_date_to_jiff, decode_hebrew_month, gregorian_to_hebrew as common_gregorian_to_hebrew,
        hebrew_to_gregorian as common_hebrew_to_gregorian, holiday_to_codes,
    };

    /// Gregorian civil date (year, month, day).
    pub struct CivilDate {
        pub year: i32,
        pub month: u8,
        pub day: u8,
    }

    /// Hebrew calendar date (year, month code, day).
    pub struct HebrewDate {
        pub year: i32,
        pub month: u8,
        pub day: u8,
    }

    /// A holiday occurring on a date, with an optional sub-day (Chanukah, Omer).
    #[derive(Copy, Clone)]
    pub struct HolidayEntry {
        pub code: HolidayCode,
        /// Day within Chanukah (1–8) or Omer (1–49); zero otherwise.
        pub sub_day: u8,
    }

    pub enum HolidayCode {
        ErevPesach,
        Pesach,
        CholHamoedPesach,
        PesachSheni,
        ErevShavuos,
        Shavuos,
        SeventeenthOfTammuz,
        TishahBav,
        TuBav,
        ErevRoshHashana,
        RoshHashana,
        FastOfGedalyah,
        ErevYomKippur,
        YomKippur,
        ErevSuccos,
        Succos,
        CholHamoedSuccos,
        HoshanaRabbah,
        SheminiAtzeres,
        SimchasTorah,
        Chanukah,
        TenthOfTeves,
        TuBshvat,
        FastOfEsther,
        Purim,
        ShushanPurim,
        PurimKatan,
        RoshChodesh,
        YomHaShoah,
        YomHazikaron,
        YomHaatzmaut,
        YomYerushalayim,
        LagBomer,
        ShushanPurimKatan,
        IsruChag,
        YomKippurKatan,
        Behab,
        FastOfTheFirstborn,
        CountOfTheOmer,
        BirchasHachamah,
        MacharHachodesh,
        ShabbosMevarchim,
    }

    pub enum ParshaCode {
        Bereshis,
        Noach,
        LechLecha,
        Vayera,
        ChayeiSara,
        Toldos,
        Vayetzei,
        Vayishlach,
        Vayeshev,
        Miketz,
        Vayigash,
        Vayechi,
        Shemos,
        Vaera,
        Bo,
        Beshalach,
        Yisro,
        Mishpatim,
        Terumah,
        Tetzaveh,
        KiSisa,
        Vayakhel,
        Pekudei,
        Vayikra,
        Tzav,
        Shmini,
        Tazria,
        Metzora,
        AchreiMos,
        Kedoshim,
        Emor,
        Behar,
        Bechukosai,
        Bamidbar,
        Nasso,
        Behaaloscha,
        Shlach,
        Korach,
        Chukas,
        Balak,
        Pinchas,
        Matos,
        Masei,
        Devarim,
        Vaeschanan,
        Eikev,
        Reeh,
        Shoftim,
        KiSeitzei,
        KiSavo,
        Nitzavim,
        Vayeilech,
        HaAzinu,
        VezosHabracha,
        VayakhelPekudei,
        TazriaMetzora,
        AchreiMosKedoshim,
        BeharBechukosai,
        ChukasBalak,
        MatosMasei,
        NitzavimVayeilech,
        Shekalim,
        Zachor,
        Parah,
        Hachodesh,
        Shuva,
        Shira,
        Hagadol,
        Chazon,
        Nachamu,
    }

    pub enum YearLengthTypeCode {
        Chaserim,
        Kesidran,
        Shelaimim,
    }

    #[diplomat::opaque]
    pub struct HolidayList(Vec<HolidayEntry>);

    impl HolidayList {
        pub fn len(&self) -> u32 {
            u32::try_from(self.0.len()).unwrap_or(u32::MAX)
        }

        pub fn get(&self, index: u32) -> Option<HolidayEntry> {
            self.0.get(index as usize).copied()
        }
    }

    fn to_internal(date: &CivilDate) -> crate::common::CivilDate {
        crate::common::CivilDate {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }

    fn to_internal_hebrew(date: &HebrewDate) -> crate::common::HebrewDate {
        crate::common::HebrewDate {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }

    fn to_ffi_hebrew(date: crate::common::HebrewDate) -> HebrewDate {
        HebrewDate {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }

    fn to_ffi_holiday_code(code: crate::common::HolidayCode) -> HolidayCode {
        use crate::common::HolidayCode as C;
        match code {
            C::ErevPesach => HolidayCode::ErevPesach,
            C::Pesach => HolidayCode::Pesach,
            C::CholHamoedPesach => HolidayCode::CholHamoedPesach,
            C::PesachSheni => HolidayCode::PesachSheni,
            C::ErevShavuos => HolidayCode::ErevShavuos,
            C::Shavuos => HolidayCode::Shavuos,
            C::SeventeenthOfTammuz => HolidayCode::SeventeenthOfTammuz,
            C::TishahBav => HolidayCode::TishahBav,
            C::TuBav => HolidayCode::TuBav,
            C::ErevRoshHashana => HolidayCode::ErevRoshHashana,
            C::RoshHashana => HolidayCode::RoshHashana,
            C::FastOfGedalyah => HolidayCode::FastOfGedalyah,
            C::ErevYomKippur => HolidayCode::ErevYomKippur,
            C::YomKippur => HolidayCode::YomKippur,
            C::ErevSuccos => HolidayCode::ErevSuccos,
            C::Succos => HolidayCode::Succos,
            C::CholHamoedSuccos => HolidayCode::CholHamoedSuccos,
            C::HoshanaRabbah => HolidayCode::HoshanaRabbah,
            C::SheminiAtzeres => HolidayCode::SheminiAtzeres,
            C::SimchasTorah => HolidayCode::SimchasTorah,
            C::Chanukah => HolidayCode::Chanukah,
            C::TenthOfTeves => HolidayCode::TenthOfTeves,
            C::TuBshvat => HolidayCode::TuBshvat,
            C::FastOfEsther => HolidayCode::FastOfEsther,
            C::Purim => HolidayCode::Purim,
            C::ShushanPurim => HolidayCode::ShushanPurim,
            C::PurimKatan => HolidayCode::PurimKatan,
            C::RoshChodesh => HolidayCode::RoshChodesh,
            C::YomHaShoah => HolidayCode::YomHaShoah,
            C::YomHazikaron => HolidayCode::YomHazikaron,
            C::YomHaatzmaut => HolidayCode::YomHaatzmaut,
            C::YomYerushalayim => HolidayCode::YomYerushalayim,
            C::LagBomer => HolidayCode::LagBomer,
            C::ShushanPurimKatan => HolidayCode::ShushanPurimKatan,
            C::IsruChag => HolidayCode::IsruChag,
            C::YomKippurKatan => HolidayCode::YomKippurKatan,
            C::Behab => HolidayCode::Behab,
            C::FastOfTheFirstborn => HolidayCode::FastOfTheFirstborn,
            C::CountOfTheOmer => HolidayCode::CountOfTheOmer,
            C::BirchasHachamah => HolidayCode::BirchasHachamah,
            C::MacharHachodesh => HolidayCode::MacharHachodesh,
            C::ShabbosMevarchim => HolidayCode::ShabbosMevarchim,
        }
    }

    fn to_ffi_parsha(code: crate::common::ParshaCode) -> ParshaCode {
        use crate::common::ParshaCode as C;
        match code {
            C::Bereshis => ParshaCode::Bereshis,
            C::Noach => ParshaCode::Noach,
            C::LechLecha => ParshaCode::LechLecha,
            C::Vayera => ParshaCode::Vayera,
            C::ChayeiSara => ParshaCode::ChayeiSara,
            C::Toldos => ParshaCode::Toldos,
            C::Vayetzei => ParshaCode::Vayetzei,
            C::Vayishlach => ParshaCode::Vayishlach,
            C::Vayeshev => ParshaCode::Vayeshev,
            C::Miketz => ParshaCode::Miketz,
            C::Vayigash => ParshaCode::Vayigash,
            C::Vayechi => ParshaCode::Vayechi,
            C::Shemos => ParshaCode::Shemos,
            C::Vaera => ParshaCode::Vaera,
            C::Bo => ParshaCode::Bo,
            C::Beshalach => ParshaCode::Beshalach,
            C::Yisro => ParshaCode::Yisro,
            C::Mishpatim => ParshaCode::Mishpatim,
            C::Terumah => ParshaCode::Terumah,
            C::Tetzaveh => ParshaCode::Tetzaveh,
            C::KiSisa => ParshaCode::KiSisa,
            C::Vayakhel => ParshaCode::Vayakhel,
            C::Pekudei => ParshaCode::Pekudei,
            C::Vayikra => ParshaCode::Vayikra,
            C::Tzav => ParshaCode::Tzav,
            C::Shmini => ParshaCode::Shmini,
            C::Tazria => ParshaCode::Tazria,
            C::Metzora => ParshaCode::Metzora,
            C::AchreiMos => ParshaCode::AchreiMos,
            C::Kedoshim => ParshaCode::Kedoshim,
            C::Emor => ParshaCode::Emor,
            C::Behar => ParshaCode::Behar,
            C::Bechukosai => ParshaCode::Bechukosai,
            C::Bamidbar => ParshaCode::Bamidbar,
            C::Nasso => ParshaCode::Nasso,
            C::Behaaloscha => ParshaCode::Behaaloscha,
            C::Shlach => ParshaCode::Shlach,
            C::Korach => ParshaCode::Korach,
            C::Chukas => ParshaCode::Chukas,
            C::Balak => ParshaCode::Balak,
            C::Pinchas => ParshaCode::Pinchas,
            C::Matos => ParshaCode::Matos,
            C::Masei => ParshaCode::Masei,
            C::Devarim => ParshaCode::Devarim,
            C::Vaeschanan => ParshaCode::Vaeschanan,
            C::Eikev => ParshaCode::Eikev,
            C::Reeh => ParshaCode::Reeh,
            C::Shoftim => ParshaCode::Shoftim,
            C::KiSeitzei => ParshaCode::KiSeitzei,
            C::KiSavo => ParshaCode::KiSavo,
            C::Nitzavim => ParshaCode::Nitzavim,
            C::Vayeilech => ParshaCode::Vayeilech,
            C::HaAzinu => ParshaCode::HaAzinu,
            C::VezosHabracha => ParshaCode::VezosHabracha,
            C::VayakhelPekudei => ParshaCode::VayakhelPekudei,
            C::TazriaMetzora => ParshaCode::TazriaMetzora,
            C::AchreiMosKedoshim => ParshaCode::AchreiMosKedoshim,
            C::BeharBechukosai => ParshaCode::BeharBechukosai,
            C::ChukasBalak => ParshaCode::ChukasBalak,
            C::MatosMasei => ParshaCode::MatosMasei,
            C::NitzavimVayeilech => ParshaCode::NitzavimVayeilech,
            C::Shekalim => ParshaCode::Shekalim,
            C::Zachor => ParshaCode::Zachor,
            C::Parah => ParshaCode::Parah,
            C::Hachodesh => ParshaCode::Hachodesh,
            C::Shuva => ParshaCode::Shuva,
            C::Shira => ParshaCode::Shira,
            C::Hagadol => ParshaCode::Hagadol,
            C::Chazon => ParshaCode::Chazon,
            C::Nachamu => ParshaCode::Nachamu,
        }
    }

    /// Converts a Gregorian date to Hebrew.
    pub fn gregorian_to_hebrew(date: &CivilDate) -> Option<HebrewDate> {
        common_gregorian_to_hebrew(to_internal(date)).map(to_ffi_hebrew)
    }

    /// Converts a Hebrew date to Gregorian.
    pub fn hebrew_to_gregorian(date: &HebrewDate) -> Option<CivilDate> {
        common_hebrew_to_gregorian(to_internal_hebrew(date)).map(|d| CivilDate {
            year: d.year,
            month: d.month,
            day: d.day,
        })
    }

    /// Returns holidays occurring on a Gregorian date.
    pub fn holidays_on_date(date: &CivilDate, in_israel: bool, use_modern_holidays: bool) -> Option<Box<HolidayList>> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        let items = jiff
            .holidays(in_israel, use_modern_holidays)
            .map(|holiday| {
                let (code, sub_day) = holiday_to_codes(holiday);
                HolidayEntry {
                    code: to_ffi_holiday_code(code),
                    sub_day,
                }
            })
            .collect();
        Some(Box::new(HolidayList(items)))
    }

    /// Returns whether work is forbidden on this date.
    pub fn is_assur_bemelacha(date: &CivilDate, in_israel: bool) -> Option<bool> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        Some(jiff.is_assur_bemelacha(in_israel))
    }

    /// Returns whether candle lighting occurs on this date.
    pub fn has_candle_lighting(date: &CivilDate, in_israel: bool) -> Option<bool> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        Some(jiff.has_candle_lighting(in_israel))
    }

    /// Returns the weekly parsha when this date is Shabbat.
    pub fn todays_parsha(date: &CivilDate, in_israel: bool) -> Option<ParshaCode> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        jiff.todays_parsha(in_israel)
            .map(|p| to_ffi_parsha(crate::common::ParshaCode::from(p)))
    }

    /// Returns the special Shabbat designation when applicable.
    pub fn special_parsha(date: &CivilDate, in_israel: bool) -> Option<ParshaCode> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        jiff.special_parsha(in_israel)
            .map(|p| to_ffi_parsha(crate::common::ParshaCode::from(p)))
    }

    /// Returns the parsha for the next Shabbat on or after this date.
    pub fn upcoming_parsha(date: &CivilDate, in_israel: bool) -> Option<ParshaCode> {
        let jiff = civil_date_to_jiff(to_internal(date))?;
        jiff.upcoming_parsha(in_israel)
            .map(|p| to_ffi_parsha(crate::common::ParshaCode::from(p)))
    }

    /// Returns the number of days in a Hebrew year.
    pub fn days_in_hebrew_year(year: i32) -> Option<i32> {
        Hebrew::days_in_hebrew_year(year)
    }

    /// Returns the number of days in a Hebrew month.
    pub fn days_in_hebrew_month(year: i32, month: u8) -> Option<u8> {
        let month = decode_hebrew_month(month)?;
        Hebrew::days_in_hebrew_month(year, month)
    }

    /// Returns whether a Hebrew year is a leap year.
    pub fn is_hebrew_leap_year(year: i32) -> bool {
        Hebrew::is_hebrew_leap_year(year)
    }

    /// Returns the Cheshvan/Kislev year length type.
    pub fn cheshvan_kislev_kviah(year: i32) -> Option<YearLengthTypeCode> {
        Hebrew::cheshvan_kislev_kviah(year).map(|value| match crate::common::YearLengthTypeCode::from(value) {
            crate::common::YearLengthTypeCode::Chaserim => YearLengthTypeCode::Chaserim,
            crate::common::YearLengthTypeCode::Kesidran => YearLengthTypeCode::Kesidran,
            crate::common::YearLengthTypeCode::Shelaimim => YearLengthTypeCode::Shelaimim,
        })
    }
}
