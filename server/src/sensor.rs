use anyhow::{Result, anyhow};
use chrono::{DateTime, Timelike, Utc};
use reqwest::header::{ACCEPT, USER_AGENT};
use std::sync::Arc;
use tokio_postgres::Client;

#[derive(serde::Deserialize)]
struct NwsPointsResponse {
    properties: NwsPointsProperties,
}

#[derive(serde::Deserialize)]
struct NwsPointsProperties {
    #[serde(rename = "forecastGridData")]
    forecast_grid_data: String,
}

#[derive(serde::Deserialize)]
struct NwsGridResponse {
    properties: NwsGridProperties,
}

#[derive(serde::Deserialize)]
struct NwsGridProperties {
    #[serde(rename = "windSpeed")]
    wind_speed: NwsSeries,
    #[serde(rename = "skyCover")]
    sky_cover: NwsSeries,
}

#[derive(serde::Deserialize)]
struct NwsSeries {
    uom: String,
    values: Vec<NwsValue>,
}

#[derive(serde::Deserialize)]
struct NwsValue {
    #[serde(rename = "validTime")]
    valid_time: String,
    value: Option<f64>,
}

fn parse_valid_time_start(valid_time: &str) -> Option<DateTime<Utc>> {
    let start = valid_time.split('/').next()?;
    let parsed = DateTime::parse_from_rfc3339(start).ok()?;
    Some(parsed.with_timezone(&Utc))
}

fn pick_latest_or_current_value(values: &[NwsValue]) -> Option<f64> {
    let now = Utc::now();

    let latest_before_or_at_now = values
        .iter()
        .filter_map(|entry| {
            let value = entry.value?;
            let start = parse_valid_time_start(&entry.valid_time)?;
            if start <= now {
                Some((start, value))
            } else {
                None
            }
        })
        .max_by_key(|(start, _)| *start)
        .map(|(_, value)| value);

    latest_before_or_at_now.or_else(|| values.iter().find_map(|entry| entry.value))
}

fn convert_wind_speed_to_mps(value: f64, uom: &str) -> f64 {
    if uom.contains("km_h-1") {
        value / 3.6
    } else if uom.contains("m_s-1") {
        value
    } else if uom.contains("kn") {
        value * 0.514_444
    } else {
        value
    }
}

fn estimate_net_irradiance_from_sky_cover(sky_cover_percent: f64, longitude: f64) -> f64 {
    let utc_now = Utc::now();
    let approx_tz_offset_hours = (longitude / 15.0).round() as i64;
    let local_hour = ((utc_now.hour() as i64 + approx_tz_offset_hours).rem_euclid(24)) as f64;

    let daylight_factor = if (6.0..=18.0).contains(&local_hour) {
        let x = (local_hour - 6.0) / 12.0;
        (std::f64::consts::PI * x).sin().max(0.0)
    } else {
        0.0
    };

    let normalized_sky_cover = (sky_cover_percent / 100.0).clamp(0.0, 1.0);
    let cloud_factor = (1.0 - 0.75 * normalized_sky_cover.powi(3)).clamp(0.15, 1.0);

    1000.0 * daylight_factor * cloud_factor
}

async fn fetch_weather_from_nws(latitude: f64, longitude: f64) -> Result<(f64, f64)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let points_url = format!("https://api.weather.gov/points/{latitude},{longitude}");

    let points = client
        .get(points_url)
        .header(ACCEPT, "application/geo+json")
        .header(USER_AGENT, "sensor-station/0.1 (hackathon project)")
        .send()
        .await?
        .error_for_status()?
        .json::<NwsPointsResponse>()
        .await?;

    let grid = client
        .get(points.properties.forecast_grid_data)
        .header(ACCEPT, "application/geo+json")
        .header(USER_AGENT, "sensor-station/0.1 (hackathon project)")
        .send()
        .await?
        .error_for_status()?
        .json::<NwsGridResponse>()
        .await?;

    let wind_raw = pick_latest_or_current_value(&grid.properties.wind_speed.values)
        .ok_or_else(|| anyhow!("NWS grid response missing windSpeed values"))?;
    let wind_speed_mps = convert_wind_speed_to_mps(wind_raw, &grid.properties.wind_speed.uom);

    let sky_cover = pick_latest_or_current_value(&grid.properties.sky_cover.values)
        .ok_or_else(|| anyhow!("NWS grid response missing skyCover values"))?;
    let net_irradiance = estimate_net_irradiance_from_sky_cover(sky_cover, longitude);

    Ok((net_irradiance, wind_speed_mps))
}

