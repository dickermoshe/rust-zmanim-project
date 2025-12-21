# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "numpy",
#     "pandas",
#     "pvlib",
# ]
# ///
import pvlib
import pandas as pd
import numpy as np
from datetime import datetime, timezone, timedelta
import csv


def generate_random_data(count=50000,reasonable:bool = True):
    """
    Generate random locations and timestamps worldwide.
    Range: 1870 to 2070.
    """
    np.random.seed(42)

    # Generate random latitudes, longitudes and altitudes
    latitudes = np.random.uniform(-60 if reasonable else -90, 60 if reasonable else 90, count)
    longitudes = np.random.uniform(-120 if reasonable else -180, 120 if reasonable else 180, count)
    altitudes = np.random.uniform(0, 5000, count)

    # Generate random timestamps between 1870 and 2070
    start_date = datetime(1870, 1, 1, tzinfo=timezone.utc)
    end_date = datetime(2070, 12, 31, 23, 59, 59, tzinfo=timezone.utc)
    epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)

    start_ts = int((start_date - epoch).total_seconds())
    end_ts = int((end_date - epoch).total_seconds())

    # Use uniform and cast to int64 to avoid int32 overflow in randint
    timestamps = np.random.uniform(start_ts, end_ts, count).astype(np.int64)

    return list(zip(latitudes, longitudes, altitudes, timestamps))


def calculate_sun_times(latitude, longitude, altitude, timestamp_unix):
    """
    Calculate sunrise, transit, and sunset times for a location and timestamp.
    Also returns solar position (zenith, azimuth, etc.) at the specific timestamp.
    All times are in UTC.
    """
    epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
    dt_utc = epoch + timedelta(seconds=int(timestamp_unix))

    # Create location with UTC timezone
    location = pvlib.location.Location(
        latitude=latitude, longitude=longitude, tz="UTC", altitude=altitude
    )

    # Create a pandas DatetimeIndex with the date (UTC)
    times_index = pd.DatetimeIndex([dt_utc], tz="UTC")


    # Get sunrise/sunset/transit times
    times = location.get_sun_rise_set_transit(times_index)
    eot = location.get_solarposition(times_index, pressure=101325)[
        "equation_of_time"
    ].iloc[0]


    sunrise = times["sunrise"].iloc[0]
    transit = times["transit"].iloc[0]
    sunset = times["sunset"].iloc[0]

    # Get solar position at the input timestamp
    solpos = location.get_solarposition(times_index)
    zenith = solpos["zenith"].iloc[0]
    azimuth = solpos["azimuth"].iloc[0]
    azimuth_astronomical = (azimuth - 180) % 360
    incidence = zenith  # Assuming horizontal surface

    # Handle NaT (Not a Time) - occurs in polar regions
    sunrise_rfc = None
    transit_rfc = None
    sunset_rfc = None

    if sunrise is not None and not pd.isna(sunrise):
        # get as rfc3339
        sunrise_rfc = sunrise.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
    if transit is not None and not pd.isna(transit):
        transit_rfc = transit.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
    if sunset is not None and not pd.isna(sunset):
        sunset_rfc = sunset.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

    return (
        sunrise_rfc,
        transit_rfc,
        sunset_rfc,
        zenith,
        azimuth_astronomical,
        azimuth,
        incidence,
        dt_utc,
        eot
    )




def generate_csv(output_file:str, count:int, reasonable:bool):
    """
    Generate CSV file with random locations, dates, and sun times.
    """
    data = generate_random_data(count,reasonable)

    with open(output_file, "w", newline="", encoding="utf-8") as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(
            [
                "latitude",
                "longitude",
                "altitude",
                "input_timestamp",
                "sunrise_rfc",
                "transit_rfc",
                "sunset_rfc",
                "zenith",
                "azimuth_astronomical",
                "azimuth",
                "incidence",
                "eot",
            ]
        )

        for lat, lon, alt, ts in data:
            alt = 0
            (
                sunrise,
                transit,
                sunset,
                zenith,
                az_astro,
                az_obs,
                incidence,
                dt_utc,
                eot,
            ) = calculate_sun_times(lat, lon, alt, ts)

            if zenith is not None:
                writer.writerow(
                    [
                        lat,
                        lon,
                        alt,
                        dt_utc.strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
                        sunrise,
                        transit,
                        sunset,
                        zenith,
                        az_astro,
                        az_obs,
                        incidence,
                        eot,
                    ]
                )
            else:
                writer.writerow([lat, lon, alt, int(ts), "", "", "", "", "", "", ""])


if __name__ == "__main__":
    generate_csv("reasonable.csv", count=5_000, reasonable=True)
    generate_csv("unreasonable.csv", count=5_000, reasonable=False)
