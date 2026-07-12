//! Limudim FFI bridge.

#[diplomat::bridge]
mod ffi {
    use kosher_rust::limudim::{
        AmudYomiBavliDirshu, DafHashavuaBavli, DafYomiBavli, DafYomiYerushalmiVilna, LimudCalendar, MishnaYomis,
        PirkeiAvos, PirkeiAvosUnit, TehillimMonthly, TehillimUnit,
    };

    use crate::common::{TractateCode as InternalTractateCode, civil_date_to_jiff, side_to_code, tractate_to_code};

    pub enum TractateCode {
        Berachos,
        Peah,
        Demai,
        Kilayim,
        Sheviis,
        Terumos,
        Maasros,
        MaaserSheni,
        Chalah,
        Orlah,
        Bikurim,
        Shabbos,
        Eruvin,
        Pesachim,
        Shekalim,
        Yoma,
        Sukkah,
        Beitzah,
        RoshHashanah,
        Taanis,
        Megillah,
        MoedKatan,
        Chagigah,
        Yevamos,
        Kesubos,
        Nedarim,
        Nazir,
        Sotah,
        Gitin,
        Kiddushin,
        BavaKamma,
        BavaMetzia,
        BavaBasra,
        Sanhedrin,
        Makkos,
        Shevuos,
        Eduyos,
        AvodahZarah,
        Avos,
        Horiyos,
        Zevachim,
        Menachos,
        Chullin,
        Bechoros,
        Arachin,
        Temurah,
        Kerisos,
        Meilah,
        Tamid,
        Midos,
        Kinnim,
        Keilim,
        Ohalos,
        Negaim,
        Parah,
        Taharos,
        Mikvaos,
        Niddah,
        Machshirin,
        Zavim,
        TevulYom,
        Yadayim,
        Uktzin,
    }

    pub enum SideCode {
        Aleph,
        Bet,
    }

    pub struct DafResult {
        pub tractate: TractateCode,
        pub page: u16,
    }

    pub struct AmudResult {
        pub tractate: TractateCode,
        pub page: u16,
        pub side: SideCode,
    }

    pub struct MishnaResult {
        pub tractate: TractateCode,
        pub chapter: usize,
        pub mishna: u16,
    }

    pub struct MishnasResult {
        pub start: MishnaResult,
        pub end: MishnaResult,
    }

    /// Flat tagged union for Pirkei Avos (`kind`: 0 = single, 1 = combined).
    pub struct PirkeiAvosResult {
        pub kind: u8,
        pub perek1: u8,
        pub perek2: u8,
    }

    /// Flat tagged union for Tehillim (`kind`: 0 = psalms range, 1 = verse range).
    pub struct TehillimResult {
        pub kind: u8,
        pub start: u8,
        pub end: u8,
        pub psalm: u8,
        pub start_verse: u16,
        pub end_verse: u16,
    }

