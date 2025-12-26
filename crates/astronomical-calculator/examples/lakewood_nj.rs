#![allow(missing_docs, clippy::unwrap_used)]
use astronomical_calculator::{AstronomicalCalculator, Refraction, SolarEventResult};
use chrono::{TimeZone, Timelike, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Lakewood, NJ coordinates
    let latitude = 40.070591415768035;
    let longitude: f64 = -74.20516698767808;
    let elevation = 23.0; // meters above sea level

    // Get current date and create noon UTC time for today
    let now_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System time is before Unix epoch")?
        .as_secs() as i64;
    let now_datetime = chrono_tz::Tz::America__New_York
        .timestamp_opt(now_timestamp, 0)
        .single()
        .ok_or("Invalid timestamp")?;
    let dt = now_datetime.with_hour(12).unwrap();

    println!("Astronomical Calculator Example - Lakewood, NJ");
    println!("Location: {:.5}°N, {:.5}°W", latitude, longitude.abs());
    println!("Date: {}", dt.format("%B %d, %Y"));
    println!("Elevation: {:.1} meters", elevation);
    println!("{:=<60}", "");

    // Create calculator for Lakewood, NJ
    let mut calc = AstronomicalCalculator::new(
        dt.naive_utc(),
        None,                         // Calculate ΔT automatically
        0.0,                          // ΔUT1 (use 0.0 if unknown)
        longitude,                    // longitude in degrees (negative = West)
        latitude,                     // latitude in degrees (positive = North)
        elevation,                    // elevation in meters
        5.0,                          // temperature in Celsius
        1021.3,                       // pressure in millibars
        None,                         // geometric dip (None = standard horizon)
        Refraction::ApSolposBennetNA, // refraction model (recommended for precision)
    )?;

    // Get current solar position
    let position = calc.get_solar_position();
    println!("Current Solar Position (at {} UTC):", dt.format("%H:%M:%S"));
    println!("  Zenith angle: {:.2}°", position.zenith.to_degrees());
    println!("  Azimuth angle: {:.2}°", position.azimuth.to_degrees());
    println!("  Elevation angle: {:.2}°", (90.0 - position.zenith.to_degrees()));
    println!();

    // Calculate sunrise and sunset
    match calc.get_sunrise()? {
        SolarEventResult::Occurs(timestamp) => {
            let sunrise_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "Sunrise: {}",
                sunrise_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        SolarEventResult::AllDay => println!("Sun never sets (midnight sun)"),
        SolarEventResult::AllNight => println!("Sun never rises (polar night)"),
    }

    match calc.get_sunset()? {
        SolarEventResult::Occurs(timestamp) => {
            let sunset_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "Sunset: {}",
                sunset_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        SolarEventResult::AllDay => println!("Sun never sets (midnight sun)"),
        SolarEventResult::AllNight => println!("Sun never rises (polar night)"),
    }
    println!();

    // Calculate twilight times
    println!("Twilight Times:");

    match calc.get_civil_dawn()? {
        SolarEventResult::Occurs(timestamp) => {
            let dawn_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Civil Dawn: {}",
                dawn_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Civil Dawn: N/A"),
    }

    match calc.get_civil_dusk()? {
        SolarEventResult::Occurs(timestamp) => {
            let dusk_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Civil Dusk: {}",
                dusk_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Civil Dusk: N/A"),
    }

    match calc.get_nautical_dawn()? {
        SolarEventResult::Occurs(timestamp) => {
            let dawn_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Nautical Dawn: {}",
                dawn_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Nautical Dawn: N/A"),
    }

    match calc.get_nautical_dusk()? {
        SolarEventResult::Occurs(timestamp) => {
            let dusk_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Nautical Dusk: {}",
                dusk_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Nautical Dusk: N/A"),
    }

    match calc.get_astronomical_dawn()? {
        SolarEventResult::Occurs(timestamp) => {
            let dawn_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Astronomical Dawn: {}",
                dawn_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Astronomical Dawn: N/A"),
    }

    match calc.get_astronomical_dusk()? {
        SolarEventResult::Occurs(timestamp) => {
            let dusk_time = Utc.timestamp_opt(timestamp, 0).unwrap();
            println!(
                "  Astronomical Dusk: {}",
                dusk_time
                    .with_timezone(&chrono_tz::America::New_York)
                    .format("%H:%M:%S EST")
            );
        }
        _ => println!("  Astronomical Dusk: N/A"),
    }
    println!();

    // Calculate solar transit (solar noon)
    let transit_timestamp = calc.get_solar_transit()?;
    let solar_noon = Utc.timestamp_opt(transit_timestamp, 0).unwrap();
    println!(
        "Solar Noon: {}",
        solar_noon
            .with_timezone(&chrono_tz::America::New_York)
            .format("%H:%M:%S EST")
    );

    // Calculate solar midnight (next one)
    let next_midnight_timestamp = calc.get_next_solar_midnight()?;
    let solar_midnight = Utc.timestamp_opt(next_midnight_timestamp, 0).unwrap();
    println!(
        "Next Solar Midnight: {}",
        solar_midnight
            .with_timezone(&chrono_tz::America::New_York)
            .format("%H:%M:%S EST")
    );
    println!();
    Ok(())
}
