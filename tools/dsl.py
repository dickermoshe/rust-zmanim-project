from __future__ import annotations

from typing import Annotated, Literal, Union

from pydantic import BaseModel, Field

# KosherJava method names supported by this DSL. The same value is stored in
# each `Zman.id`, used as the generated Rust parity-test method name, and used
# as the key for generated documentation.
ZmanimMethod = Literal[
    "getSunTransit",
    "getSolarMidnight",
    "getBeginCivilTwilight",
    "getEndCivilTwilight",
    "getBeginNauticalTwilight",
    "getEndNauticalTwilight",
    "getBeginAstronomicalTwilight",
    "getEndAstronomicalTwilight",
    "getPolarSunsetBenIshChai",
    "getPolarSunriseBenIshChai",
    "getSunrise",
    "getSeaLevelSunrise",
    "getSunset",
    "getSeaLevelSunset",
    "getAlos60Minutes",
    "getAlos72Minutes",
    "getAlos72Zmanis",
    "getAlos90Minutes",
    "getAlos90Zmanis",
    "getAlos96Minutes",
    "getAlos96Zmanis",
    "getAlos120Minutes",
    "getAlos120Zmanis",
    "getAlos16Point1Degrees",
    "getAlos18Degrees",
    "getAlos19Degrees",
    "getAlos19Point8Degrees",
    "getAlos26Degrees",
    "getAlosBaalHatanya",
    "getBainHashmashosRT13Point24Degrees",
    "getBainHashmashosRT58Point5Minutes",
    "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees",
    "getBainHashmashosRT2Stars",
    "getBainHashmashosYereim18Minutes",
    "getBainHashmashosYereim16Point875Minutes",
    "getBainHashmashosYereim13Point5Minutes",
    "getBainHashmashosYereim2Point1Degrees",
    "getBainHashmashosYereim2Point8Degrees",
    "getBainHashmashosYereim3Point05Degrees",
    "getCandleLighting",
    "getChatzosHayom",
    "getChatzosHalayla",
    "getChatzosHayomAsHalfDay",
    "getFixedLocalChatzosHayom",
    "getMinchaGedolaGRA",
    "getMinchaGedola16Point1Degrees",
    "getMinchaGedola30Minutes",
    "getMinchaGedola72Minutes",
    "getMinchaGedolaAhavatShalom",
    "getMinchaGedolaGRAGreaterThan30",
    "getMinchaGedolaAteretTorah",
    "getMinchaGedolaBaalHatanya",
    "getMinchaGedolaGRAFixedLocalChatzos30Minutes",
    "getMinchaKetanaGRA",
    "getMinchaKetana16Point1Degrees",
    "getMinchaKetana72Minutes",
    "getMinchaKetanaAhavatShalom",
    "getMinchaKetanaAteretTorah",
    "getMinchaKetanaBaalHatanya",
    "getMinchaKetanaGRAFixedLocalChatzosToSunset",
    "getMisheyakir10Point2Degrees",
    "getMisheyakir11Degrees",
    "getMisheyakir11Point5Degrees",
    "getMisheyakir12Point85Degrees",
    "getMisheyakir7Point65Degrees",
    "getMisheyakir9Point5Degrees",
    "getPlagHaminchaGRA",
    "getPlagAhavatShalom",
    "getPlagAlos16Point1DegreesToTzaisGeonim7Point083Degrees",
    "getPlagAlosToSunset",
    "getPlagHamincha60Minutes",
    "getPlagHamincha72Minutes",
    "getPlagHamincha72MinutesZmanis",
    "getPlagHamincha90Minutes",
    "getPlagHamincha90MinutesZmanis",
    "getPlagHamincha96Minutes",
    "getPlagHamincha96MinutesZmanis",
    "getPlagHamincha120Minutes",
    "getPlagHamincha120MinutesZmanis",
    "getPlagHamincha16Point1Degrees",
    "getPlagHamincha18Degrees",
    "getPlagHamincha19Point8Degrees",
    "getPlagHamincha26Degrees",
    "getPlagHaminchaAteretTorah",
    "getPlagHaminchaBaalHatanya",
    "getPlagHaminchaGRAFixedLocalChatzosToSunset",
    "getSamuchLeMinchaKetanaGRA",
    "getSamuchLeMinchaKetana16Point1Degrees",
    "getSamuchLeMinchaKetana72Minutes",
    "getSofZmanAchilasChametzGRA",
    "getSofZmanAchilasChametzMGA72Minutes",
    "getSofZmanAchilasChametzMGA72MinutesZmanis",
    "getSofZmanAchilasChametzMGA16Point1Degrees",
    "getSofZmanAchilasChametzBaalHatanya",
    "getSofZmanBiurChametzGRA",
    "getSofZmanBiurChametzMGA72Minutes",
    "getSofZmanBiurChametzMGA72MinutesZmanis",
    "getSofZmanBiurChametzMGA16Point1Degrees",
    "getSofZmanBiurChametzBaalHatanya",
    "getSofZmanShmaGRA",
    "getSofZmanShmaMGA19Point8Degrees",
    "getSofZmanShmaMGA16Point1Degrees",
    "getSofZmanShmaMGA18Degrees",
    "getSofZmanShmaMGA72Minutes",
    "getSofZmanShmaMGA72MinutesZmanis",
    "getSofZmanShmaMGA90Minutes",
    "getSofZmanShmaMGA90MinutesZmanis",
    "getSofZmanShmaMGA96Minutes",
    "getSofZmanShmaMGA96MinutesZmanis",
    "getSofZmanShma3HoursBeforeChatzos",
    "getSofZmanShmaMGA120Minutes",
    "getSofZmanShmaAlos16Point1ToSunset",
    "getSofZmanShmaAlos16Point1DegreesToTzaisGeonim7Point083Degrees",
    "getSofZmanShmaAteretTorah",
    "getSofZmanShmaBaalHatanya",
    "getSofZmanShmaGRASunriseToFixedLocalChatzos",
    "getSofZmanShmaMGA18DegreesToFixedLocalChatzos",
    "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos",
    "getSofZmanShmaMGA90MinutesToFixedLocalChatzos",
    "getSofZmanShmaMGA72MinutesToFixedLocalChatzos",
    "getSofZmanTfilaGRA",
    "getSofZmanTfilaMGA72Minutes",
    "getSofZmanTfilaMGA19Point8Degrees",
    "getSofZmanTfilaMGA16Point1Degrees",
    "getSofZmanTfilaMGA18Degrees",
    "getSofZmanTfilaMGA72MinutesZmanis",
    "getSofZmanTfilaMGA90Minutes",
    "getSofZmanTfilaMGA90MinutesZmanis",
    "getSofZmanTfilaMGA96Minutes",
    "getSofZmanTfilaMGA96MinutesZmanis",
    "getSofZmanTfila2HoursBeforeChatzos",
    "getSofZmanTfilaMGA120Minutes",
    "getSofZmanTfilaAteretTorah",
    "getSofZmanTfilaBaalHatanya",
    "getSofZmanTfilaGRASunriseToFixedLocalChatzos",
    "getTzaisGeonim8Point5Degrees",
    "getTzais50Minutes",
    "getTzais60Minutes",
    "getTzais72Minutes",
    "getTzais72Zmanis",
    "getTzais90Minutes",
    "getTzais90Zmanis",
    "getTzais96Minutes",
    "getTzais96Zmanis",
    "getTzais120Minutes",
    "getTzais120Zmanis",
    "getTzais16Point1Degrees",
    "getTzais18Degrees",
    "getTzais19Point8Degrees",
    "getTzais26Degrees",
    "getTzaisAteretTorah",
    "getTzaisBaalHatanya",
    "getTzaisGeonim3Point7Degrees",
    "getTzaisGeonim3Point8Degrees",
    "getTzaisGeonim5Point95Degrees",
    "getTzaisGeonim4Point66Degrees",
    "getTzaisGeonim4Point42Degrees",
    "getTzaisGeonim4Point8Degrees",
    "getTzaisGeonim6Point45Degrees",
    "getTzaisGeonim7Point083Degrees",
    "getTzaisGeonim7Point67Degrees",
    "getTzaisGeonim9Point3Degrees",
    "getTzaisGeonim9Point75Degrees",
    "getSofZmanKidushLevana15Days",
    "getSofZmanKidushLevanaBetweenMoldos",
    "getTchilasZmanKidushLevana3Days",
    "getTchilasZmanKidushLevana7Days",
]

ZmanType = Literal[
    "twilight",  # Start and End of Astronomical, Civil, and Nautical Twilight
    "alos",
    "misheyakir",
    "netz",
    "sof_zman_shma",
    "sof_zman_tefila",
    "sof_zman_achilas_chametz",
    "sof_zman_biur_chametz",
    "chatzos_hayom",
    "mincha_gedola",
    "plag_hamincha",
    "samuch_le_mincha_ketana",
    "mincha_ketana",
    "bein_hashmashos",
    "candle_lighting",
    "shkiya",
    "tzais",
    "kidush_levana",
    "chatzos_halayla",
]


class ElevationAdjustedSunrise(BaseModel):
    """Equivalent to `getSunrise`."""

    type_: Literal["elevation_adjusted_sunrise"] = "elevation_adjusted_sunrise"


class SeaLevelSunrise(BaseModel):
    """Equivalent to `getSeaLevelSunrise`."""

    type_: Literal["sea_level_sunrise"] = "sea_level_sunrise"


class ConfiguredSunrise(BaseModel):
    """Equivalent to `getSunriseBasedOnElevationSetting`."""

    type_: Literal["configured_sunrise"] = "configured_sunrise"


class ConfiguredSunset(BaseModel):
    """Equivalent to `getSunsetBasedOnElevationSetting`."""

    type_: Literal["configured_sunset"] = "configured_sunset"


class SolarTransit(BaseModel):
    """Equivalent to `getSunTransit`."""

    type_: Literal["solar_transit"] = "solar_transit"


class SolarMidnight(BaseModel):
    """Equivalent to `getSolarMidnight`."""

    type_: Literal["solar_midnight"] = "solar_midnight"


class ElevationAdjustedSunset(BaseModel):
    """Equivalent to `getSunset`."""

    type_: Literal["elevation_adjusted_sunset"] = "elevation_adjusted_sunset"


class SeaLevelSunset(BaseModel):
    """Equivalent to `getSeaLevelSunset`."""

    type_: Literal["sea_level_sunset"] = "sea_level_sunset"


class SunriseOffsetByDegrees(BaseModel):
    """
    Equivalent to `getSunriseOffsetByDegrees` with the slight difference that
    java takes the complete zenith (e.g. getSunriseOffsetByDegrees(96) for 6 degrees before sunrise)
    while we take the offset (e.g. getSunriseOffsetByDegrees(6) for 6 degrees before sunrise).
    """

    type_: Literal["sunrise_offset_by_degrees"] = "sunrise_offset_by_degrees"
    degrees: float


class SunsetOffsetByDegrees(BaseModel):
    """
    Equivalent to `getSunsetOffsetByDegrees` with the slight difference that
    java takes the complete zenith (e.g. getSunsetOffsetByDegrees(96) for 6 degrees after sunset)
    while we take the offset (e.g. getSunsetOffsetByDegrees(6) for 6 degrees after sunset).
    """

    type_: Literal["sunset_offset_by_degrees"] = "sunset_offset_by_degrees"
    degrees: float


class LocalMeanTime(BaseModel):
    """Equivalent to `getLocalMeanTime`."""

    type_: Literal["local_mean_time"] = "local_mean_time"
    hour: float


class CandleLighting(BaseModel):
    """Equivalent to `getCandleLighting`."""

    type_: Literal["candle_lighting"] = "candle_lighting"


class Offset(BaseModel):
    """Equivalent to `getTimeOffset`."""

    type_: Literal["offset"] = "offset"
    base: ZmanPrimitive
    duration_secs: float


class ZmanisOffset(BaseModel):
    """Equivalent to `getZmanisBasedOffset`."""

    type_: Literal["zmanis_offset"] = "zmanis_offset"
    base: ZmanPrimitive
    hours: float


class HalfDayBasedOffset(BaseModel):
    """Equivalent to `getHalfDayBasedZman`."""

    type_: Literal["half_day_based_offset"] = "half_day_based_offset"
    start: ZmanPrimitive
    end: ZmanPrimitive
    fraction: float


class Shema(BaseModel):
    """Equivalent to `getSofZmanShma`."""

    type_: Literal["shema"] = "shema"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class MinchaGedola(BaseModel):
    """Equivalent to `getMinchaGedola`."""

    type_: Literal["mincha_gedola"] = "mincha_gedola"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class SamuchLeMinchaKetana(BaseModel):
    """Equivalent to `getSamuchLeMinchaKetana`."""

    type_: Literal["samuch_le_mincha_ketana"] = "samuch_le_mincha_ketana"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class MinchaKetana(BaseModel):
    """Equivalent to `getMinchaKetana`."""

    type_: Literal["mincha_ketana"] = "mincha_ketana"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class Tefila(BaseModel):
    """Equivalent to `getSofZmanTfila`."""

    type_: Literal["tefila"] = "tefila"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class PlagHamincha(BaseModel):
    """Equivalent to `getPlagHamincha`."""

    type_: Literal["plag_hamincha"] = "plag_hamincha"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class SofZmanBiurChametz(BaseModel):
    """Equivalent to `getSofZmanBiurChametz`."""

    type_: Literal["sof_zman_biur_chametz"] = "sof_zman_biur_chametz"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class SofZmanAchilasChametz(BaseModel):
    """Equivalent to `getSofZmanAchilasChametz`."""

    type_: Literal["sof_zman_achilas_chametz"] = "sof_zman_achilas_chametz"
    start: ZmanPrimitive
    end: ZmanPrimitive
    synchronous: bool