// kg/m^2/s
fn penman_equation(
    net_irradiance: f64,            // W/m^2
    wind_speed: f64,                // m/s

    air_temperature_celsius: f64,   // degrees C
    water_temperature_celsius: f64, // degrees C
    ground_temperature_celsius: f64,// degrees C
    pressure_hpa: f64,              // hPa
    water_depth_meters: f64,        // m
    humidity: f64,               // %RH
) -> f64 {
    let pressure_kpa = pressure_hpa / 10.0;
    let air_temperature_kelvin = air_temperature_celsius + 273.15;

    // kPa
    // Tetens equation
    let saturation_vapor_pressure =
        0.61078 * f64::exp(17.27 * air_temperature_celsius / (air_temperature_celsius + 237.3));

    // kPa/K
    // d/dx Tetens equation
    let derivative_of_saturation_vapor_pressure = 
        saturation_vapor_pressure * 17.27 * 237.3 
        / f64::powi(air_temperature_celsius + 237.3, 2);

    let derivative_of_saturation_vapor_pressure_pascals_per_kelvin =
        derivative_of_saturation_vapor_pressure * 1000.0;

    let partial_pressure_water_vapor = saturation_vapor_pressure * humidity / 100.0;
    let partial_pressure_dry_air = pressure_kpa - partial_pressure_water_vapor;

    let specific_gas_constant_dry_air = 287.05; // J/(kg*K)
    let specific_gas_constant_for_water_vapor = 461.52; // J/(kg*K)

    // kg/m^3
    let mass_dry_air = partial_pressure_dry_air * 1000.0
        / (specific_gas_constant_dry_air * air_temperature_kelvin);
    let mass_water_vapor = partial_pressure_water_vapor * 1000.0
        / (specific_gas_constant_for_water_vapor * air_temperature_kelvin);

    let density_of_air = mass_dry_air + mass_water_vapor;

    let humidity_ratio = mass_water_vapor / mass_dry_air;

    let specific_heat_capacity_dry_air = 1005.0; // J/(kg*K)
    let specific_heat_capacity_water_vapor = 1860.0; // J/(kg*K)

    // J/(kg*K)
    let specific_heat_capacity_of_air = (specific_heat_capacity_dry_air
        + specific_heat_capacity_water_vapor * humidity_ratio)
        / (1.0 + humidity_ratio);

    // kPa
    let vapor_pressure_deficit = saturation_vapor_pressure - partial_pressure_water_vapor;
    let vapor_pressure_deficit_pascals = vapor_pressure_deficit * 1000.0;

    // assumed to be constant for simplicity because I'm tired and this is just a hackathon
    let latent_heat_of_vaporization = 2.45e6; // J/kg

    let psychometric_constant =
        (pressure_kpa * 1000.0 * specific_heat_capacity_dry_air)
        / (latent_heat_of_vaporization * 0.622);

    // empirical formula I found online but forgot to write down the link
    let momentum_surface_aerodynamic_conductance = wind_speed * (7.5e-4 + 6.7e-5 * wind_speed);

    // normally this would be measured, but we don't have that sensor, so I looked through
    // wikipedia and did a bunch of math and came up with the following to estimate it, but
    // it's probably not that great
    let soil_thermal_conductivity = 2.0; // (W/m*K) - guess for saturated soil

    // rough estimate from data from this paper: https://doi.org/10.1016/j.agwat.2025.109571
    // assumes somewhere roughly between 10cm and 3m depth; I don't have data for deeper
    let deep_ground_temperature_celsius = ground_temperature_celsius - 1.5;

    let heat_gradient = (water_temperature_celsius - ground_temperature_celsius) / water_depth_meters; // K/m
    let ground_heat_flux = -soil_thermal_conductivity * heat_gradient; // W/m^2

    // penman equation
    ((derivative_of_saturation_vapor_pressure_pascals_per_kelvin * net_irradiance)
        + (density_of_air
            * specific_heat_capacity_of_air
            * vapor_pressure_deficit_pascals
            * momentum_surface_aerodynamic_conductance))
        / (latent_heat_of_vaporization
            * (derivative_of_saturation_vapor_pressure_pascals_per_kelvin + psychometric_constant))
}

use tokio::sync::Mutex;

