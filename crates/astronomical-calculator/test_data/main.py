import csv
import random
import numpy as np
from datetime import datetime, timedelta, timezone
from skyfield import almanac
from skyfield.api import load, wgs84
from concurrent.futures import ProcessPoolExecutor
import multiprocessing

# Settings
OUTPUT_FILENAME = 'solar_test_data1.csv'
NUM_ROWS = 1000  # How many test cases to generate

def get_random_inputs():
    """Generates random test inputs."""
    # Latitude: -65 to 65 to ensure we mostly get valid sunrises/sets for basic testing.
    # You can expand to -90/90 to test Polar handling in your Rust code.
    lat = random.uniform(-65.0, 65.0) 
    lon = random.uniform(-180.0, 180.0)
    elev = random.uniform(0.0, 2500.0) # Meters
    
    # Random date in the last year, always at midday (12:00 UTC)
    end_date = datetime.now(timezone.utc)
    start_date = end_date - timedelta(days=365)
    random_days = random.randint(0, 365)
    date_utc = (start_date + timedelta(days=random_days)).replace(hour=12, minute=0, second=0, microsecond=0)
    
    # Random Atmos
    pressure = random.uniform(980.0, 1030.0)
    temp_c = random.uniform(-10.0, 35.0)
    
    return lat, lon, elev, date_utc, pressure, temp_c

def find_closest_time(times, reference_time):
    """Finds the time in list 'times' closest to 'reference_time'."""
    if not times:
        return None
    # Calculate absolute differences using TT (Terrestrial Time) floats for speed
    diffs = [abs(t.tt - reference_time.tt) for t in times]
    idx = np.argmin(diffs)
    return times[idx]

def generate_row_worker(seed):
    """Worker function for parallel processing. Each worker loads its own ephemeris."""
    random.seed(seed)
    
    # Each worker loads its own ephemeris
    eph = load('de421.bsp')
    ts = load.timescale()
    sun = eph['Sun']
    earth = eph['Earth']
    
    lat_deg, lon_deg, elev_m, date_utc, pres_mb, temp_c = get_random_inputs()
    
    t_input = ts.from_datetime(date_utc)
    loc = wgs84.latlon(lat_deg, lon_deg, elevation_m=elev_m)
    observer = earth + loc

    # ---------------------------------------------------------
    # 1. SPA Data (Instantaneous Position)
    # ---------------------------------------------------------
    
    # Apparent position (Topo)
    astrometric = observer.at(t_input).observe(sun)
    apparent = astrometric.apparent()
    alt, az, _ = apparent.altaz(pressure_mbar=pres_mb, temperature_C=temp_c)
    
    zenith_deg = 90.0 - alt.degrees
    azimuth_deg = az.degrees

    # Delta T and UT1
    delta_t = t_input.delta_t
    dut1 = t_input.dut1

    # ---------------------------------------------------------
    # 2. Almanac Data (Events)
    # ---------------------------------------------------------
    
    # Search window: +/- 2 days to ensure we find surrounding midnights
    t_start = ts.from_datetime(date_utc - timedelta(days=2))
    t_end = ts.from_datetime(date_utc + timedelta(days=2))

    # --- A. Transits (Noon/Midnight) ---
    f_transits = almanac.meridian_transits(eph, sun, loc)
    t_ev, y_ev = almanac.find_discrete(t_start, t_end, f_transits)
    
    # Event 1: Solar Transit (Noon) closest to input t
    noons = t_ev[y_ev == 1]
    transit_noon = find_closest_time(noons, t_input)

    # If we are in a polar night/day where transit doesn't happen strictly, handle None
    if transit_noon is None:
        # Fallback logic for polar regions could go here, 
        # but for this test gen we return None to indicate "No Event"
        pass

    # Event 0: Midnight BEFORE input t
    midnights = t_ev[y_ev == 0]
    midnights_before = [tm for tm in midnights if tm.tt < t_input.tt]
    transit_mid_prev = midnights_before[-1] if midnights_before else None

    # Event 2: Midnight AFTER input t
    midnights_after = [tm for tm in midnights if tm.tt > t_input.tt]
    transit_mid_next = midnights_after[0] if midnights_after else None

    # --- B. Rise / Set / Twilights ---
    # We anchor these events to the Computed Noon (transit_noon).
    # If Noon is None (Polar), we fallback to input t.
    anchor_time = transit_noon if transit_noon is not None else t_input

    def get_event_pair(horizon_deg):
        f = almanac.risings_and_settings(eph, sun, loc, horizon_degrees=horizon_deg)
        t_r, y_r = almanac.find_discrete(t_start, t_end, f)
        
        # 1 = Rise, 0 = Set
        rises = t_r[y_r == 1]
        sets = t_r[y_r == 0]
        
        # Find the Rise and Set closest to the Solar Noon of this cycle
        r_closest = find_closest_time(rises, anchor_time)
        s_closest = find_closest_time(sets, anchor_time)
        return r_closest, s_closest

    # Standard Rise/Set (-0.8333 deg)
    ev_rise, ev_set = get_event_pair(-0.8333)
    # Civil (-6 deg)
    civ_rise, civ_set = get_event_pair(-6.0)
    # Nautical (-12 deg)
    nav_rise, nav_set = get_event_pair(-12.0)
    # Astronomical (-18 deg)
    astro_rise, astro_set = get_event_pair(-18.0)

    # ---------------------------------------------------------
    # 3. Format Output
    # ---------------------------------------------------------
    def to_unix(skyfield_t):
        if skyfield_t is None:
            return "" # CSV Empty String for NULL
        # Skyfield utc_datetime() returns a python datetime object
        return str(int(skyfield_t.utc_datetime().timestamp()))

    row = {
        # Inputs
        "input_timestamp_unix": int(date_utc.timestamp()),
        "lat_deg": f"{lat_deg:.6f}",
        "lon_deg": f"{lon_deg:.6f}",
        "elev_m": f"{elev_m:.2f}",
        "pressure_mb": f"{pres_mb:.2f}",
        "temp_c": f"{temp_c:.2f}",
        "delta_t_s": f"{delta_t:.6f}",
        "delta_ut1_s": f"{dut1:.6f}",
        
        # SPA Outputs
        "spa_zenith_deg": f"{zenith_deg:.6f}",
        "spa_azimuth_deg": f"{azimuth_deg:.6f}",
        
        # Almanac Outputs (Unix Timestamps)
        "ev_0_midnight_pre": to_unix(transit_mid_prev),
        "ev_1_noon_transit": to_unix(transit_noon),
        "ev_2_midnight_post": to_unix(transit_mid_next),
        "ev_3_sunrise": to_unix(ev_rise),
        "ev_4_sunset": to_unix(ev_set),
        "ev_5_civil_dawn": to_unix(civ_rise),
        "ev_6_civil_dusk": to_unix(civ_set),
        "ev_7_naut_dawn": to_unix(nav_rise),
        "ev_8_naut_dusk": to_unix(nav_set),
        "ev_9_astro_dawn": to_unix(astro_rise),
        "ev_10_astro_dusk": to_unix(astro_set),
    }
    return row

