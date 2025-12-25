from skyfield.timelib import Time
from pydantic import BaseModel
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

fake = Faker()


class City(BaseModel):
    name: str
    timezone: str
    coordinates: str


class Result(BaseModel):
    lat: float
    lon: float
    midday: float
    sunrise: float | None
    sunset: float | None
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

        zone = timezone(city_data["timezone"])
        now = zone.localize(
            _worker_fake.date_time_between(
                start_date=dt.date(1900, 1, 1), end_date=dt.date(2100, 12, 31)
            )
        )
        midday = now.replace(hour=12, minute=0, second=0, microsecond=0)
        location = wgs84.latlon(lat, lon)

        today_transit = find_transit(now, location)
        yesterday_transit = find_transit(now - dt.timedelta(days=1), location)
        tomorrow_transit = find_transit(now + dt.timedelta(days=1), location)
        sunrise = find_sunrise(yesterday_transit, today_transit, location)
        sunset = find_sunset(today_transit, tomorrow_transit, location)
        sunrise_dt = sunrise.astimezone(zone) if sunrise is not None else None
        transit_dt = today_transit.astimezone(zone)
        sunset_dt = sunset.astimezone(zone) if sunset is not None else None

        return {
            "lat": lat,
            "lon": lon,
            "midday": midday.timestamp(),
            "sunrise": sunrise_dt.timestamp() if sunrise_dt is not None else None,
            "sunset": sunset_dt.timestamp() if sunset_dt is not None else None,
            "transit": transit_dt.timestamp(),
            "now": now.isoformat(),
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
            "index": index,
        }
        for index, city in enumerate(cities)
    ]

    # Add some random data to the list
    for _ in range(1000):
        city_dicts.append(
            {
                "name": fake.city(),
                "timezone": fake.timezone(),
                "coordinates": f"{fake.latitude()},{fake.longitude()}",
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