class TzaisAteretTorah(BaseModel):
    """Equivalent to `getTzaisAteretTorah`."""

    type_: Literal["tzais_ateret_torah"] = "tzais_ateret_torah"


class SofZmanKidushLevana15Days(BaseModel):
    """Equivalent to `getSofZmanKidushLevana15Days`."""

    type_: Literal["sof_zman_kidush_levana_15_days"] = "sof_zman_kidush_levana_15_days"


class SofZmanKidushLevanaBetweenMoldos(BaseModel):
    """Equivalent to `getSofZmanKidushLevanaBetweenMoldos`."""

    type_: Literal["sof_zman_kidush_levana_between_moldos"] = (
        "sof_zman_kidush_levana_between_moldos"
    )


class TchilasZmanKidushLevana3Days(BaseModel):
    """Equivalent to `getTchilasZmanKidushLevana3Days`."""

    type_: Literal["tchilas_zman_kidush_levana_3_days"] = (
        "tchilas_zman_kidush_levana_3_days"
    )


class TchilasZmanKidushLevana7Days(BaseModel):
    """Equivalent to `getTchilasZmanKidushLevana7Days`."""

    type_: Literal["tchilas_zman_kidush_levana_7_days"] = (
        "tchilas_zman_kidush_levana_7_days"
    )


class BainHashmashosRt2Stars(BaseModel):
    """Equivalent to `getBainHashmashosRt2Stars`."""

    type_: Literal["bain_hashmashos_rt2_stars"] = "bain_hashmashos_rt2_stars"


class MinchaGedolaAhavatShalom(BaseModel):
    """Equivalent to `getMinchaGedolaAhavatShalom`."""

    type_: Literal["mincha_gedola_ahavat_shalom"] = "mincha_gedola_ahavat_shalom"


class MinchaGedolaGraGreaterThan30(BaseModel):
    """Equivalent to `getMinchaGedolaGraGreaterThan30`."""

    type_: Literal["mincha_gedola_gra_greater_than_30"] = (
        "mincha_gedola_gra_greater_than_30"
    )


class MinchaKetanaAhavatShalom(BaseModel):
    """Equivalent to `getMinchaKetanaAhavatShalom`."""

    type_: Literal["mincha_ketana_ahavat_shalom"] = "mincha_ketana_ahavat_shalom"


class PlagAhavatShalom(BaseModel):
    """Equivalent to `getPlagAhavatShalom`."""

    type_: Literal["plag_ahavat_shalom"] = "plag_ahavat_shalom"


class BeginCivilTwilight(BaseModel):
    """Equivalent to `getBeginCivilTwilight`."""

    type_: Literal["begin_civil_twilight"] = "begin_civil_twilight"


class EndCivilTwilight(BaseModel):
    """Equivalent to `getEndCivilTwilight`."""

    type_: Literal["end_civil_twilight"] = "end_civil_twilight"


class BeginNauticalTwilight(BaseModel):
    """Equivalent to `getBeginNauticalTwilight`."""

    type_: Literal["begin_nautical_twilight"] = "begin_nautical_twilight"


class EndNauticalTwilight(BaseModel):
    """Equivalent to `getEndNauticalTwilight`."""

    type_: Literal["end_nautical_twilight"] = "end_nautical_twilight"


class BeginAstronomicalTwilight(BaseModel):
    """Equivalent to `getBeginAstronomicalTwilight`."""

    type_: Literal["begin_astronomical_twilight"] = "begin_astronomical_twilight"


class EndAstronomicalTwilight(BaseModel):
    """Equivalent to `getEndAstronomicalTwilight`."""

    type_: Literal["end_astronomical_twilight"] = "end_astronomical_twilight"


class PolarSunsetBenIshChai(BaseModel):
    """Equivalent to `getPolarSunsetBenIshChai`."""

    type_: Literal["polar_sunset_ben_ish_chai"] = "polar_sunset_ben_ish_chai"


class PolarSunriseBenIshChai(BaseModel):
    """Equivalent to `getPolarSunriseBenIshChai`."""

    type_: Literal["polar_sunrise_ben_ish_chai"] = "polar_sunrise_ben_ish_chai"


class ChatzosHayomAsHalfDay(BaseModel):
    """Equivalent to `getChatzosHayomAsHalfDay`."""

    type_: Literal["chatzos_hayom_as_half_day"] = "chatzos_hayom_as_half_day"


class ChatzosHayom(BaseModel):
    """Equivalent to `getChatzosHayom`."""

    type_: Literal["chatzos_hayom"] = "chatzos_hayom"


class ChatzosHalayla(BaseModel):
    """Equivalent to `getChatzosHalayla`."""

    type_: Literal["chatzos_halayla"] = "chatzos_halayla"


# A primitive is the reusable calculation shape behind a zman preset. Most
# presets are either a direct primitive, a fixed/zmanis offset from one, or a
# proportional-time calculation between two primitives.
ZmanPrimitive = Annotated[
    Union[
        ElevationAdjustedSunrise,
        SeaLevelSunrise,
        ConfiguredSunrise,
        ConfiguredSunset,
        SolarTransit,
        SolarMidnight,
        ElevationAdjustedSunset,
        SeaLevelSunset,
        SunriseOffsetByDegrees,
        SunsetOffsetByDegrees,
        LocalMeanTime,
        CandleLighting,
        Offset,
        ZmanisOffset,
        HalfDayBasedOffset,
        Shema,
        MinchaGedola,
        SamuchLeMinchaKetana,
        MinchaKetana,
        Tefila,
        PlagHamincha,
        SofZmanBiurChametz,
        SofZmanAchilasChametz,
        TzaisAteretTorah,
        SofZmanKidushLevana15Days,
        SofZmanKidushLevanaBetweenMoldos,
        TchilasZmanKidushLevana3Days,
        TchilasZmanKidushLevana7Days,
        BainHashmashosRt2Stars,
        MinchaGedolaAhavatShalom,
        MinchaGedolaGraGreaterThan30,
        MinchaKetanaAhavatShalom,
        PlagAhavatShalom,
        BeginCivilTwilight,
        EndCivilTwilight,
        BeginNauticalTwilight,
        EndNauticalTwilight,
        BeginAstronomicalTwilight,
        EndAstronomicalTwilight,
        PolarSunsetBenIshChai,
        PolarSunriseBenIshChai,
        ChatzosHayomAsHalfDay,
        ChatzosHayom,
        ChatzosHalayla,
    ],
    Field(discriminator="type_"),
]


class Zman(BaseModel):
    """A supported zman preset and the DSL primitive used to calculate it."""

    id: ZmanimMethod
    """The KosherJava getter this preset matches, such as `getAlos120Zmanis`."""
    type_: ZmanType
    """The broad zman category used for grouping."""
    name: str
    """The human-readable display name."""
    zman: ZmanPrimitive | None = None
    """The calculation primitive. `None` means the preset is not implemented."""
    deprecated: bool = False
    """Whether this preset should only be generated behind test/deprecated gates."""
    developer_notes: str | None = None
    """Any additional notes for the developer."""

    def description(self) -> str:
        """
        Return the generated user-facing documentation for this zman.

        A couple of notes on how to render this to users:

        - `{ateret_torah_offset}` and `{candel_lighting_offset}` should be
            replaced with the configured offsets.
        - `{chatzos_hayom_calculation}` and `{chatzos_halayla_calculation}`
            should be replaced based on `use_astronomical_chatzos`. See
            `CHATZOS_HAYOM_CALCULATION` and `CHATZOS_HALAYLA_CALCULATION`.
        - There are 3 configurations that affect how zmanim are calculated:
            - `use_elevation`
            - `use_astronomical_chatzos`
            - `use_astronomical_chatzos_for_other_zmanim`
            At runtime, check if any of your zmanim are affected by these setting and append
            a message to them.
            See the rust preset code for an example of this.
        """
        desc: str | None = DOCS.get(self.id)
        if desc is None:
            raise RuntimeError(f"No description found for {self.id}")
        return desc


