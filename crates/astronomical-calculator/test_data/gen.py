from skyfield.timelib import Time
from pydantic import BaseModel, Field, field_validator, validator
import csv
from faker import Faker
import datetime as dt
from datetime import datetime
from pytz import timezone
from skyfield import almanac
from skyfield.api import wgs84, load
from skyfield.toposlib import GeographicPosition
from concurrent.futures import ProcessPoolExecutor, as_completed
from tqdm import tqdm
from timezonefinder import TimezoneFinder

tf = TimezoneFinder()
fake = Faker()


class City(BaseModel):
    name: str
    timezone: str
    coordinates: str
    elevation: float = Field(
        default=0.0,
    )  # Emtpy string to 0 meters

    @field_validator("elevation", mode="before")
    def validate_elevation(cls, v):
        if v == "":
            return 0.0
        return float(v)


class Result(BaseModel):
    lat: float
    lon: float
    elevation: float
    midday: float
    sunrise: float | None
    sunset: float | None
    civil_dawn: float | None
    civil_dusk: float | None
    nautical_dawn: float | None
    nautical_dusk: float | None
    astronomical_dawn: float | None
    astronomical_dusk: float | None
    transit: float
    now: str


class Results(BaseModel):
    results: list[Result]


# Module-level globals that will be initialized per process
eph = None
ts = None
_worker_fake = None


def init_worker():
    """Initialize worker process with ephemeris and timescale."""
    global eph, ts, _worker_fake
    eph = load("de440s.bsp")
    ts = load.timescale()
    _worker_fake = Faker()


def find_transit(d: datetime, p: GeographicPosition) -> Time:
    midnight = d.replace(hour=0, minute=0, second=0, microsecond=0)
    next_midnight = midnight + dt.timedelta(days=1)
    t0 = ts.from_datetime(midnight)
    t1 = ts.from_datetime(next_midnight)
    f = almanac.meridian_transits(eph, eph["Sun"], p)
    times, events = almanac.find_discrete(t0, t1, f)
    times = times[events == 1]
    return times[0]


def find_sunrise(
    yesterday_transit: Time, today_transit: Time, p: GeographicPosition
) -> Time | None:
    observer = eph["Earth"] + p
    f = almanac.find_risings(observer, eph["Sun"], yesterday_transit, today_transit)
    if len(f) == 0:
        return None
    f = f[0]
    if len(f) == 0:
        return None
    return f[0]


def find_sunset(
    yesterday_transit: Time, today_transit: Time, p: GeographicPosition
) -> Time | None:
    observer = eph["Earth"] + p
    f = almanac.find_settings(observer, eph["Sun"], yesterday_transit, today_transit)
    if len(f) == 0:
        return None
    f = f[0]
    if len(f) == 0:
        return None
    return f[0]