def main():
    print(f"Generating {NUM_ROWS} test rows using {multiprocessing.cpu_count()} CPU cores...")
    
    # Define Column Order
    headers = [
        "input_timestamp_unix", "lat_deg", "lon_deg", "elev_m", 
        "pressure_mb", "temp_c", "delta_t_s", "delta_ut1_s",
        "spa_zenith_deg", "spa_azimuth_deg",
        "ev_0_midnight_pre", "ev_1_noon_transit", "ev_2_midnight_post",
        "ev_3_sunrise", "ev_4_sunset",
        "ev_5_civil_dawn", "ev_6_civil_dusk",
        "ev_7_naut_dawn", "ev_8_naut_dusk",
        "ev_9_astro_dawn", "ev_10_astro_dusk"
    ]

    # Generate unique seeds for each row to ensure reproducible random data
    seeds = [random.randint(0, 2**32 - 1) for _ in range(NUM_ROWS)]
    
    # Use multiprocessing to generate rows in parallel
    rows = []
    with ProcessPoolExecutor(max_workers=multiprocessing.cpu_count()) as executor:
        for i, row in enumerate(executor.map(generate_row_worker, seeds)):
            rows.append(row)
            if (i + 1) % 50 == 0 or (i + 1) == NUM_ROWS:
                print(f"  Processed {i + 1}/{NUM_ROWS} rows...")
    
    print(f"Writing {NUM_ROWS} rows to file...")
    with open(OUTPUT_FILENAME, mode='w', newline='') as csv_file:
        writer = csv.DictWriter(csv_file, fieldnames=headers)
        writer.writeheader()
        writer.writerows(rows)

    print(f"Done. Saved to {OUTPUT_FILENAME}")

if __name__ == "__main__":
    main()