    fn to_ffi_tractate(code: InternalTractateCode) -> TractateCode {
        use InternalTractateCode as C;
        match code {
            C::Berachos => TractateCode::Berachos,
            C::Peah => TractateCode::Peah,
            C::Demai => TractateCode::Demai,
            C::Kilayim => TractateCode::Kilayim,
            C::Sheviis => TractateCode::Sheviis,
            C::Terumos => TractateCode::Terumos,
            C::Maasros => TractateCode::Maasros,
            C::MaaserSheni => TractateCode::MaaserSheni,
            C::Chalah => TractateCode::Chalah,
            C::Orlah => TractateCode::Orlah,
            C::Bikurim => TractateCode::Bikurim,
            C::Shabbos => TractateCode::Shabbos,
            C::Eruvin => TractateCode::Eruvin,
            C::Pesachim => TractateCode::Pesachim,
            C::Shekalim => TractateCode::Shekalim,
            C::Yoma => TractateCode::Yoma,
            C::Sukkah => TractateCode::Sukkah,
            C::Beitzah => TractateCode::Beitzah,
            C::RoshHashanah => TractateCode::RoshHashanah,
            C::Taanis => TractateCode::Taanis,
            C::Megillah => TractateCode::Megillah,
            C::MoedKatan => TractateCode::MoedKatan,
            C::Chagigah => TractateCode::Chagigah,
            C::Yevamos => TractateCode::Yevamos,
            C::Kesubos => TractateCode::Kesubos,
            C::Nedarim => TractateCode::Nedarim,
            C::Nazir => TractateCode::Nazir,
            C::Sotah => TractateCode::Sotah,
            C::Gitin => TractateCode::Gitin,
            C::Kiddushin => TractateCode::Kiddushin,
            C::BavaKamma => TractateCode::BavaKamma,
            C::BavaMetzia => TractateCode::BavaMetzia,
            C::BavaBasra => TractateCode::BavaBasra,
            C::Sanhedrin => TractateCode::Sanhedrin,
            C::Makkos => TractateCode::Makkos,
            C::Shevuos => TractateCode::Shevuos,
            C::Eduyos => TractateCode::Eduyos,
            C::AvodahZarah => TractateCode::AvodahZarah,
            C::Avos => TractateCode::Avos,
            C::Horiyos => TractateCode::Horiyos,
            C::Zevachim => TractateCode::Zevachim,
            C::Menachos => TractateCode::Menachos,
            C::Chullin => TractateCode::Chullin,
            C::Bechoros => TractateCode::Bechoros,
            C::Arachin => TractateCode::Arachin,
            C::Temurah => TractateCode::Temurah,
            C::Kerisos => TractateCode::Kerisos,
            C::Meilah => TractateCode::Meilah,
            C::Tamid => TractateCode::Tamid,
            C::Midos => TractateCode::Midos,
            C::Kinnim => TractateCode::Kinnim,
            C::Keilim => TractateCode::Keilim,
            C::Ohalos => TractateCode::Ohalos,
            C::Negaim => TractateCode::Negaim,
            C::Parah => TractateCode::Parah,
            C::Taharos => TractateCode::Taharos,
            C::Mikvaos => TractateCode::Mikvaos,
            C::Niddah => TractateCode::Niddah,
            C::Machshirin => TractateCode::Machshirin,
            C::Zavim => TractateCode::Zavim,
            C::TevulYom => TractateCode::TevulYom,
            C::Yadayim => TractateCode::Yadayim,
            C::Uktzin => TractateCode::Uktzin,
        }
    }

    fn to_ffi_side(code: crate::common::SideCode) -> SideCode {
        match code {
            crate::common::SideCode::Aleph => SideCode::Aleph,
            crate::common::SideCode::Bet => SideCode::Bet,
        }
    }

    fn to_mishna_result(m: kosher_rust::limudim::Mishna) -> MishnaResult {
        MishnaResult {
            tractate: to_ffi_tractate(tractate_to_code(m.tractate)),
            chapter: m.chapter,
            mishna: m.mishna,
        }
    }

    fn jiff_from_parts(year: i32, month: u8, day: u8) -> Option<jiff::civil::Date> {
        civil_date_to_jiff(crate::common::CivilDate { year, month, day })
    }