# Ordered registry of supported zman presets. Keep this as a list so each
# `Zman` is self-contained: the Java method name lives in `id` and the order
# remains stable for tools that preserve source order.
ZMAN: list[Zman] = [
    Zman(
        id="getSunTransit",
        type_="chatzos_hayom",
        name="Solar Transit",
        zman=SolarTransit(),
    ),
    Zman(
        id="getSolarMidnight",
        type_="chatzos_halayla",
        name="Solar Midnight",
        zman=SolarMidnight(),
    ),
    Zman(
        id="getBeginCivilTwilight",
        type_="twilight",
        name="Begin Civil Twilight",
        zman=BeginCivilTwilight(),
    ),
    Zman(
        id="getEndCivilTwilight",
        type_="twilight",
        name="End Civil Twilight",
        zman=EndCivilTwilight(),
    ),
    Zman(
        id="getBeginNauticalTwilight",
        type_="twilight",
        name="Begin Nautical Twilight",
        zman=BeginNauticalTwilight(),
    ),
    Zman(
        id="getEndNauticalTwilight",
        type_="twilight",
        name="End Nautical Twilight",
        zman=EndNauticalTwilight(),
    ),
    Zman(
        id="getBeginAstronomicalTwilight",
        type_="twilight",
        name="Begin Astronomical Twilight",
        zman=BeginAstronomicalTwilight(),
    ),
    Zman(
        id="getEndAstronomicalTwilight",
        type_="twilight",
        name="End Astronomical Twilight",
        zman=EndAstronomicalTwilight(),
    ),
    Zman(
        id="getPolarSunsetBenIshChai",
        type_="shkiya",
        name="Polar Sunset Ben Ish Chai",
        zman=PolarSunsetBenIshChai(),
    ),
    Zman(
        id="getPolarSunriseBenIshChai",
        type_="netz",
        name="Polar Sunrise Ben Ish Chai",
        zman=PolarSunriseBenIshChai(),
    ),
    Zman(
        id="getSunrise", type_="netz", name="Sunrise", zman=ElevationAdjustedSunrise()
    ),
    Zman(
        id="getSeaLevelSunrise",
        type_="netz",
        name="Sea Level Sunrise",
        zman=SeaLevelSunrise(),
    ),
    Zman(id="getSunset", type_="shkiya", name="Sunset", zman=ElevationAdjustedSunset()),
    Zman(
        id="getSeaLevelSunset",
        type_="shkiya",
        name="Sea Level Sunset",
        zman=SeaLevelSunset(),
    ),
    Zman(
        id="getAlos60Minutes",
        type_="alos",
        name="Alos (60 Minutes)",
        zman=Offset(
            base=ConfiguredSunrise(),
            duration_secs=-3600.0,
        ),
    ),
    Zman(
        id="getAlos72Minutes",
        type_="alos",
        name="Alos (72 Minutes)",
        zman=Offset(
            base=ConfiguredSunrise(),
            duration_secs=-4320.0,
        ),
    ),
    Zman(
        id="getAlos72Zmanis",
        type_="alos",
        name="Alos (72 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunrise(),
            hours=-1.2,
        ),
    ),
    Zman(
        id="getAlos90Minutes",
        type_="alos",
        name="Alos (90 Minutes)",
        zman=Offset(
            base=ConfiguredSunrise(),
            duration_secs=-5400.0,
        ),
    ),
    Zman(
        id="getAlos90Zmanis",
        type_="alos",
        name="Alos (90 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunrise(),
            hours=-1.5,
        ),
    ),
    Zman(
        id="getAlos96Minutes",
        type_="alos",
        name="Alos (96 Minutes)",
        zman=Offset(
            base=ConfiguredSunrise(),
            duration_secs=-5760.0,
        ),
    ),
    Zman(
        id="getAlos96Zmanis",
        type_="alos",
        name="Alos (96 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunrise(),
            hours=-1.6,
        ),
    ),
    Zman(
        id="getAlos120Minutes",
        type_="alos",
        name="Alos (120 Minutes)",
        zman=Offset(
            base=ConfiguredSunrise(),
            duration_secs=-7200.0,
        ),
        deprecated=True,
    ),
    Zman(
        id="getAlos120Zmanis",
        type_="alos",
        name="Alos (120 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunrise(),
            hours=-2.0,
        ),
        deprecated=True,
    ),
    Zman(
        id="getAlos16Point1Degrees",
        type_="alos",
        name="Alos (16.1 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=16.1),
    ),
    Zman(
        id="getAlos18Degrees",
        type_="alos",
        name="Alos (18 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=18.0),
    ),
    Zman(
        id="getAlos19Degrees",
        type_="alos",
        name="Alos (19 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=19.0),
    ),
    Zman(
        id="getAlos19Point8Degrees",
        type_="alos",
        name="Alos (19.8 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=19.8),
    ),
    Zman(
        id="getAlos26Degrees",
        type_="alos",
        name="Alos (26 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=26.0),
        deprecated=True,
    ),
    Zman(
        id="getAlosBaalHatanya",
        type_="alos",
        name="Alos (Baal Hatanya)",
        zman=SunriseOffsetByDegrees(degrees=16.9),
    ),
    Zman(
        id="getBainHashmashosRT13Point24Degrees",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Rabbeinu Tam, 13.24 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=13.24),
    ),
    Zman(
        id="getBainHashmashosRT58Point5Minutes",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Rabbeinu Tam, 58.5 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=3510.0,
        ),
    ),
    Zman(
        id="getBainHashmashosRT13Point5MinutesBefore7Point083Degrees",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Rabbeinu Tam, 13.5 Minutes Before 7.083 Degrees)",
        zman=Offset(
            base=SunsetOffsetByDegrees(degrees=7.083333333333333),
            duration_secs=-810.0,
        ),
    ),
    Zman(
        id="getBainHashmashosRT2Stars",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Rabbeinu Tam, 2 Stars)",
        zman=BainHashmashosRt2Stars(),
    ),
    Zman(
        id="getBainHashmashosYereim18Minutes",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 18 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=-1080.0,
        ),
    ),
    Zman(
        id="getBainHashmashosYereim16Point875Minutes",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 16.875 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=-1012.5,
        ),
    ),
    Zman(
        id="getBainHashmashosYereim13Point5Minutes",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 13.5 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=-810.0,
        ),
    ),
    Zman(
        id="getBainHashmashosYereim2Point1Degrees",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 2.1 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=-2.1),
    ),
    Zman(
        id="getBainHashmashosYereim2Point8Degrees",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 2.8 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=-2.8),
    ),
    Zman(
        id="getBainHashmashosYereim3Point05Degrees",
        type_="bein_hashmashos",
        name="Bain Hashmashos (Yereim, 3.05 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=-3.05),
    ),
    Zman(
        id="getCandleLighting",
        type_="candle_lighting",
        name="Candle Lighting",
        zman=CandleLighting(),
    ),
    Zman(
        id="getChatzosHayom",
        type_="chatzos_hayom",
        name="Chatzos Hayom",
        zman=ChatzosHayom(),
    ),
    Zman(
        id="getChatzosHalayla",
        type_="chatzos_halayla",
        name="Chatzos Halayla",
        zman=ChatzosHalayla(),
    ),
    Zman(
        id="getChatzosHayomAsHalfDay",
        type_="chatzos_hayom",
        name="Chatzos Hayom (Midpoint of Sunrise and Sunset)",
        zman=ChatzosHayomAsHalfDay(),
    ),
    Zman(
        id="getFixedLocalChatzosHayom",
        type_="chatzos_hayom",
        name="Chatzos Hayom (Fixed Local Chatzos)",
        zman=LocalMeanTime(hour=12.0),
    ),
    Zman(
        id="getMinchaGedolaGRA",
        type_="mincha_gedola",
        name="Mincha Gedola (GR'A)",
        zman=MinchaGedola(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaGedola16Point1Degrees",
        type_="mincha_gedola",
        name="Mincha Gedola (16.1 Degrees)",
        zman=MinchaGedola(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaGedola30Minutes",
        type_="mincha_gedola",
        name="Mincha Gedola (30 Minutes)",
        zman=Offset(
            base=ChatzosHayom(),
            duration_secs=1800.0,
        ),
    ),
    Zman(
        id="getMinchaGedola72Minutes",
        type_="mincha_gedola",
        name="Mincha Gedola (72 Minutes)",
        zman=MinchaGedola(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaGedolaAhavatShalom",
        type_="mincha_gedola",
        name="Mincha Gedola (Ahavat Shalom)",
        zman=MinchaGedolaAhavatShalom(),
    ),
    Zman(
        id="getMinchaGedolaGRAGreaterThan30",
        type_="mincha_gedola",
        name="Mincha Gedola (GR'A, Greater Than 30)",
        zman=MinchaGedolaGraGreaterThan30(),
    ),
    Zman(
        id="getMinchaGedolaAteretTorah",
        type_="mincha_gedola",
        name="Mincha Gedola (Ateret Torah)",
        zman=MinchaGedola(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=TzaisAteretTorah(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getMinchaGedolaBaalHatanya",
        type_="mincha_gedola",
        name="Mincha Gedola (Baal Hatanya)",
        zman=MinchaGedola(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaGedolaGRAFixedLocalChatzos30Minutes",
        type_="mincha_gedola",
        name="Mincha Gedola (GR'A, Fixed Local Chatzos, 30 Minutes)",
        zman=Offset(
            base=LocalMeanTime(hour=12.0),
            duration_secs=1800.0,
        ),
    ),
    Zman(
        id="getMinchaKetanaGRA",
        type_="mincha_ketana",
        name="Mincha Ketana (GR'A)",
        zman=MinchaKetana(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaKetana16Point1Degrees",
        type_="mincha_ketana",
        name="Mincha Ketana (16.1 Degrees)",
        zman=MinchaKetana(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaKetana72Minutes",
        type_="mincha_ketana",
        name="Mincha Ketana (72 Minutes)",
        zman=MinchaKetana(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaKetanaAhavatShalom",
        type_="mincha_ketana",
        name="Mincha Ketana (Ahavat Shalom)",
        zman=MinchaKetanaAhavatShalom(),
    ),
    Zman(
        id="getMinchaKetanaAteretTorah",
        type_="mincha_ketana",
        name="Mincha Ketana (Ateret Torah)",
        zman=MinchaKetana(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=TzaisAteretTorah(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getMinchaKetanaBaalHatanya",
        type_="mincha_ketana",
        name="Mincha Ketana (Baal Hatanya)",
        zman=MinchaKetana(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
    ),
    Zman(
        id="getMinchaKetanaGRAFixedLocalChatzosToSunset",
        type_="mincha_ketana",
        name="Mincha Ketana (GR'A, Fixed Local Chatzos to Sunset)",
        zman=HalfDayBasedOffset(
            start=LocalMeanTime(hour=12.0),
            end=ConfiguredSunset(),
            fraction=3.5,
        ),
    ),
    Zman(
        id="getMisheyakir10Point2Degrees",
        type_="misheyakir",
        name="Misheyakir (10.2 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=10.2),
    ),
    Zman(
        id="getMisheyakir11Degrees",
        type_="misheyakir",
        name="Misheyakir (11 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=11.0),
    ),
    Zman(
        id="getMisheyakir11Point5Degrees",
        type_="misheyakir",
        name="Misheyakir (11.5 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=11.5),
    ),
    Zman(
        id="getMisheyakir12Point85Degrees",
        type_="misheyakir",
        name="Misheyakir (12.85 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=12.85),
        deprecated=True,
    ),
    Zman(
        id="getMisheyakir7Point65Degrees",
        type_="misheyakir",
        name="Misheyakir (7.65 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=7.65),
    ),
    Zman(
        id="getMisheyakir9Point5Degrees",
        type_="misheyakir",
        name="Misheyakir (9.5 Degrees)",
        zman=SunriseOffsetByDegrees(degrees=9.5),
    ),
    Zman(
        id="getPlagHaminchaGRA",
        type_="plag_hamincha",
        name="Plag Hamincha (GR'A)",
        zman=PlagHamincha(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getPlagAhavatShalom",
        type_="plag_hamincha",
        name="Plag Hamincha (Ahavat Shalom)",
        zman=PlagAhavatShalom(),
    ),
    Zman(
        id="getPlagAlos16Point1DegreesToTzaisGeonim7Point083Degrees",
        type_="plag_hamincha",
        name="Plag Hamincha (Alos 16.1 to Tzais Geonim 7.083 Degrees)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=7.083333333333333),
            synchronous=False,
        ),
    ),
    Zman(
        id="getPlagAlosToSunset",
        type_="plag_hamincha",
        name="Plag Hamincha (Alos 16.1 to Sunset)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=ConfiguredSunset(),
            synchronous=False,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha60Minutes",
        type_="plag_hamincha",
        name="Plag Hamincha (60 Minutes)",
        zman=PlagHamincha(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-3600.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=3600.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getPlagHamincha72Minutes",
        type_="plag_hamincha",
        name="Plag Hamincha (72 Minutes)",
        zman=PlagHamincha(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha72MinutesZmanis",
        type_="plag_hamincha",
        name="Plag Hamincha (72 Minutes in Shaos Zmanios)",
        zman=PlagHamincha(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.2,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha90Minutes",
        type_="plag_hamincha",
        name="Plag Hamincha (90 Minutes)",
        zman=PlagHamincha(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5400.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5400.0,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha90MinutesZmanis",
        type_="plag_hamincha",
        name="Plag Hamincha (90 Minutes in Shaos Zmanios)",
        zman=PlagHamincha(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.5,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.5,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha96Minutes",
        type_="plag_hamincha",
        name="Plag Hamincha (96 Minutes)",
        zman=PlagHamincha(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5760.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5760.0,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha96MinutesZmanis",
        type_="plag_hamincha",
        name="Plag Hamincha (96 Minutes in Shaos Zmanios)",
        zman=PlagHamincha(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.6,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.6,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha120Minutes",
        type_="plag_hamincha",
        name="Plag Hamincha (120 Minutes)",
        zman=PlagHamincha(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-7200.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=7200.0,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha120MinutesZmanis",
        type_="plag_hamincha",
        name="Plag Hamincha (120 Minutes in Shaos Zmanios)",
        zman=PlagHamincha(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-2.0,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=2.0,
            ),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha16Point1Degrees",
        type_="plag_hamincha",
        name="Plag Hamincha (16.1 Degrees)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha18Degrees",
        type_="plag_hamincha",
        name="Plag Hamincha (18 Degrees)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=18.0),
            end=SunsetOffsetByDegrees(degrees=18.0),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha19Point8Degrees",
        type_="plag_hamincha",
        name="Plag Hamincha (19.8 Degrees)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=19.8),
            end=SunsetOffsetByDegrees(degrees=19.8),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHamincha26Degrees",
        type_="plag_hamincha",
        name="Plag Hamincha (26 Degrees)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=26.0),
            end=SunsetOffsetByDegrees(degrees=26.0),
            synchronous=True,
        ),
        deprecated=True,
    ),
    Zman(
        id="getPlagHaminchaAteretTorah",
        type_="plag_hamincha",
        name="Plag Hamincha (Ateret Torah)",
        zman=PlagHamincha(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=TzaisAteretTorah(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getPlagHaminchaBaalHatanya",
        type_="plag_hamincha",
        name="Plag Hamincha (Baal Hatanya)",
        zman=PlagHamincha(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
    ),
    Zman(
        id="getPlagHaminchaGRAFixedLocalChatzosToSunset",
        type_="plag_hamincha",
        name="Plag Hamincha (GR'A, Fixed Local Chatzos to Sunset)",
        zman=HalfDayBasedOffset(
            start=LocalMeanTime(hour=12.0),
            end=ConfiguredSunset(),
            fraction=4.75,
        ),
    ),
    Zman(
        id="getSamuchLeMinchaKetanaGRA",
        type_="samuch_le_mincha_ketana",
        name="Samuch Le Mincha Ketana (GR'A)",
        zman=SamuchLeMinchaKetana(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSamuchLeMinchaKetana16Point1Degrees",
        type_="samuch_le_mincha_ketana",
        name="Samuch Le Mincha Ketana (16.1 Degrees)",
        zman=SamuchLeMinchaKetana(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSamuchLeMinchaKetana72Minutes",
        type_="samuch_le_mincha_ketana",
        name="Samuch Le Mincha Ketana (72 Minutes)",
        zman=SamuchLeMinchaKetana(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanAchilasChametzGRA",
        type_="sof_zman_achilas_chametz",
        name="Sof Zman Achilas Chametz (GR'A)",
        zman=SofZmanAchilasChametz(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanAchilasChametzMGA72Minutes",
        type_="sof_zman_achilas_chametz",
        name="Sof Zman Achilas Chametz (MGA, 72 Minutes)",
        zman=SofZmanAchilasChametz(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanAchilasChametzMGA72MinutesZmanis",
        type_="sof_zman_achilas_chametz",
        name="Sof Zman Achilas Chametz (MGA, 72 Minutes in Shaos Zmanios)",
        zman=SofZmanAchilasChametz(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.2,
            ),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanAchilasChametzMGA16Point1Degrees",
        type_="sof_zman_achilas_chametz",
        name="Sof Zman Achilas Chametz (MGA, 16.1 Degrees)",
        zman=SofZmanAchilasChametz(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanAchilasChametzBaalHatanya",
        type_="sof_zman_achilas_chametz",
        name="Sof Zman Achilas Chametz (Baal Hatanya)",
        zman=SofZmanAchilasChametz(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanBiurChametzGRA",
        type_="sof_zman_biur_chametz",
        name="Sof Zman Biur Chametz (GR'A)",
        zman=SofZmanBiurChametz(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanBiurChametzMGA72Minutes",
        type_="sof_zman_biur_chametz",
        name="Sof Zman Biur Chametz (MGA, 72 Minutes)",
        zman=SofZmanBiurChametz(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanBiurChametzMGA72MinutesZmanis",
        type_="sof_zman_biur_chametz",
        name="Sof Zman Biur Chametz (MGA, 72 Minutes in Shaos Zmanios)",
        zman=SofZmanBiurChametz(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.2,
            ),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanBiurChametzMGA16Point1Degrees",
        type_="sof_zman_biur_chametz",
        name="Sof Zman Biur Chametz (MGA, 16.1 Degrees)",
        zman=SofZmanBiurChametz(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanBiurChametzBaalHatanya",
        type_="sof_zman_biur_chametz",
        name="Sof Zman Biur Chametz (Baal Hatanya)",
        zman=SofZmanBiurChametz(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
        developer_notes="A value will always be returned if the date is Erev Pesach.",
    ),
    Zman(
        id="getSofZmanShmaGRA",
        type_="sof_zman_shma",
        name="Sof Zman Shma (GR'A)",
        zman=Shema(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA19Point8Degrees",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 19.8 Degrees)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=19.8),
            end=SunsetOffsetByDegrees(degrees=19.8),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA16Point1Degrees",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 16.1 Degrees)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA18Degrees",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 18 Degrees)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=18.0),
            end=SunsetOffsetByDegrees(degrees=18.0),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA72Minutes",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 72 Minutes)",
        zman=Shema(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA72MinutesZmanis",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 72 Minutes in Shaos Zmanios)",
        zman=Shema(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.2,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA90Minutes",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 90 Minutes)",
        zman=Shema(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5400.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5400.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA90MinutesZmanis",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 90 Minutes in Shaos Zmanios)",
        zman=Shema(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.5,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.5,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA96Minutes",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 96 Minutes)",
        zman=Shema(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5760.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5760.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA96MinutesZmanis",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 96 Minutes in Shaos Zmanios)",
        zman=Shema(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.6,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.6,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShma3HoursBeforeChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (3 Hours Before Chatzos)",
        zman=Offset(
            base=ChatzosHayom(),
            duration_secs=-10800.0,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA120Minutes",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 120 Minutes)",
        zman=Shema(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-7200.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=7200.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaAlos16Point1ToSunset",
        type_="sof_zman_shma",
        name="Sof Zman Shma (Alos 16.1 to Sunset)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=ConfiguredSunset(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getSofZmanShmaAlos16Point1DegreesToTzaisGeonim7Point083Degrees",
        type_="sof_zman_shma",
        name="Sof Zman Shma (Alos 16.1 to Tzais Geonim 7.083 Degrees)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=7.083333333333333),
            synchronous=False,
        ),
    ),
    Zman(
        id="getSofZmanShmaAteretTorah",
        type_="sof_zman_shma",
        name="Sof Zman Shma (Ateret Torah)",
        zman=Shema(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=TzaisAteretTorah(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getSofZmanShmaBaalHatanya",
        type_="sof_zman_shma",
        name="Sof Zman Shma (Baal Hatanya)",
        zman=Shema(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanShmaGRASunriseToFixedLocalChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (GR'A, Sunrise to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=ConfiguredSunrise(),
            end=LocalMeanTime(hour=12.0),
            fraction=3.0,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA18DegreesToFixedLocalChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 18 Degrees to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=SunriseOffsetByDegrees(degrees=18.0),
            end=LocalMeanTime(hour=12.0),
            fraction=3.0,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 16.1 Degrees to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=LocalMeanTime(hour=12.0),
            fraction=3.0,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA90MinutesToFixedLocalChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 90 Minutes to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5400.0,
            ),
            end=LocalMeanTime(hour=12.0),
            fraction=3.0,
        ),
    ),
    Zman(
        id="getSofZmanShmaMGA72MinutesToFixedLocalChatzos",
        type_="sof_zman_shma",
        name="Sof Zman Shma (MGA, 72 Minutes to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=LocalMeanTime(hour=12.0),
            fraction=3.0,
        ),
    ),
    Zman(
        id="getSofZmanTfilaGRA",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (GR'A)",
        zman=Tefila(
            start=ConfiguredSunrise(),
            end=ConfiguredSunset(),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA72Minutes",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 72 Minutes)",
        zman=Tefila(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-4320.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=4320.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA19Point8Degrees",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 19.8 Degrees)",
        zman=Tefila(
            start=SunriseOffsetByDegrees(degrees=19.8),
            end=SunsetOffsetByDegrees(degrees=19.8),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA16Point1Degrees",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 16.1 Degrees)",
        zman=Tefila(
            start=SunriseOffsetByDegrees(degrees=16.1),
            end=SunsetOffsetByDegrees(degrees=16.1),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA18Degrees",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 18 Degrees)",
        zman=Tefila(
            start=SunriseOffsetByDegrees(degrees=18.0),
            end=SunsetOffsetByDegrees(degrees=18.0),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA72MinutesZmanis",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 72 Minutes in Shaos Zmanios)",
        zman=Tefila(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.2,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA90Minutes",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 90 Minutes)",
        zman=Tefila(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5400.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5400.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA90MinutesZmanis",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 90 Minutes in Shaos Zmanios)",
        zman=Tefila(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.5,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.5,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA96Minutes",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 96 Minutes)",
        zman=Tefila(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-5760.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=5760.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA96MinutesZmanis",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 96 Minutes in Shaos Zmanios)",
        zman=Tefila(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.6,
            ),
            end=ZmanisOffset(
                base=ConfiguredSunset(),
                hours=1.6,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfila2HoursBeforeChatzos",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (2 Hours Before Chatzos)",
        zman=Offset(
            base=ChatzosHayom(),
            duration_secs=-7200.0,
        ),
    ),
    Zman(
        id="getSofZmanTfilaMGA120Minutes",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (MGA, 120 Minutes)",
        zman=Tefila(
            start=Offset(
                base=ConfiguredSunrise(),
                duration_secs=-7200.0,
            ),
            end=Offset(
                base=ConfiguredSunset(),
                duration_secs=7200.0,
            ),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaAteretTorah",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (Ateret Torah)",
        zman=Tefila(
            start=ZmanisOffset(
                base=ConfiguredSunrise(),
                hours=-1.2,
            ),
            end=TzaisAteretTorah(),
            synchronous=False,
        ),
    ),
    Zman(
        id="getSofZmanTfilaBaalHatanya",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (Baal Hatanya)",
        zman=Tefila(
            start=SunriseOffsetByDegrees(degrees=1.583),
            end=SunsetOffsetByDegrees(degrees=1.583),
            synchronous=True,
        ),
    ),
    Zman(
        id="getSofZmanTfilaGRASunriseToFixedLocalChatzos",
        type_="sof_zman_tefila",
        name="Sof Zman Tfila (GR'A, Sunrise to Fixed Local Chatzos)",
        zman=HalfDayBasedOffset(
            start=ConfiguredSunrise(),
            end=LocalMeanTime(hour=12.0),
            fraction=4.0,
        ),
    ),
    Zman(
        id="getTzaisGeonim8Point5Degrees",
        type_="tzais",
        name="Tzais (Geonim, 8.5 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=8.5),
    ),
    Zman(
        id="getTzais50Minutes",
        type_="tzais",
        name="Tzais (50 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=3000.0,
        ),
    ),
    Zman(
        id="getTzais60Minutes",
        type_="tzais",
        name="Tzais (60 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=3600.0,
        ),
    ),
    Zman(
        id="getTzais72Minutes",
        type_="tzais",
        name="Tzais (72 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=4320.0,
        ),
    ),
    Zman(
        id="getTzais72Zmanis",
        type_="tzais",
        name="Tzais (72 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunset(),
            hours=1.2,
        ),
    ),
    Zman(
        id="getTzais90Minutes",
        type_="tzais",
        name="Tzais (90 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=5400.0,
        ),
    ),
    Zman(
        id="getTzais90Zmanis",
        type_="tzais",
        name="Tzais (90 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunset(),
            hours=1.5,
        ),
    ),
    Zman(
        id="getTzais96Minutes",
        type_="tzais",
        name="Tzais (96 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=5760.0,
        ),
    ),
    Zman(
        id="getTzais96Zmanis",
        type_="tzais",
        name="Tzais (96 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunset(),
            hours=1.6,
        ),
    ),
    Zman(
        id="getTzais120Minutes",
        type_="tzais",
        name="Tzais (120 Minutes)",
        zman=Offset(
            base=ConfiguredSunset(),
            duration_secs=7200.0,
        ),
        deprecated=True,
    ),
    Zman(
        id="getTzais120Zmanis",
        type_="tzais",
        name="Tzais (120 Minutes in Shaos Zmanios)",
        zman=ZmanisOffset(
            base=ConfiguredSunset(),
            hours=2.0,
        ),
        deprecated=True,
    ),
    Zman(
        id="getTzais16Point1Degrees",
        type_="tzais",
        name="Tzais (16.1 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=16.1),
    ),
    Zman(
        id="getTzais18Degrees",
        type_="tzais",
        name="Tzais (18 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=18.0),
    ),
    Zman(
        id="getTzais19Point8Degrees",
        type_="tzais",
        name="Tzais (19.8 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=19.8),
    ),
    Zman(
        id="getTzais26Degrees",
        type_="tzais",
        name="Tzais (26 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=26.0),
        deprecated=True,
    ),
    Zman(
        id="getTzaisAteretTorah",
        type_="tzais",
        name="Tzais (Ateret Torah)",
        zman=TzaisAteretTorah(),
    ),
    Zman(
        id="getTzaisBaalHatanya",
        type_="tzais",
        name="Tzais (Baal Hatanya)",
        zman=SunsetOffsetByDegrees(degrees=6.0),
    ),
    Zman(
        id="getTzaisGeonim3Point7Degrees",
        type_="tzais",
        name="Tzais (Geonim, 3.7 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=3.7),
        deprecated=True,
    ),
    Zman(
        id="getTzaisGeonim3Point8Degrees",
        type_="tzais",
        name="Tzais (Geonim, 3.8 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=3.8),
        deprecated=True,
    ),
    Zman(
        id="getTzaisGeonim5Point95Degrees",
        type_="tzais",
        name="Tzais (Geonim, 5.95 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=5.95),
    ),
    Zman(
        id="getTzaisGeonim4Point66Degrees",
        type_="tzais",
        name="Tzais (Geonim, 4.66 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=4.66),
        deprecated=True,
    ),
    Zman(
        id="getTzaisGeonim4Point42Degrees",
        type_="tzais",
        name="Tzais (Geonim, 4.42 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=4.42),
        deprecated=True,
    ),
    Zman(
        id="getTzaisGeonim4Point8Degrees",
        type_="tzais",
        name="Tzais (Geonim, 4.8 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=4.8),
    ),
    Zman(
        id="getTzaisGeonim6Point45Degrees",
        type_="tzais",
        name="Tzais (Geonim, 6.45 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=6.45),
    ),
    Zman(
        id="getTzaisGeonim7Point083Degrees",
        type_="tzais",
        name="Tzais (Geonim, 7.083 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=7.083333333333333),
    ),
    Zman(
        id="getTzaisGeonim7Point67Degrees",
        type_="tzais",
        name="Tzais (Geonim, 7.67 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=7.67),
    ),
    Zman(
        id="getTzaisGeonim9Point3Degrees",
        type_="tzais",
        name="Tzais (Geonim, 9.3 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=9.3),
    ),
    Zman(
        id="getTzaisGeonim9Point75Degrees",
        type_="tzais",
        name="Tzais (Geonim, 9.75 Degrees)",
        zman=SunsetOffsetByDegrees(degrees=9.75),
    ),
    Zman(
        id="getSofZmanKidushLevana15Days",
        type_="kidush_levana",
        name="Sof Zman Kidush Levana (15 Days)",
        zman=SofZmanKidushLevana15Days(),
    ),
    Zman(
        id="getSofZmanKidushLevanaBetweenMoldos",
        type_="kidush_levana",
        name="Sof Zman Kidush Levana (Between Moldos)",
        zman=SofZmanKidushLevanaBetweenMoldos(),
    ),
    Zman(
        id="getTchilasZmanKidushLevana3Days",
        type_="kidush_levana",
        name="Tchilas Zman Kidush Levana (3 Days)",
        zman=TchilasZmanKidushLevana3Days(),
    ),
    Zman(
        id="getTchilasZmanKidushLevana7Days",
        type_="kidush_levana",
        name="Tchilas Zman Kidush Levana (7 Days)",
        zman=TchilasZmanKidushLevana7Days(),
    ),
]


for _model in (
    ElevationAdjustedSunrise,
    SeaLevelSunrise,
    ConfiguredSunrise,
    ConfiguredSunset,
    SolarTransit,
    SolarMidnight,
    ElevationAdjustedSunset,
    SeaLevelSunset,
    SunriseOffsetByDegrees,
    SunsetOffsetByDegrees,
    LocalMeanTime,
    CandleLighting,
    Offset,
    ZmanisOffset,
    HalfDayBasedOffset,
    Shema,
    MinchaGedola,
    SamuchLeMinchaKetana,
    MinchaKetana,
    Tefila,
    PlagHamincha,
    SofZmanBiurChametz,
    SofZmanAchilasChametz,
    TzaisAteretTorah,
    SofZmanKidushLevana15Days,
    SofZmanKidushLevanaBetweenMoldos,
    TchilasZmanKidushLevana3Days,
    TchilasZmanKidushLevana7Days,
    BainHashmashosRt2Stars,
    MinchaGedolaAhavatShalom,
    MinchaGedolaGraGreaterThan30,
    MinchaKetanaAhavatShalom,
    PlagAhavatShalom,
    BeginCivilTwilight,
    EndCivilTwilight,
    BeginNauticalTwilight,
    EndNauticalTwilight,
    BeginAstronomicalTwilight,
    EndAstronomicalTwilight,
    PolarSunsetBenIshChai,
    PolarSunriseBenIshChai,
    ChatzosHayomAsHalfDay,
    ChatzosHayom,
    ChatzosHalayla,
):
    _model.model_rebuild()

Zman.model_rebuild()


DOCS = {
    "getAlos120Minutes": "Alos (dawn), using an extremely early 120-minute calculation.\n\n120 minutes before sunrise. \n\nBased on the time to walk 5 mil at 24 minutes per mil.\n\nThis zman should be used lechumra only, such as stopping to eat on a fast day, and not as the start of daytime mitzvos.\n\nIn places where sunrise cannot be calculated, this zman may not be available.",
    "getAlos120Zmanis": "Alos (dawn), using an extremely early 120-zmaniyos-minute calculation.\n\n120 zmaniyos minutes before sunrise, or 1/6 of the day. \n\nBased on 5 mil at 24 minutes per mil, measured in shaos zmaniyos.\n\nThis zman should be used lechumra only, such as stopping to eat on a fast day, and not as the start of daytime mitzvos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getAlos16Point1Degrees": "Alos (dawn), the beginning of morning twilight before sunrise.\n\nThe time when the sun is 16.1 degrees below the eastern horizon before sunrise.\n\nThis reflects the 72-minute alos calculation, based on 4 mil at 18 minutes per mil.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getAlos18Degrees": "Alos (dawn), the beginning of morning twilight before sunrise.\n\nThe time when the sun is 18 degrees below the eastern horizon before sunrise.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getAlos19Degrees": "Alos (dawn), associated with the Rambam's alos.\n\nThe time when the sun is 19 degrees below the eastern horizon before sunrise.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getAlos19Point8Degrees": "Alos (dawn), the beginning of morning twilight before sunrise.\n\nThe time when the sun is 19.8 degrees below the eastern horizon before sunrise.\n\nThis is the degree-based equivalent of alos 90 minutes before sunrise around the equinox in Jerusalem.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getAlos26Degrees": "Alos (dawn), using an extremely early degree-based calculation.\n\nThe time when the sun is 26 degrees below the eastern horizon before sunrise.\n\nThis is the degree-based equivalent of alos 120 minutes before sunrise around the equinox in Jerusalem.\n\nThis zman should be used lechumra only, such as stopping to eat on a fast day, and not as the start of daytime mitzvos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getAlos60Minutes": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n60 minutes before sunrise. \n\nBased on the time to walk 4 mil at 15 minutes per mil.\n\nIn places where sunrise cannot be calculated, this zman may not be available.",
    "getAlos72Minutes": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n72 minutes before sunrise. \n\nBased on the time to walk 4 mil at 18 minutes per mil.\n\nIn places where sunrise cannot be calculated, this zman may not be available.",
    "getAlos72Zmanis": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n72 zmaniyos minutes before sunrise, or 1/10 of the day. \n\nBased on 4 mil at 18 minutes per mil, measured in shaos zmaniyos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getAlos90Minutes": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n90 minutes before sunrise. \n\nBased on the time to walk 4 mil at 22.5 minutes per mil.\n\nIn places where sunrise cannot be calculated, this zman may not be available.",
    "getAlos90Zmanis": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n90 zmaniyos minutes before sunrise, or 1/8 of the day. \n\nBased on 4 mil at 22.5 minutes per mil, measured in shaos zmaniyos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getAlos96Minutes": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n96 minutes before sunrise. \n\nBased on the time to walk 4 mil at 24 minutes per mil.\n\nIn places where sunrise cannot be calculated, this zman may not be available.",
    "getAlos96Zmanis": "Alos (dawn), the beginning of morning twilight before sunrise.\n\n96 zmaniyos minutes before sunrise, or 1/7.5 of the day. \n\nBased on 4 mil at 24 minutes per mil, measured in shaos zmaniyos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getAlosBaalHatanya": "Alos (dawn) according to the Baal Hatanya.\n\nThe time when the sun is 16.9 degrees below the eastern horizon before sunrise.\n\nBased on the view that the interval from dawn to netz amiti is 72 minutes, or 4 mil at 18 minutes per mil.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getBainHashmashosRT13Point24Degrees": "The beginning of Rabbeinu Tam's bain hashmashos.\n\nWhen the sun is 13.24 degrees below the western geometric horizon after sunset. This is the degree-based equivalent of bain hashmashos 58.5 minutes after sunset: in Jerusalem [around the equinox or equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), the sun is 13.24 degrees below geometric zenith about 58.5 minutes after sunset. The source cited in the original documentation says that the proper dip should be slightly less than 13 degrees; this project's recalculation gives about 13.2456 degrees, so 13.24 degrees is used as a truncation that yields a time about 1.5 seconds earlier, lechumra.\n\nAt some northern and southern locations, including places even south of the Arctic Circle and north of the Antarctic Circle, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees": "The beginning of Rabbeinu Tam's bain hashmashos.\n\n13.5 minutes (3/4 of an 18-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement)) before shkiah calculated at 7.083 degrees below the horizon.\n\nAt some northern and southern locations, including places even south of the Arctic Circle and north of the Antarctic Circle, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getBainHashmashosRT2Stars": "The beginning of Rabbeinu Tam's bain hashmashos, according to the Divrei Yosef (see Yisrael Vehazmanim).\n\nCalculated as 5/18 (about 27.77%) of the time from alos at 19.8 degrees before sunrise to sunrise; that interval is added after sunset to reach Rabbeinu Tam's bain hashmashos.\n\nAt some northern and southern locations, including places even south of the Arctic Circle and north of the Antarctic Circle, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getBainHashmashosRT58Point5Minutes": "The beginning of Rabbeinu Tam's bain hashmashos.\n\n58.5 minutes after sunset. Bain hashmashos is 3/4 of a [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement) before tzais, or 3 1/4 mil after sunset. With an 18-minute mil, 3.25 * 18 = 58.5 minutes.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim13Point5Minutes": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\n13.5 minutes, or 3/4 of an 18-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement), before sunset. According to the Yereim, bain hashmashos starts 3/4 of a mil before sunset and tzais (nightfall) is at sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim16Point875Minutes": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\n16.875 minutes, or 3/4 of a 22.5-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement), before sunset. According to the Yereim, bain hashmashos starts 3/4 of a mil before sunset and tzais (nightfall) is at sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim18Minutes": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\n18 minutes, or 3/4 of a 24-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement), before sunset. According to the Yereim, bain hashmashos starts 3/4 of a mil before sunset and tzais (nightfall) is at sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim2Point1Degrees": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\nWhen the sun is 2.1 degrees above the horizon - in Jerusalem [around the equinox or equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), about 13.5 minutes or 3/4 of an 18-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement) before sunset. According to the Yereim, bain hashmashos starts 3/4 of a mil before sunset and tzais (nightfall) is at sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim2Point8Degrees": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\nWhen the sun is 2.8 degrees above the horizon - in Jerusalem [around the equinox or equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), about 16.875 minutes or 3/4 of a 22.5-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement) before sunset. According to the Yereim, bain hashmashos starts 3/4 of a mil before sunset and tzais (nightfall) is at sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBainHashmashosYereim3Point05Degrees": "The beginning of bain hashmashos (twilight) according to the [Yereim (Rabbi Eliezer of Metz)](https://en.wikipedia.org/wiki/Eliezer_ben_Samuel).\n\nWhen the sun is 3.05 degrees above the horizon. In Jerusalem [around the equinox or equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), that matches about 18 minutes, or 3/4 of a 24-minute [mil](https://en.wikipedia.org/wiki/Biblical_and_Talmudic_units_of_measurement), before sunset.\n\nThe Yereim holds that bain hashmashos begins 3/4 of a mil before sunset and that tzais (nightfall) is at sunset. This degree-based version uses 0.5166 degrees of refraction instead of the traditional 0.566 degrees, which shifts the time earlier by about 14 seconds lechumra and is closer to refraction in Eretz Yisrael per [Rabbi Yedidya Manet](http://beinenu.com/rabbis/%D7%94%D7%A8%D7%91-%D7%99%D7%93%D7%99%D7%93%D7%99%D7%94-%D7%9E%D7%A0%D7%AA) ([Zmanei HaHalacha Lema'aseh](https://www.nli.org.il/en/books/NNL_ALEPH002542826/NLI), p. 11) and the [Luach Itim Lebinah](https://zmanim.online/). For background, see the [The Yereim's Bain Hashmashos](https://kosherjava.com/2020/12/07/the-yereims-bein-hashmashos/) article on the KosherJava blog.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getBeginAstronomicalTwilight": "The beginning of [astronomical twilight](https://en.wikipedia.org/wiki/Twilight#Astronomical_twilight) at dawn.\n\nCalculated using a zenith of 108 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getBeginCivilTwilight": "The beginning of [civil twilight](https://en.wikipedia.org/wiki/Twilight#Civil_twilight) at dawn.\n\nCalculated using a zenith of 96 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getBeginNauticalTwilight": "The beginning of [nautical twilight](https://en.wikipedia.org/wiki/Twilight#Nautical_twilight) at dawn.\n\nCalculated using a zenith of 102 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getCandleLighting": "The time to light candles before Shabbos or Yom Tov.\n\n{candel_lighting_offset} before sea level sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getChatzosHalayla": "Chatzos halayla.\n\nFor how chatzos can be defined and calculated, see [The Definition of Chatzos](https://kosherjava.com/2020/07/02/definition-of-chatzos/) on the KosherJava blog.\n\n",
    "getChatzosHayom": "Chatzos hayom.\n\nFor how chatzos can be defined and calculated, see [The Definition of Chatzos](https://kosherjava.com/2020/07/02/definition-of-chatzos/) on the KosherJava blog.\n\n",
    "getChatzosHayomAsHalfDay": "Chatzos hayom calculated as the halfway point between sunrise and sunset.\n\nThis is the same as six shaos zmaniyos after sunrise when the day is measured from sunrise to sunset. Many hold that chatzos is the midpoint between sea level sunrise and sea level sunset, even though astronomical chatzos is usually a slightly different time.\n\nA day measured from alos to tzais with the same offset on both sides can also have the same midpoint.\n\nFor how chatzos can be defined and calculated, see [The Definition of Chatzos](https://kosherjava.com/2020/07/02/definition-of-chatzos/) on the KosherJava blog.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getEndAstronomicalTwilight": "The end of astronomical twilight in the evening.\n\nCalculated using a zenith of 108 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getEndCivilTwilight": "The end of [civil twilight](https://en.wikipedia.org/wiki/Twilight#Civil_twilight) in the evening.\n\nCalculated using a zenith of 96 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getEndNauticalTwilight": "The end of nautical twilight in the evening.\n\nCalculated using a zenith of 102 degrees.\n\nThis zman may not be available or cannot be calculated when the computation cannot be performed.",
    "getFixedLocalChatzosHayom": "Fixed local chatzos - clock noon adjusted for the location's longitude and time zone, not tied only to theoretical 15-degree time zones.\n\nThe globe is divided into 24 hours over 360 degrees, or 15 degrees per hour (4 minutes per degree). At longitudes 0, 15, 30, and so on, chatzos is exactly 12:00 noon. The result is adjusted to the actual time zone and [daylight saving time](https://en.wikipedia.org/wiki/Daylight_saving_time).\n\nThis is the time of chatzos according to the [Aruch Hashulchan](https://en.wikipedia.org/wiki/Aruch_HaShulchan) ([Orach Chaim 233:14](https://hebrewbooks.org/pdfpager.aspx?req=7705&pgnum=426)) and [Rabbi Moshe Feinstein](https://en.wikipedia.org/wiki/Moshe_Feinstein) ([Igros Moshe, Orach Chaim 1:24](https://hebrewbooks.org/pdfpager.aspx?req=916&st=&pgnum=67), [2:20](https://hebrewbooks.org/pdfpager.aspx?req=14675&pgnum=191)).",
    "getMinchaGedola16Point1Degrees": "Mincha gedola according to the Magen Avraham, using the 16.1-degree day.\n\nHalf a shaah zmanis after chatzos hayom, using a day that begins and ends at 16.1 degrees.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMinchaGedola30Minutes": "Mincha gedola calculated as 30 minutes after astronomical chatzos hayom.\n\nSome use this in winter when half a shaah zmanis is less than 30 minutes, to delay the start of mincha.\n\nDo not use this time to begin mincha before standard mincha gedola GRA.\n\nIn places where chatzos cannot be calculated, this zman may not be available.",
    "getMinchaGedola72Minutes": "Mincha gedola according to the Magen Avraham.\n\nHalf a shaah zmanis after chatzos hayom, using a day from alos 72 minutes to tzais 72 minutes.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaGedolaAhavatShalom": "Mincha gedola according to Rabbi Yaakov Moshe Hillel, as published in the luach of the Bais Horaah of Yeshivat Chevrat Ahavat Shalom.\n\nHalf a shaah zmanis after chatzos, using a day from alos 16.1 degrees to tzais 3.7 degrees.\n\nThe later of this time or 30 clock minutes after chatzos is used.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMinchaGedolaAteretTorah": "Mincha gedola according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\n6.5 shaos zmaniyos after alos.\n\nThe day begins at alos 1/10 of the day before sunrise and ends {ateret_torah_offset} after sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaGedolaBaalHatanya": "Mincha gedola according to the Baal Hatanya.\n\n6.5 shaos zmaniyos after netz amiti, using a day from Baal Hatanya sunrise to sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaGedolaGRA": "Mincha gedola according to the [Vilna Gaon](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n6.5 shaos zmaniyos after sunrise. The day is measured from sunrise to sunset for the proportional hour.\n\nThis is the earliest time one can pray mincha.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaGedolaGRAFixedLocalChatzos30Minutes": "Mincha gedola according to Rav Moshe Feinstein's opinion.\n\n30 minutes after fixed local chatzos.\n\nIn places where fixed local chatzos cannot be calculated, this zman may not be available.",
    "getMinchaGedolaGRAGreaterThan30": "Mincha gedola calculated as the later of mincha gedola GRA and 30 minutes after astronomical chatzos hayom.\n\nIn the winter, when half a shaah zmanis is less than 30 minutes, the 30-minutes-after-chatzos time is used. Otherwise, mincha gedola GRA is used.\n\nIn places where sunrise, sunset, or chatzos cannot be calculated, this zman may not be available.",
    "getMinchaKetana16Point1Degrees": "Mincha ketana according to the Magen Avraham, using the 16.1-degree day.\n\n9.5 shaos zmaniyos after alos 16.1 degrees, using a day that begins and ends at 16.1 degrees.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMinchaKetana72Minutes": "Mincha ketana according to the Magen Avraham, using the 72-minute day.\n\n9.5 shaos zmaniyos after alos 72 minutes before sunrise, using a day that starts 72 minutes before sunrise and ends 72 minutes after sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaKetanaAhavatShalom": "Mincha ketana according to Rabbi Yaakov Moshe Hillel, as published in the luach of the Bais Horaah of Yeshivat Chevrat Ahavat Shalom.\n\n2.5 shaos zmaniyos before tzais 3.8 degrees, using a day from alos 16.1 degrees to tzais 3.8 degrees.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMinchaKetanaAteretTorah": "Mincha ketana according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\n9.5 shaos zmaniyos after alos.\n\nThe day begins at alos 1/10 of the day before sunrise and ends {ateret_torah_offset} after sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaKetanaBaalHatanya": "Mincha ketana according to the Baal Hatanya.\n\n9.5 shaos zmaniyos after netz amiti, using a day from Baal Hatanya sunrise to sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaKetanaGRA": "Mincha ketana according to the [Vilna Gaon](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n9.5 shaos zmaniyos after sunrise, using a day from sunrise to sunset. \n\nThis is the preferred earliest time to pray mincha according to the Rambam and others.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getMinchaKetanaGRAFixedLocalChatzosToSunset": "Mincha ketana according to Rav Moshe Feinstein's opinion, following the view of the Vilna Gaon.\n\n3.5 shaos zmaniyos after fixed local chatzos.\n\nIn places where fixed local chatzos or sunset cannot be calculated, this zman may not be available.",
    "getMisheyakir10Point2Degrees": "Misheyakir according to some opinions.\n\nThe time when the sun is 10.2 degrees below the horizon before sunrise.\n\nThis is about 45 minutes before sunrise in Jerusalem around the equinox.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMisheyakir11Degrees": "Misheyakir according to some opinions.\n\nThe time when the sun is 11 degrees below the horizon before sunrise.\n\nThis is about 48 minutes before sunrise in Jerusalem around the equinox.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMisheyakir11Point5Degrees": "Misheyakir according to some opinions.\n\nThe time when the sun is 11.5 degrees below the horizon before sunrise.\n\nThis is about 52 minutes before sunrise in Jerusalem around the equinox.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMisheyakir12Point85Degrees": "Misheyakir using a very early calculation.\n\nThe time when the sun is 12.85 degrees below the horizon before sunrise.\n\nThis is slightly later than 57 minutes before sunrise in Jerusalem around the equinox.\n\nThis zman should be used only bish'as hadchak. A later zman should be used lechatchila.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMisheyakir7Point65Degrees": "Misheyakir according to the 35-36 minute approach.\n\nThe time when the sun is 7.65 degrees below the horizon before sunrise.\n\nThis is based on a 35-36 minute misheyakir around the equinox, when twilight is shortest.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getMisheyakir9Point5Degrees": "Misheyakir according to the 45-minute approach used by some communities.\n\nThe time when the sun is 9.5 degrees below the horizon before sunrise.\n\nThis is based on a 45-minute misheyakir calculation.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagAhavatShalom": "Plag hamincha according to [Rabbi Yaakov Moshe Hillel](https://en.wikipedia.org/wiki/Yaakov_Moshe_Hillel), as published in the luach of the Bais Horaah of Yeshivat Chevrat Ahavat Shalom.\n\n1.25 shaos zmaniyos before tzais 3.8 degrees, using a day from alos 16.1 degrees to tzais 3.8 degrees.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagAlos16Point1DegreesToTzaisGeonim7Point083Degrees": "Plag hamincha based on a day from alos 16.1 degrees to tzais Geonim 7.083 degrees.\n\n10.75 shaos zmaniyos after alos 16.1 degrees.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagAlosToSunset": "Plag hamincha based on a day from alos 16.1 degrees to sea-level sunset.\n\n10.75 shaos zmaniyos after alos 16.1 degrees.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagHamincha120Minutes": "Plag hamincha according to the Magen Avraham, using the 120-minute day.\n\n10.75 shaos zmaniyos after alos 120 minutes before sunrise, using a day that starts 120 minutes before sunrise and ends 120 minutes after sunset.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha120MinutesZmanis": "Plag hamincha based on alos 120 zmaniyos minutes (one-sixth of the day) before sunrise.\n\n10.75 shaos zmaniyos after that alos.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha16Point1Degrees": "Plag hamincha based on the 16.1-degree day.\n\n10.75 shaos zmaniyos after alos 16.1 degrees, using a day that begins and ends at 16.1 degrees.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagHamincha18Degrees": "Plag hamincha based on the 18-degree day.\n\n10.75 shaos zmaniyos after alos 18 degrees, using a day that begins and ends at 18 degrees.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagHamincha19Point8Degrees": "Plag hamincha based on the 19.8-degree day.\n\n10.75 shaos zmaniyos after alos 19.8 degrees, using a day that begins and ends at 19.8 degrees.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagHamincha26Degrees": "Plag hamincha based on the 26-degree day.\n\n10.75 shaos zmaniyos after alos 26 degrees, using a day that begins and ends at 26 degrees.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nAt some northern and southern locations, this zman may not be available if the sun does not reach low enough below the horizon.",
    "getPlagHamincha60Minutes": "Plag hamincha according to the Magen Avraham, using the 60-minute day.\n\n10.75 shaos zmaniyos after alos 60 minutes before sunrise, using a day that starts 60 minutes before sunrise and ends 60 minutes after sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha72Minutes": "Plag hamincha according to the Magen Avraham, using the 72-minute day.\n\n10.75 shaos zmaniyos after alos 72 minutes before sunrise, using a day that starts 72 minutes before sunrise and ends 72 minutes after sunset.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha72MinutesZmanis": "Plag hamincha based on alos 72 zmaniyos minutes before sunrise.\n\n10.75 shaos zmaniyos after that alos.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha90Minutes": "Plag hamincha according to the Magen Avraham, using the 90-minute day.\n\n10.75 shaos zmaniyos after alos 90 minutes before sunrise, using a day that starts 90 minutes before sunrise and ends 90 minutes after sunset.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha90MinutesZmanis": "Plag hamincha based on alos 90 zmaniyos minutes before sunrise.\n\n10.75 shaos zmaniyos after that alos.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha96Minutes": "Plag hamincha according to the Magen Avraham, using the 96-minute day.\n\n10.75 shaos zmaniyos after alos 96 minutes before sunrise, using a day that starts 96 minutes before sunrise and ends 96 minutes after sunset.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHamincha96MinutesZmanis": "Plag hamincha based on alos 96 zmaniyos minutes before sunrise.\n\n10.75 shaos zmaniyos after that alos.\n\nThis zman should be used lechumra only. It can be a very late time, often after shkiah, and using it leniently can lead to chillul Shabbos.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHaminchaAteretTorah": "Plag hamincha according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\n10.75 shaos zmaniyos after alos.\n\nThe day begins at alos 1/10 of the day before sunrise and ends {ateret_torah_offset} after sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHaminchaBaalHatanya": "Plag hamincha according to the Baal Hatanya.\n\n10.75 shaos zmaniyos after netz amiti, using a day from Baal Hatanya sunrise to sunset.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.\n\nSee [About Our Zmanim Calculations @ Chabad.org](https://www.chabad.org/library/article_cdo/aid/3209349/jewish/About-Our-Zmanim-Calculations.htm).",
    "getPlagHaminchaGRA": "Plag hamincha according to the [Vilna Gaon](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n10.75 shaos zmaniyos after sunrise, using a day from sunrise to sunset. \n\nThis is the earliest time Shabbos can be started.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.",
    "getPlagHaminchaGRAFixedLocalChatzosToSunset": "Plag hamincha according to [Rav Moshe Feinstein](https://en.wikipedia.org/wiki/Moshe_Feinstein)'s opinion, following the [Vilna Gaon](https://en.wikipedia.org/wiki/Vilna_Gaon) with the day ending at sunset.\n\n4.75 shaos zmaniyos after fixed local chatzos.\n\nIn places where sunset cannot be calculated, this zman may not be available.",
    "getSamuchLeMinchaKetana16Point1Degrees": "The point near mincha ketana when eating or other activity should not begin before praying mincha, using the 16.1-degree day.\n\n9 shaos zmaniyos after alos at 16.1 degrees below the horizon, using a day that begins and ends at 16.1 degrees.\n\nThis is half a shaah zmanis before mincha ketana for this calculation.\n\nAt some northern and southern locations, including places even south of the Arctic Circle and north of the Antarctic Circle, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.\n\nSee [Mechaber and Mishna Berurah 232](https://hebrewbooks.org/pdfpager.aspx?req=60387&st=&pgnum=294) and [249:2](https://hebrewbooks.org/pdfpager.aspx?req=60388&pgnum=34).",
    "getSamuchLeMinchaKetana72Minutes": "The point near mincha ketana when eating or other activity should not begin before praying mincha, using the 72-minute day.\n\n9 shaos zmaniyos after alos 72 minutes before sunrise, using a day that starts 72 minutes before sunrise and ends 72 minutes after sunset.\n\nThis is half a shaah zmanis before mincha ketana for this calculation.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.\n\nSee [Mechaber and Mishna Berurah 232](https://hebrewbooks.org/pdfpager.aspx?req=60387&st=&pgnum=294) and [249:2](https://hebrewbooks.org/pdfpager.aspx?req=60388&pgnum=34).",
    "getSamuchLeMinchaKetanaGRA": "The point near mincha ketana when eating or other activity should not begin before praying mincha, following the [Vilna Gaon](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n9 shaos zmaniyos after sunrise, using a day from sunrise to sunset. \n\nThis is half a shaah zmanis before mincha ketana.\n\nIn places where sunrise or sunset cannot be calculated, this zman may not be available.\n\nSee [Mechaber and Mishna Berurah 232](https://hebrewbooks.org/pdfpager.aspx?req=60387&st=&pgnum=294) and [249:2](https://hebrewbooks.org/pdfpager.aspx?req=60388&pgnum=34).",
    "getSeaLevelSunrise": "Sunrise at sea level, without adjusting for the location's elevation.\n\nThis is the astronomical sunrise used as the reference for dawn times measured as degrees below the horizon. Dawn and dusk depend on visible light, which is not affected by elevation the way sunrise and sunset at a raised location can be.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSeaLevelSunset": "Sunset at sea level, without adjusting for the location's elevation.\n\nThis is the astronomical sunset used as the reference for dusk times measured as degrees below the horizon. Dawn and dusk depend on visible light, which is not affected by elevation the way sunrise and sunset at a raised location can be.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanAchilasChametzBaalHatanya": "Sof zman achilas chametz - the latest time to eat chametz on Erev Pesach according to the Baal Hatanya. Same as sof zman tfila Baal Hatanya.\n\n4 shaos zmaniyos after netz amiti (true sunrise), with the day measured from Baal Hatanya sunrise to sunset.",
    "getSofZmanAchilasChametzGRA": "Sof zman achilas chametz - the latest time to eat chametz on Erev Pesach according to the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon). Same as sof zman tfila GRA.\n\n4 shaos zmaniyos after sea level sunrise, with the day measured from sunrise to sunset.",
    "getSofZmanAchilasChametzMGA16Point1Degrees": "Sof zman achilas chametz - the latest time to eat chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 16.1-degree day.\n\n4 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to tzais at 16.1 degrees.",
    "getSofZmanAchilasChametzMGA72Minutes": "Sof zman achilas chametz - the latest time to eat chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72-minute day. Same as sof zman tfila MGA 72 minutes.\n\n4 shaos zmaniyos after alos 72 minutes before sunrise, with the day measured from alos 72 minutes before sunrise to tzais 72 minutes after sunset.",
    "getSofZmanAchilasChametzMGA72MinutesZmanis": "Sof zman achilas chametz - the latest time to eat chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72 zmaniyos-minute day. Same as sof zman tfila MGA 72 minutes zmanis.\n\n4 shaos zmaniyos after alos 72 zmaniyos minutes before sunrise, with the day measured from alos 72 zmaniyos minutes before sunrise to tzais 72 zmaniyos minutes after sunset.",
    "getSofZmanBiurChametzBaalHatanya": "Sof zman biur chametz - the latest time to burn chametz on Erev Pesach according to the Baal Hatanya.\n\n5 shaos zmaniyos after netz amiti (true sunrise), with the day measured from Baal Hatanya sunrise to sunset.",
    "getSofZmanBiurChametzGRA": "Sof zman biur chametz - the latest time to burn chametz on Erev Pesach according to the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n5 shaos zmaniyos after sea level sunrise, with the day measured from sunrise to sunset.",
    "getSofZmanBiurChametzMGA16Point1Degrees": "Sof zman biur chametz - the latest time to burn chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 16.1-degree day.\n\n5 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to tzais at 16.1 degrees.",
    "getSofZmanBiurChametzMGA72Minutes": "Sof zman biur chametz - the latest time to burn chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72-minute day.\n\n5 shaos zmaniyos after alos 72 minutes before sunrise, with the day measured from alos 72 minutes before sunrise to tzais 72 minutes after sunset.",
    "getSofZmanBiurChametzMGA72MinutesZmanis": "Sof zman biur chametz - the latest time to burn chametz on Erev Pesach according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72 zmaniyos-minute day.\n\n5 shaos zmaniyos after alos 72 zmaniyos minutes before sunrise, with the day measured from alos 72 zmaniyos minutes before sunrise to tzais 72 zmaniyos minutes after sunset.",
    "getSofZmanKidushLevana15Days": "The latest time for Kiddush Levana according to the Shulchan Aruch (Orach Chaim 426) - 15 days after the molad.\n\nSome hold that the [Rema](https://en.wikipedia.org/wiki/Moses_Isserles), who cites the [Maharil's](https://en.wikipedia.org/wiki/Yaakov_ben_Moshe_Levi_Moelin) approach of calculating halfway between molad and molad, reflects the Mechaber's view as well. See Rabbi Dovid Heber's detailed write-up in siman daled (chapter 4) of [Shaarei Zmanim](https://hebrewbooks.org/53000).\n\nNote that although this time may be during the daytime, Kiddush Levana cannot be said during the daytime.",
    "getSofZmanKidushLevanaBetweenMoldos": "The latest time for Kiddush Levana according to the [Maharil](https://en.wikipedia.org/wiki/Yaakov_ben_Moshe_Levi_Moelin) - halfway between this molad and the next.\n\nHalf of the 29 days, 12 hours, and 793 chalakim interval between molad and molad (14 days, 18 hours, 22 minutes, and 666 milliseconds) after the month's molad.\n\nNote that although this time may be during the daytime, Kiddush Levana cannot be said during the daytime.",
    "getSofZmanShma3HoursBeforeChatzos": 'Sof zman krias shema - the latest time to recite morning Shema, calculated as 3 regular clock hours before chatzos hayom (not shaos zmaniyos). Often grouped with the "Komarno" zmanim after [Rav Yitzchak Eizik of Komarno](https://en.wikipedia.org/wiki/Komarno_(Hasidic_dynasty)#Rabbi_Yitzchak_Eisik_Safrin), though this calculation is much older.\n\n3 clock hours before chatzos hayom.\n\nThis view is cited by the Shach in Nekudas Hakesef (Yoreh Deah 184), [Rav Moshe Lifshitz](https://hebrewbooks.org/pdfpager.aspx?req=21638&st=&pgnum=30) in [Lechem Mishneh on Brachos 1:2](https://hebrewbooks.org/pdfpager.aspx?req=21638&st=&pgnum=50), the [Yaavetz](https://en.wikipedia.org/wiki/Jacob_Emden), and later by Komarno, Shevus Yaakov, Chasan Sofer, and others. See also [Yisrael Vehazmanim vol. 1, 7:3](https://hebrewbooks.org/pdfpager.aspx?req=9765&st=&pgnum=83).\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.',
    "getSofZmanShmaAlos16Point1ToSunset": "Sof zman krias shema - the latest time to recite morning Shema according to the opinion of the [Chidushei V'Chlalot HaRazah](https://hebrewbooks.org/40357) and the [Menorah HaTehorah](https://hebrewbooks.org/14799), as cited in [Yisrael Vehazmanim vol. 1, sec. 7, ch. 3 no. 16](https://hebrewbooks.org/pdfpager.aspx?req=9765&pgnum=81).\n\n3 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to sea level sunset. By this calculation, chatzos is not at midday.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaAlos16Point1DegreesToTzaisGeonim7Point083Degrees": "Sof zman krias shema - the latest time to recite morning Shema using a day from alos at 16.1 degrees to tzais at 7.083 degrees.\n\n3 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to tzais at 7.083 degrees. By this calculation, chatzos is not at midday.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaAteretTorah": "Sof zman krias shema - the latest time to recite morning Shema according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\nAteret Torah zmanim use a day that begins at alos 1/10 of the day before sunrise and ends {ateret_torah_offset} after sunset. Sof zman krias shema is 3 of those shaos zmaniyos after that alos. By this calculation, chatzos is not at midday.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaBaalHatanya": "Sof zman krias shema - the latest time to recite morning Shema according to the Baal Hatanya.\n\n3 shaos zmaniyos after netz amiti (true sunrise), with the day measured from Baal Hatanya sunrise to sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaGRA": "Sof zman krias shema - the latest time to recite morning Shema according to the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n3 shaos zmaniyos after sunrise. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaGRASunriseToFixedLocalChatzos": "Sof zman krias shema - the latest time to recite morning Shema according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon) day, using only the first half of the day.\n\n3 shaos zmaniyos after sunrise, with shaos zmaniyos measured from sunrise to fixed local chatzos (half of that half-day). \n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA120Minutes": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 120-minute day. This is an extremely early time, used as a chumra.\n\n3 shaos zmaniyos after alos 120 minutes before sunrise, with the day measured from alos 120 minutes before sunrise to tzais 120 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA16Point1Degrees": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 16.1-degree day.\n\n3 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to tzais at 16.1 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos": "Sof zman krias shema - the latest time to recite morning Shema according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner) day, using only the first half of the day.\n\n3 shaos zmaniyos after alos at 16.1 degrees, with shaos zmaniyos measured from that alos to fixed local chatzos (half of that half-day).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA18Degrees": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 18-degree day.\n\n3 shaos zmaniyos after alos at 18 degrees, with the day measured from alos at 18 degrees to tzais at 18 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA18DegreesToFixedLocalChatzos": "Sof zman krias shema - the latest time to recite morning Shema according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner) day, using only the first half of the day.\n\n3 shaos zmaniyos after alos at 18 degrees, with shaos zmaniyos measured from that alos to fixed local chatzos (half of that half-day).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA19Point8Degrees": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 19.8-degree day.\n\n3 shaos zmaniyos after alos at 19.8 degrees, with the day measured from alos at 19.8 degrees to tzais at 19.8 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA72Minutes": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72-minute day.\n\n3 shaos zmaniyos after alos 72 minutes before sunrise, with the day measured from alos 72 minutes before sunrise to tzais 72 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA72MinutesToFixedLocalChatzos": "Sof zman krias shema - the latest time to recite morning Shema according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner) day, using only the first half of the day.\n\n3 shaos zmaniyos after alos 72 minutes before sunrise, with shaos zmaniyos measured from that alos to fixed local chatzos (half of that half-day).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA72MinutesZmanis": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72 zmaniyos-minute day.\n\n3 shaos zmaniyos after alos 72 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 72 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA90Minutes": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 90-minute day.\n\n3 shaos zmaniyos after alos 90 minutes before sunrise, with the day measured from alos 90 minutes before sunrise to tzais 90 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA90MinutesToFixedLocalChatzos": "Sof zman krias shema - the latest time to recite morning Shema according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner) day, using only the first half of the day.\n\n3 shaos zmaniyos after alos 90 minutes before sunrise, with shaos zmaniyos measured from that alos to fixed local chatzos (half of that half-day).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanShmaMGA90MinutesZmanis": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 90 zmaniyos-minute day.\n\n3 shaos zmaniyos after alos 90 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 90 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA96Minutes": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 96-minute day.\n\n3 shaos zmaniyos after alos 96 minutes before sunrise, with the day measured from alos 96 minutes before sunrise to tzais 96 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanShmaMGA96MinutesZmanis": "Sof zman krias shema - the latest time to recite morning Shema according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 96 zmaniyos-minute day.\n\n3 shaos zmaniyos after alos 96 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 96 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfila2HoursBeforeChatzos": 'Sof zman tfila - the latest time to recite morning prayers (Shacharis), calculated as 2 regular clock hours before chatzos hayom (not shaos zmaniyos). Paired with sof zman krias shema at 3 clock hours before chatzos; often grouped with the "Komarno" zmanim after [Rav Yitzchak Eizik of Komarno](https://en.wikipedia.org/wiki/Komarno_(Hasidic_dynasty)#Rabbi_Yitzchak_Eisik_Safrin), though this calculation is much older.\n\n2 clock hours before chatzos hayom.\n\nThis view is cited by the Shach in Nekudas Hakesef (Yoreh Deah 184), [Rav Moshe Lifshitz](https://hebrewbooks.org/pdfpager.aspx?req=21638&st=&pgnum=30) in [Lechem Mishneh on Brachos 1:2](https://hebrewbooks.org/pdfpager.aspx?req=21638&st=&pgnum=50), the [Yaavetz](https://en.wikipedia.org/wiki/Jacob_Emden), and later by Komarno, Shevus Yaakov, Chasan Sofer, and others. See also [Yisrael Vehazmanim vol. 1, 7:3](https://hebrewbooks.org/pdfpager.aspx?req=9765&st=&pgnum=83).\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.',
    "getSofZmanTfilaAteretTorah": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\nAteret Torah zmanim use a day that begins at alos 1/10 of the day before sunrise and ends {ateret_torah_offset} after sunset. Sof zman tfila is 4 of those shaos zmaniyos after that alos. By this calculation, chatzos is not at midday.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaBaalHatanya": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the Baal Hatanya.\n\n4 shaos zmaniyos after netz amiti (true sunrise), with the day measured from Baal Hatanya sunrise to sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaGRA": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon).\n\n4 shaos zmaniyos after sunrise. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaGRASunriseToFixedLocalChatzos": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to [Rav Moshe Feinstein's](https://en.wikipedia.org/wiki/Moshe_Feinstein) view of the [GRA](https://en.wikipedia.org/wiki/Vilna_Gaon) day, using only the first half of the day.\n\n4 shaos zmaniyos after sunrise, with shaos zmaniyos measured from sunrise to fixed local chatzos (two-thirds of that half-day). \n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanTfilaMGA120Minutes": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 120-minute day. This is an extremely early time, used as a chumra.\n\n4 shaos zmaniyos after alos 120 minutes before sunrise, with the day measured from alos 120 minutes before sunrise to tzais 120 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA16Point1Degrees": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 16.1-degree day.\n\n4 shaos zmaniyos after alos at 16.1 degrees, with the day measured from alos at 16.1 degrees to tzais at 16.1 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanTfilaMGA18Degrees": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 18-degree day.\n\n4 shaos zmaniyos after alos at 18 degrees, with the day measured from alos at 18 degrees to tzais at 18 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanTfilaMGA19Point8Degrees": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 19.8-degree day.\n\n4 shaos zmaniyos after alos at 19.8 degrees, with the day measured from alos at 19.8 degrees to tzais at 19.8 degrees.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getSofZmanTfilaMGA72Minutes": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72-minute day.\n\n4 shaos zmaniyos after alos 72 minutes before sunrise, with the day measured from alos 72 minutes before sunrise to tzais 72 minutes after sunset. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA72MinutesZmanis": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 72 zmaniyos-minute day.\n\n4 shaos zmaniyos after alos 72 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 72 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA90Minutes": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 90-minute day.\n\n4 shaos zmaniyos after alos 90 minutes before sunrise, with the day measured from alos 90 minutes before sunrise to tzais 90 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA90MinutesZmanis": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 90 zmaniyos-minute day.\n\n4 shaos zmaniyos after alos 90 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 90 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA96Minutes": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 96-minute day.\n\n4 shaos zmaniyos after alos 96 minutes before sunrise, with the day measured from alos 96 minutes before sunrise to tzais 96 minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSofZmanTfilaMGA96MinutesZmanis": "Sof zman tfila - the latest time to recite morning prayers (Shacharis) according to the [Magen Avraham (MGA)](https://en.wikipedia.org/wiki/Avraham_Gombiner), using the 96 zmaniyos-minute day.\n\n4 shaos zmaniyos after alos 96 zmaniyos minutes before sunrise, with the day measured from that alos to tzais 96 zmaniyos minutes after sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSolarMidnight": "Solar midnight - when the sun transits the lower celestial meridian (at its nadir).\n\nCalculated for the end of the current day. For example, solar midnight for February 8 is the moment between February 8 and February 9 when the sun is at its lowest point. See [The Definition of Chatzos](https://kosherjava.com/2020/07/02/definition-of-chatzos/) for details on the proper definition of solar noon and midnight.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getSunrise": "Sunrise, adjusted for the location's elevation.\n\nThe time when the upper edge of the sun appears above the horizon, accounting for atmospheric refraction and the sun's radius.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getPolarSunriseBenIshChai": "Sunrise calculated as the time when the sun is directly due east (azimuth 90 degrees) in polar regions on days when there is no sunrise.\n\nIf sunrise occurs that day, this zman is not available. In polar regions (the Arctic or Antarctic circles), there are days with no sunrise or sunset. Some opinions hold that during these periods, sunrise is when the sun is directly due east (azimuth 90 degrees), and sunset is when the sun is directly due west (azimuth 270 degrees), returned by `getPolarSunsetBenIshChai`. This follows [Rabbi Yehosef Schwarz](https://en.wikipedia.org/wiki/Joseph_Schwarz_(geographer)) in [Devarim Yosef - Derech Mevo Hashemesh](https://hebrewbooks.org/pdfpager.aspx?req=31703&pgnum=134) and [Devarim Yosef - Teshuvot, She'elah 8](https://hebrewbooks.org/pdfpager.aspx?req=159&pgnum=83), brought lehalacha by the [Ben Ish Chai](https://en.wikipedia.org/wiki/Yosef_Hayyim) in [Rav Pe'alim, chelek 2, Sod Yesharim siman 4](https://hebrewbooks.org/pdfpager.aspx?req=1401&pgnum=461). This time is close to six hours before astronomical chatzos, but depending on the season and location in the Arctic or Antarctic, it can be up to 46 minutes earlier or later.\n\nIf there is no sunrise that day and the sun does not reach azimuth 90 degrees, this zman may not be available or cannot be calculated.",
    "getSunset": "Sunset, adjusted for the location's elevation.\n\nThe time when the upper edge of the sun disappears below the horizon, accounting for atmospheric refraction and the sun's radius.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getPolarSunsetBenIshChai": "Sunset calculated as the time when the sun is directly due west (azimuth 270 degrees) in polar regions on days when there is no sunset.\n\nIf sunset occurs that day, this zman is not available. In polar regions (the Arctic or Antarctic circles), there are days with no sunrise or sunset. Some opinions hold that during these periods, sunset (the day-night boundary) is when the sun is directly due west (azimuth 270 degrees), and sunrise is when the sun is directly due east (azimuth 90 degrees), returned by `getPolarSunriseBenIshChai`. This follows [Rabbi Yehosef Schwarz](https://en.wikipedia.org/wiki/Joseph_Schwarz_(geographer)) in [Devarim Yosef - Derech Mevo Hashemesh](https://hebrewbooks.org/pdfpager.aspx?req=31703&pgnum=134) and [Devarim Yosef - Teshuvot, She'elah 8](https://hebrewbooks.org/pdfpager.aspx?req=159&pgnum=83), brought lehalacha by the [Ben Ish Chai](https://en.wikipedia.org/wiki/Yosef_Hayyim) in [Rav Pe'alim, chelek 2, Sod Yesharim siman 4](https://hebrewbooks.org/pdfpager.aspx?req=1401&pgnum=461). This time is close to six hours after astronomical chatzos, but depending on the season and location in the Arctic or Antarctic, it can be up to 46 minutes earlier or later.\n\nIf there is no sunset that day and the sun does not reach azimuth 270 degrees, this zman may not be available or cannot be calculated.",
    "getSunTransit": "Solar noon - when the sun transits the celestial meridian.\n\nAlso called sundial noon or astronomical chatzos hayom. See [The Definition of Chatzos](https://kosherjava.com/2020/07/02/definition-of-chatzos/) for details on the proper definition of solar noon.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTchilasZmanKidushLevana3Days": "The earliest time for Kiddush Levana according to [Rabbeinu Yonah](https://en.wikipedia.org/wiki/Yonah_Gerondi) - 3 days after the molad.\n\nNote that although this time may be during the daytime, Kiddush Levana cannot be said during the daytime.",
    "getTchilasZmanKidushLevana7Days": "The earliest time for Kiddush Levana according to the opinion that it should not be said until 7 days after the molad.\n\nNote that although this time may be during the daytime, Kiddush Levana cannot be said during the daytime.",
    "getTzais120Minutes": "Tzais (nightfall) according to [Rav Chaim Naeh](https://en.wikipedia.org/wiki/Avraham_Chaim_Naeh) - 120 minutes after sunset.\n\nBased on Ula's calculation of tzais as 5 mil after sunset, using the Rambam's 24-minute mil. \n\nThis zman should be used lechumra only, such as delaying the start of nighttime mitzvos. Using it leniently can lead to chillul Shabbos and similar serious errors.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais120Zmanis": "Tzais (nightfall) - 120 zmaniyos minutes after sea level sunset.\n\nThis zman should be used lechumra only, such as delaying the start of nighttime mitzvos. The sun is well below the 18-degree point in most places. Using it leniently can lead to chillul Shabbos and similar serious errors.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais16Point1Degrees": "Tzais (nightfall) - when the sun is 16.1 degrees below the western horizon after sunset.\n\nMatches Rabbeinu Tam's 72-minute tzais [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/) in Jerusalem.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzais18Degrees": "Tzais (nightfall) - when the sun is 18 degrees below the western horizon after sunset.\n\nCalculated the same way as alos at 18 degrees, but for the evening.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzais19Point8Degrees": "Tzais (nightfall) - when the sun is 19.8 degrees below the western horizon after sunset.\n\nDegree-based calculation corresponding to 90 minutes after sunset [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/) in Jerusalem.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzais26Degrees": "Tzais (nightfall) - when the sun is 26 degrees below the western horizon after sunset.\n\nThis zman should be used lechumra only, such as delaying the start of nighttime mitzvos. Using it leniently can lead to chillul Shabbos and similar serious errors.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzais50Minutes": "Tzais (nightfall) according to [Rav Moshe Feinstein](https://en.wikipedia.org/wiki/Moshe_Feinstein) for the New York area - 50 minutes after sunset.\n\n50 minutes after sunset. \n\nThis zman should not be used for latitudes other than ones similar to the New York area.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais60Minutes": "Tzais (nightfall) according to the [Chavas Yair](https://en.wikipedia.org/wiki/Yair_Bacharach) and [Divrei Malkiel](https://he.wikipedia.org/wiki/%D7%9E%D7%9C%D7%9B%D7%99%D7%90%D7%9C_%D7%A6%D7%91%D7%99_%D7%98%D7%A0%D7%A0%D7%91%D7%95%D7%99%D7%9D) - 60 minutes after sunset.\n\nBased on a 15-minute mil, for a total of 4 mil after sunset. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais72Minutes": "Tzais (nightfall) according to Rabbeinu Tam - 72 minutes after sunset, the time to walk 4 mil at 18 minutes per mil.\n\n72 standard clock minutes after sunset, any time of year and in any location. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais72Zmanis": "Tzais (nightfall) according to the [Minchas Cohen](https://en.wikipedia.org/wiki/Abraham_Cohen_Pimentel) - 72 zmaniyos minutes (1/10 of the day) after sea level sunset.\n\nThis is the Minchas Cohen's calculation of Rabbeinu Tam's tzais. Note that twilight does not vary in direct proportion to the length of the day, so this zman does not match astronomical reality.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais90Minutes": "Tzais (nightfall) according to the Magen Avraham - 90 minutes after sunset.\n\nBased on Ula's calculation of tzais as 5 mil after sunset, using the Rambam's 18-minute mil. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais90Zmanis": "Tzais (nightfall) - 90 zmaniyos minutes (1/8 of the day) after sea level sunset.\n\nKnown in Yiddish as the achtel zman, used in various kehilos.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais96Minutes": "Tzais (nightfall) - 96 minutes after sunset.\n\nBased on the time to walk 4 mil at 24 minutes per mil. \n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzais96Zmanis": "Tzais (nightfall) - 96 zmaniyos minutes (1/7.5 of the day) after sea level sunset.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzaisAteretTorah": "Tzais (nightfall) according to the Ateret Torah calculation of Chacham Yosef Harari-Raful of Yeshivat Ateret Torah.\n\n{ateret_torah_offset} after sunset. Chacham Harari-Raful uses this timing for calculating other zmanim (such as sof zman krias shema and plag hamincha), but his calendars do not publish a separate tzais zman. A 25-minute offset was provided for Israel.\n\nIn places such as the Arctic Circle, where there is at least one day a year when the sun does not rise and one when it does not set, this zman may not be available or cannot be calculated.",
    "getTzaisBaalHatanya": "Tzais (nightfall) according to the [Baal Hatanya](https://en.wikipedia.org/wiki/Shneur_Zalman_of_Liadi) - when the sun is 6 degrees below the western horizon after sunset.\n\nBased on shkiah amitis plus 18 minutes (3/4 of a 24-minute mil) and 2 minutes for bain hashmashos of Rav Yosi, about 24 minutes after sunset in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/). See [About Our Zmanim Calculations at Chabad.org](https://www.chabad.org/library/article_cdo/aid/3209349/jewish/About-Our-Zmanim-Calculations.htm).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim3Point7Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 3.7 degrees below the western horizon after sunset.\n\nCorresponds to 13.5 minutes after sunset (3/4 of a mil at 18 minutes per mil) in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/). Does not include the time to walk 49 amos for bain hashmashos of Rav Yosi.",
    "getTzaisGeonim3Point8Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 3.8 degrees below the western horizon after sunset.\n\nCorresponds to 14 minutes after sunset: 13.5 minutes for 3/4 of an 18-minute mil, plus 30 seconds for 49 amos (bain hashmashos of Rav Yosi), in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/).",
    "getTzaisGeonim4Point42Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 4.42 degrees below the western horizon after sunset.\n\nBased on 3/4 of a 22.5-minute mil (16 7/8 minutes after sunset). This is a very early zman and should not be relied on without rabbinical guidance. Does not include the time to walk 49 amos for bain hashmashos of Rav Yosi.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim4Point66Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 4.66 degrees below the western horizon after sunset.\n\nBased on 3/4 of a 24-minute mil (18 minutes after sunset). This is a very early zman and should not be relied on without rabbinical guidance. Does not include the time to walk 49 amos for bain hashmashos of Rav Yosi.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim4Point8Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 4.8 degrees below the western horizon after sunset.\n\n18.6 minutes after sunset: 3/4 of a 24-minute mil plus time for 49 amos (bain hashmashos of Rav Yosi). Based on [Rav Yechiel Michel Shlezinger's](https://he.wikipedia.org/wiki/%D7%99%D7%97%D7%99%D7%90%D7%9C_%D7%9E%D7%99%D7%9B%D7%9C_%D7%A9%D7%9C%D7%96%D7%99%D7%A0%D7%92%D7%A8) [Aizehu Bain Hashmashos](https://www.nli.org.il/he/books/NNL_ALEPH997010042055805171/NLI) and [Rabbi Yehuda (Leo) Levi's](https://en.wikipedia.org/wiki/Yehuda_(Leo)_Levi) calculations in [Zmanei Hayom BaHalacha](https://www.nli.org.il/en/items/NNL_ALEPH990022548970205171/NLI). This is an early zman and should not be relied on without rabbinical guidance.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim5Point95Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 5.95 degrees below the western horizon after sunset.\n\nAbout 24 minutes after sunset in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), based on 18 minutes (3/4 of a 24-minute mil) plus shkiah amitis and bain hashmashos of Rav Yosi. Chabad calendars usually use the related 6-degree [Baal Hatanya](https://en.wikipedia.org/wiki/Shneur_Zalman_of_Liadi) tzais built on this calculation.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim6Point45Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 6.45 degrees below the western horizon after sunset.\n\nCommonly used in Israel. Based on [Rabbi Yechiel Michel Tucazinsky's](https://en.wikipedia.org/wiki/Yechiel_Michel_Tucazinsky) calculation, about 31 minutes after sea level sunset in Jerusalem and about 26.5 minutes at the equinox. Also used in [Luach Itim Lebinah](https://www.worldcat.org/oclc/243303103). See [Birur Halacha Yoreh Deah 262](https://hebrewbooks.org/pdfpager.aspx?req=50536&st=&pgnum=51).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim7Point083Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 7.083 degrees (7 degrees 5 minutes) below the western horizon after sunset.\n\nBased on Dr. Baruch (Berthold) Cohn's observation of 3 medium-sized stars in his [1899 luach](https://sammlungen.ub.uni-frankfurt.de/freimann/content/titleinfo/983088). Endorsed by [Rav Dovid Tzvi Hoffman](https://en.wikipedia.org/wiki/David_Zvi_Hoffmann) in [Melamed Leho'il Orach Chaim 30](https://hebrewbooks.org/pdfpager.aspx?req=1053&st=&pgnum=37). Close to the [Makor Chessed](https://hebrewbooks.org/22044) of the Sefer Chasidim and to about 30 minutes after sunset in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/), but not exactly.\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim7Point67Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 7.67 degrees below the western horizon after sunset.\n\nCorresponds to 45 minutes after sunset during the summer solstice in New York, when twilight is longest. Cited in [Igros Moshe Even Haezer 4, ch. 4](https://hebrewbooks.org/pdfpager.aspx?req=921&pgnum=149) regarding tzais for krias shema, and in Rabbi Heber's [Shaarei Zmanim](https://hebrewbooks.org/53000) ([chapter 10, page 87](https://hebrewbooks.org/pdfpager.aspx?req=53055&pgnum=101) and [chapter 12, page 108](https://hebrewbooks.org/pdfpager.aspx?req=53055&pgnum=122)). Also endorsed by [Rabbi Shmuel Kamenetsky](https://en.wikipedia.org/wiki/Shmuel_Kamenetsky).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim8Point5Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 8.5 degrees below the western horizon after sunset.\n\nBased on Rabbi Meir Posen's [Ohr Meir](https://www.worldcat.org/oclc/29283612) calculation for when 3 small stars are visible, which is later than the required 3 medium stars. About 36 minutes after sunset in Jerusalem [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim9Point3Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 9.3 degrees below the western horizon after sunset.\n\nThe stringent tzais used in [Luach Itim Lebinah](https://www.worldcat.org/oclc/243303103).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
    "getTzaisGeonim9Point75Degrees": "Tzais (nightfall) according to the Geonim - when the sun is 9.75 degrees below the western horizon after sunset.\n\nCorresponds to 60 minutes after sunset [around the equinox/equilux](https://kosherjava.com/2022/01/12/equinox-vs-equilux-zmanim-calculations/) in New York, when a solar hour is 60 minutes. The opinion of [Rabbi Eliyahu Henkin](https://en.wikipedia.org/wiki/Yosef_Eliyahu_Henkin) and [Rabbi Shmuel Kamenetsky](https://en.wikipedia.org/wiki/Shmuel_Kamenetsky).\n\nAt some northern and southern locations, this zman may not be available or cannot be calculated if the sun does not reach low enough below the horizon.",
}