pub async fn handle_sensor_data(
    db: Arc<Client>,
    most_recent_image: Arc<Mutex<Vec<f32>>>,
    rx: tokio::sync::oneshot::Receiver<()>,
) {
    let mut process_sensor_data = async move || {
        let mut f = async move || -> Result<()> {
            use tokio::net::TcpStream;
            use tokio::io::AsyncReadExt;

            println!("Connecting to sensor at host.docker.internal:12347...");
            let mut stream = TcpStream::connect("127.17.0.1:12347").await?;
            println!("Connected to sensor at host.docker.internal:12347");

            loop {
                let mut header = [0u8; 4];

                stream.read_exact(&mut header).await?;
                let len = u32::from_le_bytes(header) as usize;
                let mut buf = vec![0u8; len - 4];

                stream.read_exact(&mut buf).await?;

                println!("Received sensor data packet of length {len} bytes");

                let timestamp = f32::from_le_bytes(buf[0..4].try_into().unwrap());

                let co2 = f32::from_le_bytes(buf[4..8].try_into().unwrap());
                let air_temperature = f32::from_le_bytes(buf[8..12].try_into().unwrap());
                let air_pressure = f32::from_le_bytes(buf[12..16].try_into().unwrap());
                let _estimated_altitude = f32::from_le_bytes(buf[16..20].try_into().unwrap());
                let humidity = f32::from_le_bytes(buf[20..24].try_into().unwrap());

                let thermal_camera_data: Vec<f32> = buf[24..]
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();

                let mut guard = most_recent_image.lock().await;

                guard.clone_from(&thermal_camera_data);

                drop(guard);

                let row = db.query_one("
                    SELECT
                        water_depth,
                        water_bitmap,
                        land_bitmap,
                        ST_Y(location::geometry) as latitude,
                        ST_X(location::geometry) as longitude,
                        boot_timestamp
                    FROM stations
                    WHERE id = 1
                ", &[]).await?;

                let water_depth: f64 = row.get(0);

                use bit_vec::BitVec;
                let water_bitmap: BitVec = row.get(1);
                let water_bitmap = water_bitmap.to_bytes();

                let land_bitmap: BitVec = row.get(2);
                let land_bitmap = land_bitmap.to_bytes();

                let latitude: f64 = row.get(3);
                let longitude: f64 = row.get(4);

                let boot_timestamp: f32 = row.get(5);
                let timestamp = boot_timestamp + timestamp;

                fn find_average_temperature_over_selected_area(
                    thermal_camera_data: &[f32],
                    bitmap: &[u8],
                ) -> f64 {
                    let (sum, count) = bitmap.iter().enumerate().map(|(n, b)| -> (f64, usize) {
                        let mut acc: f64 = 0.0;
                        let mut count: usize = 0;

                        for i in 0..8 {
                            if b & (0b1000_0000 >> i) != 0 {
                                acc += thermal_camera_data[(n * 8) + i as usize] as f64;
                                count += 1;
                            }
                        }

                        (acc, count)
                    }).fold((0.0, 0), |(acc1, count1), (acc2, count2)| (acc1 + acc2, count1 + count2));

                    sum / count as f64
                }

                let water_temperature = find_average_temperature_over_selected_area(&thermal_camera_data, &water_bitmap);
                let ground_temperature = find_average_temperature_over_selected_area(&thermal_camera_data, &land_bitmap);

                // Pull weather from NWS using station location. If unavailable, keep pipeline alive.
                let (net_irradiance, wind_speed) = match fetch_weather_from_nws(latitude, longitude).await {
                    Ok((irradiance, wind)) => (irradiance, wind),
                    Err(err) => {
                        eprintln!("NWS API fetch failed, using fallback values: {err}");
                        (800.0, 5.0)
                    }
                };

                let rate_of_evaporation = penman_equation(
                    net_irradiance,
                    wind_speed,

                    air_temperature as f64,
                    water_temperature,
                    ground_temperature,
                    air_pressure as f64,
                    water_depth as f64,
                    humidity as f64,
                );

                db.execute("
                    INSERT INTO sensor_data (
                        timestamp,
                        co2,
                        air_temperature,
                        pressure,
                        humidity,
                        water_temperature,
                        ground_temperature,
                        net_irradiance,
                        rate_of_evaporation,
                        location
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, ST_SetSRID(ST_MakePoint($10, $11), 4326))
                ", &[
                    &timestamp,
                    &co2,
                    &air_temperature,
                    &air_pressure,
                    &humidity,
                    &water_temperature,
                    &ground_temperature,
                    &net_irradiance,
                    &rate_of_evaporation,
                ]).await?;

                println!("Inserted sensor data into database with timestamp {timestamp}");
            }
        };

        while let Err(e) = f().await {
            eprintln!("Error processing sensor data: {e}");
            println!("Attempting to reconnect to sensor in 5 seconds...");

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            println!("Reconnecting to sensor...");
        }
    };

    tokio::select! {
        _ = rx => { },
        _ = process_sensor_data() => { },
    };
}
