#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Tractate {
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

pub const ALL_TRACTATES: [Tractate; 63] = [
    Tractate::Berachos,
    Tractate::Peah,
    Tractate::Demai,
    Tractate::Kilayim,
    Tractate::Sheviis,
    Tractate::Terumos,
    Tractate::Maasros,
    Tractate::MaaserSheni,
    Tractate::Chalah,
    Tractate::Orlah,
    Tractate::Bikurim,
    Tractate::Shabbos,
    Tractate::Eruvin,
    Tractate::Pesachim,
    Tractate::Shekalim,
    Tractate::Yoma,
    Tractate::Sukkah,
    Tractate::Beitzah,
    Tractate::RoshHashanah,
    Tractate::Taanis,
    Tractate::Megillah,
    Tractate::MoedKatan,
    Tractate::Chagigah,
    Tractate::Yevamos,
    Tractate::Kesubos,
    Tractate::Nedarim,
    Tractate::Nazir,
    Tractate::Sotah,
    Tractate::Gitin,
    Tractate::Kiddushin,
    Tractate::BavaKamma,
    Tractate::BavaMetzia,
    Tractate::BavaBasra,
    Tractate::Sanhedrin,
    Tractate::Makkos,
    Tractate::Shevuos,
    Tractate::Eduyos,
    Tractate::AvodahZarah,
    Tractate::Avos,
    Tractate::Horiyos,
    Tractate::Zevachim,
    Tractate::Menachos,
    Tractate::Chullin,
    Tractate::Bechoros,
    Tractate::Arachin,
    Tractate::Temurah,
    Tractate::Kerisos,
    Tractate::Meilah,
    Tractate::Tamid,
    Tractate::Midos,
    Tractate::Kinnim,
    Tractate::Keilim,
    Tractate::Ohalos,
    Tractate::Negaim,
    Tractate::Parah,
    Tractate::Taharos,
    Tractate::Mikvaos,
    Tractate::Niddah,
    Tractate::Machshirin,
    Tractate::Zavim,
    Tractate::TevulYom,
    Tractate::Yadayim,
    Tractate::Uktzin,
];

pub const BAVLI_TRACTATES: [Tractate; 40] = [
    Tractate::Berachos,
    Tractate::Shabbos,
    Tractate::Eruvin,
    Tractate::Pesachim,
    Tractate::Shekalim,
    Tractate::Yoma,
    Tractate::Sukkah,
    Tractate::Beitzah,
    Tractate::RoshHashanah,
    Tractate::Taanis,
    Tractate::Megillah,
    Tractate::MoedKatan,
    Tractate::Chagigah,
    Tractate::Yevamos,
    Tractate::Kesubos,
    Tractate::Nedarim,
    Tractate::Nazir,
    Tractate::Sotah,
    Tractate::Gitin,
    Tractate::Kiddushin,
    Tractate::BavaKamma,
    Tractate::BavaMetzia,
    Tractate::BavaBasra,
    Tractate::Sanhedrin,
    Tractate::Makkos,
    Tractate::Shevuos,
    Tractate::AvodahZarah,
    Tractate::Horiyos,
    Tractate::Zevachim,
    Tractate::Menachos,
    Tractate::Chullin,
    Tractate::Bechoros,
    Tractate::Arachin,
    Tractate::Temurah,
    Tractate::Kerisos,
    Tractate::Meilah,
    Tractate::Kinnim,
    Tractate::Tamid,
    Tractate::Midos,
    Tractate::Niddah,
];
pub const YERUSHALMI_TRACTATES: [Tractate; 39] = [
    Tractate::Berachos,
    Tractate::Peah,
    Tractate::Demai,
    Tractate::Kilayim,
    Tractate::Sheviis,
    Tractate::Terumos,
    Tractate::Maasros,
    Tractate::MaaserSheni,
    Tractate::Chalah,
    Tractate::Orlah,
    Tractate::Bikurim,
    Tractate::Shabbos,
    Tractate::Eruvin,
    Tractate::Pesachim,
    Tractate::Beitzah,
    Tractate::RoshHashanah,
    Tractate::Yoma,
    Tractate::Sukkah,
    Tractate::Taanis,
    Tractate::Shekalim,
    Tractate::Megillah,
    Tractate::Chagigah,
    Tractate::MoedKatan,
    Tractate::Yevamos,
    Tractate::Kesubos,
    Tractate::Sotah,
    Tractate::Nedarim,
    Tractate::Nazir,
    Tractate::Gitin,
    Tractate::Kiddushin,
    Tractate::BavaKamma,
    Tractate::BavaMetzia,
    Tractate::BavaBasra,
    Tractate::Shevuos,
    Tractate::Makkos,
    Tractate::Sanhedrin,
    Tractate::AvodahZarah,
    Tractate::Horiyos,
    Tractate::Niddah,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Daf {
    pub tractate: Tractate,
    pub page: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Side {
    Aleph,
    Bet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Amud {
    pub tractate: Tractate,
    pub page: u16,
    pub side: Side,
}
impl Amud {
    pub const fn new(tractate: Tractate, page: u16, side: Side) -> Self {
        Self { tractate, page, side }
    }
    fn next(&self) -> Self {
        if self.side == Side::Aleph {
            Self {
                tractate: self.tractate,
                page: self.page,
                side: Side::Bet,
            }
        } else {
            Self {
                tractate: self.tractate,
                page: self.page + 1,
                side: Side::Aleph,
            }
        }
    }
}

pub struct AmudIter {
    end: Amud,
    current: Option<Amud>,
}
impl Iterator for AmudIter {
    type Item = Amud;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        if current == self.end {
            self.current = None;
        } else {
            self.current = Some(current.next());
        }
        Some(current)
    }
}
impl AmudIter {
    pub const fn new(start: Amud, end: Amud) -> Self {
        Self {
            end,
            current: Some(start),
        }
    }
    pub const fn empty() -> Self {
        Self {
            end: Amud::new(Tractate::Bechoros, 0, Side::Aleph),
            current: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mishna {
    pub tractate: Tractate,
    pub chapter: usize,
    pub mishna: u16,
}