    fn daf_yomi_bavli_impl(year: i32, month: u8, day: u8) -> Option<DafResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let daf = jiff.limud(DafYomiBavli::default())?;
        Some(DafResult {
            tractate: to_ffi_tractate(tractate_to_code(daf.tractate)),
            page: daf.page,
        })
    }

    fn daf_yomi_yerushalmi_impl(year: i32, month: u8, day: u8) -> Option<DafResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let daf = jiff.limud(DafYomiYerushalmiVilna::default())?;
        Some(DafResult {
            tractate: to_ffi_tractate(tractate_to_code(daf.tractate)),
            page: daf.page,
        })
    }

    fn daf_hashavua_bavli_impl(year: i32, month: u8, day: u8) -> Option<DafResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let daf = jiff.limud(DafHashavuaBavli::default())?;
        Some(DafResult {
            tractate: to_ffi_tractate(tractate_to_code(daf.tractate)),
            page: daf.page,
        })
    }

    fn amud_yomi_bavli_dirshu_impl(year: i32, month: u8, day: u8) -> Option<AmudResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let amud = jiff.limud(AmudYomiBavliDirshu::default())?;
        Some(AmudResult {
            tractate: to_ffi_tractate(tractate_to_code(amud.tractate)),
            page: amud.page,
            side: to_ffi_side(side_to_code(amud.side)),
        })
    }

    fn mishna_yomis_impl(year: i32, month: u8, day: u8) -> Option<MishnasResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let mishnas = jiff.limud(MishnaYomis)?;
        Some(MishnasResult {
            start: to_mishna_result(mishnas.0),
            end: to_mishna_result(mishnas.1),
        })
    }

    fn pirkei_avos_impl(year: i32, month: u8, day: u8, in_israel: bool) -> Option<PirkeiAvosResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let unit = jiff.limud(PirkeiAvos { in_israel })?;
        Some(match unit {
            PirkeiAvosUnit::Single(perek) => PirkeiAvosResult {
                kind: 0,
                perek1: perek,
                perek2: 0,
            },
            PirkeiAvosUnit::Combined(p1, p2) => PirkeiAvosResult {
                kind: 1,
                perek1: p1,
                perek2: p2,
            },
        })
    }

    fn tehillim_monthly_impl(year: i32, month: u8, day: u8) -> Option<TehillimResult> {
        let jiff = jiff_from_parts(year, month, day)?;
        let unit = jiff.limud(TehillimMonthly)?;
        Some(match unit {
            TehillimUnit::Psalms { start, end } => TehillimResult {
                kind: 0,
                start,
                end,
                psalm: 0,
                start_verse: 0,
                end_verse: 0,
            },
            TehillimUnit::PsalmVerses {
                psalm,
                start_verse,
                end_verse,
            } => TehillimResult {
                kind: 1,
                start: 0,
                end: 0,
                psalm,
                start_verse,
                end_verse,
            },
        })
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn daf_yomi_bavli(year: i32, month: u8, day: u8) -> Option<DafResult> {
        daf_yomi_bavli_impl(year, month, day)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn daf_yomi_yerushalmi(year: i32, month: u8, day: u8) -> Option<DafResult> {
        daf_yomi_yerushalmi_impl(year, month, day)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn daf_hashavua_bavli(year: i32, month: u8, day: u8) -> Option<DafResult> {
        daf_hashavua_bavli_impl(year, month, day)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn amud_yomi_bavli_dirshu(year: i32, month: u8, day: u8) -> Option<AmudResult> {
        amud_yomi_bavli_dirshu_impl(year, month, day)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn mishna_yomis(year: i32, month: u8, day: u8) -> Option<MishnasResult> {
        mishna_yomis_impl(year, month, day)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn pirkei_avos(year: i32, month: u8, day: u8, in_israel: bool) -> Option<PirkeiAvosResult> {
        pirkei_avos_impl(year, month, day, in_israel)
    }

    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn tehillim_monthly(year: i32, month: u8, day: u8) -> Option<TehillimResult> {
        tehillim_monthly_impl(year, month, day)
    }

    /// Dart entry point: construct once and call instance methods.
    #[diplomat::cfg(not(supports = free_functions))]
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct Limudim(u8);

    impl Limudim {
        #[diplomat::attr(*, constructor)]
        pub fn new() -> Box<Limudim> {
            Box::new(Limudim(0))
        }

        pub fn daf_yomi_bavli(&self, year: i32, month: u8, day: u8) -> Option<DafResult> {
            daf_yomi_bavli_impl(year, month, day)
        }

        pub fn daf_yomi_yerushalmi(&self, year: i32, month: u8, day: u8) -> Option<DafResult> {
            daf_yomi_yerushalmi_impl(year, month, day)
        }

        pub fn daf_hashavua_bavli(&self, year: i32, month: u8, day: u8) -> Option<DafResult> {
            daf_hashavua_bavli_impl(year, month, day)
        }

        pub fn amud_yomi_bavli_dirshu(&self, year: i32, month: u8, day: u8) -> Option<AmudResult> {
            amud_yomi_bavli_dirshu_impl(year, month, day)
        }

        pub fn mishna_yomis(&self, year: i32, month: u8, day: u8) -> Option<MishnasResult> {
            mishna_yomis_impl(year, month, day)
        }

        pub fn pirkei_avos(&self, year: i32, month: u8, day: u8, in_israel: bool) -> Option<PirkeiAvosResult> {
            pirkei_avos_impl(year, month, day, in_israel)
        }

        pub fn tehillim_monthly(&self, year: i32, month: u8, day: u8) -> Option<TehillimResult> {
            tehillim_monthly_impl(year, month, day)
        }
    }
}