def process_city(city_data: dict) -> dict | None:
    """Process a single city and return a Result dict, or None if processing fails.

    Args:
        city_data: Dictionary with 'name', 'timezone', and 'coordinates' keys.

    Returns:
        Dictionary with Result fields, or None on error.
    """
    global eph, ts, _worker_fake
    try:
        lat, lon = city_data["coordinates"].split(",")
        lat = float(lat)
        lon = float(lon)
        elevation = city_data["elevation"]

        zone = timezone(city_data["timezone"])
        now = zone.localize(
            _worker_fake.date_time_between(
                start_date=dt.date(1900, 1, 1), end_date=dt.date(2100, 12, 31)
            )
        )
        midday = now.replace(hour=12, minute=0, second=0, microsecond=0)
        location = wgs84.latlon(lat, lon, elevation)

        today_transit = find_transit(now, location)
        yesterday_transit = find_transit(now - dt.timedelta(days=1), location)
        tomorrow_transit = find_transit(now + dt.timedelta(days=1), location)
        sunrise = find_sunrise(yesterday_transit, today_transit, location)
        sunset = find_sunset(today_transit, tomorrow_transit, location)
        sunrise_dt = sunrise.astimezone(zone) if sunrise is not None else None
        transit_dt = today_transit.astimezone(zone)
        sunset_dt = sunset.astimezone(zone) if sunset is not None else None

        f = almanac.dark_twilight_day(eph, location)
        times, events = almanac.find_discrete(yesterday_transit, today_transit, f)

        def get_dawn_timestamp(event_id):
            matches = times[events == event_id]
            dt = matches[-1].astimezone(zone) if len(matches) > 0 else None
            return dt.timestamp() if dt is not None else None

        astronomical_dawn_dt = get_dawn_timestamp(1)
        nautical_dawn_dt = get_dawn_timestamp(2)
        civil_dawn_dt = get_dawn_timestamp(3)

        times, events = almanac.find_discrete(today_transit, tomorrow_transit, f)

        def get_dusk_timestamp(event_id):
            matches = times[events == event_id]
            dt = matches[0].astimezone(zone) if len(matches) > 0 else None
            return dt.timestamp() if dt is not None else None

        # Event IDs: 0=dark, 1=astronomical twilight, 2=nautical twilight, 3=civil twilight, 4=sun up
        # For dusk (sun going down): transitions TO these states
        # 4→3 = sunset, 3→2 = civil dusk, 2→1 = nautical dusk, 1→0 = astronomical dusk
        astronomical_dusk_dt = get_dusk_timestamp(
            0
        )  # Transition to dark (from astronomical twilight)
        nautical_dusk_dt = get_dusk_timestamp(
            1
        )  # Transition to astronomical twilight (from nautical twilight)
        civil_dusk_dt = get_dusk_timestamp(
            2
        )  # Transition to nautical twilight (from civil twilight)

        return {
            "lat": lat,
            "lon": lon,
            "elevation": elevation,
            "midday": midday.timestamp(),
            "sunrise": sunrise_dt.timestamp() if sunrise_dt is not None else None,
            "sunset": sunset_dt.timestamp() if sunset_dt is not None else None,
            "transit": transit_dt.timestamp(),
            "now": now.isoformat(),
            "civil_dawn": civil_dawn_dt,
            "nautical_dawn": nautical_dawn_dt,
            "astronomical_dawn": astronomical_dawn_dt,
            "astronomical_dusk": astronomical_dusk_dt,
            "nautical_dusk": nautical_dusk_dt,
            "civil_dusk": civil_dusk_dt,
        }
    except Exception as e:
        # Log error but don't crash - just skip this city
        print(f"Error processing city {city_data.get('name', 'unknown')}: {e}")
        return None


if __name__ == "__main__":
    # Read cities from CSV
    with open("cities.csv", "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        cities = [City.model_validate(row) for row in reader]

    # Process cities in parallel
    # Convert City objects to dicts for pickling
    city_dicts = [
        {
            "name": city.name,
            "timezone": city.timezone,
            "coordinates": city.coordinates,
            "elevation": city.elevation,
            "index": index,
        }
        for index, city in enumerate(cities)
    ]

    # Add some random data to the list
    for _ in range(1000):
        lat = fake.latitude()
        lon = fake.longitude()
        tz = tf.timezone_at(lat=lat, lng=lon)
        if tz is None:
            continue

        city_dicts.append(
            {
                "name": fake.city(),
                "timezone": tz,
                "coordinates": f"{fake.latitude()},{fake.longitude()}",
                "elevation": fake.random_int(min=0, max=1000),
                "index": len(city_dicts),
            }
        )

    results_list = []
    with ProcessPoolExecutor(
        max_workers=10, initializer=init_worker
    ) as executor:  # None uses default (CPU count)
        # Submit all tasks
        future_to_city = {
            executor.submit(process_city, city_dict): city_dict
            for city_dict in city_dicts
        }

        # Collect results as they complete with progress bar
        for future in tqdm(
            as_completed(future_to_city),
            total=len(city_dicts),
            desc="Processing cities",
        ):
            result_dict = future.result()
            if result_dict is not None:
                # Convert dict back to Result object
                results_list.append(Result(**result_dict))

    results = Results(results=results_list)

    # Save as JSON
    with open("data.json", "w", encoding="utf-8") as f:
        f.write(results.model_dump_json())
