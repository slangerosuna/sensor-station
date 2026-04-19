use anyhow::{Result, anyhow};
use chrono::{DateTime, Timelike, Utc};
use prost::Message;
use reqwest::header::{ACCEPT, USER_AGENT};
use std::sync::Arc;
use tokio_postgres::Client;

// Manually defined protobuf message structs matching sensor.proto
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SensorData {
    #[prost(float, tag = "1")]
    pub timestamp: f32,
    #[prost(int32, tag = "2")]
    pub co2: i32,
    #[prost(float, tag = "3")]
    pub bme_temperature: f32,
    #[prost(float, tag = "4")]
    pub bme_pressure: f32,
    #[prost(float, tag = "5")]
    pub bme_altitude: f32,
    #[prost(float, tag = "6")]
    pub bme_humidity: f32,
    #[prost(message, repeated, tag = "7")]
    pub row: ::prost::alloc::vec::Vec<Row>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Row {
    #[prost(float, repeated, tag = "1")]
    pub pixel_temp: ::prost::alloc::vec::Vec<f32>,
}

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

#[derive(serde::Deserialize)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
}

#[derive(serde::Deserialize)]
struct OpenMeteoCurrent {
    wind_speed_10m: f64,
    cloud_cover: f64,
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

async fn fetch_weather_from_open_meteo(latitude: f64, longitude: f64) -> Result<(f64, f64)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}&current=wind_speed_10m,cloud_cover&wind_speed_unit=ms"
    );

    let weather = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "sensor-station/0.1 (hackathon project)")
        .send()
        .await?
        .error_for_status()?
        .json::<OpenMeteoResponse>()
        .await?;

    let wind_speed = weather.current.wind_speed_10m;
    let net_irradiance = estimate_net_irradiance_from_sky_cover(weather.current.cloud_cover, longitude);

    Ok((net_irradiance, wind_speed))
}

// kg/m^2/s
fn penman_equation(
    net_irradiance: f64, // W/m^2
    wind_speed: f64,     // m/s

    air_temperature_celsius: f64,    // degrees C
    water_temperature_celsius: f64,  // degrees C
    ground_temperature_celsius: f64, // degrees C
    pressure_hpa: f64,               // hPa
    water_depth_meters: f64,         // m
    humidity: f64,                   // %RH
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
        saturation_vapor_pressure * 17.27 * 237.3 / f64::powi(air_temperature_celsius + 237.3, 2);

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

    let psychometric_constant = (pressure_kpa * 1000.0 * specific_heat_capacity_dry_air)
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

    let heat_gradient =
        (water_temperature_celsius - deep_ground_temperature_celsius) / water_depth_meters; // K/m
    let ground_heat_flux = -soil_thermal_conductivity * heat_gradient; // W/m^2

    // penman equation
    ((derivative_of_saturation_vapor_pressure_pascals_per_kelvin * (net_irradiance - ground_heat_flux))
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
    sensor_addr: String,
) {
    let mut process_sensor_data = async move || {
        let sensor_addr = sensor_addr.clone();
        let mut f = async move || -> Result<()> {
            use tokio::io::AsyncReadExt;
            use tokio::net::TcpStream;

            println!("Connecting to sensor at {sensor_addr}...");
            let mut stream = TcpStream::connect(&sensor_addr).await?;
            println!("Connected to sensor");

            loop {
                let mut header = [0u8; 4];

                stream.read_exact(&mut header).await?;
                let len = u32::from_be_bytes(header) as usize;

                if len == 0 || len > 1_000_000 {
                    return Err(anyhow!("invalid sensor packet length: {len}"));
                }

                let mut buf = vec![0u8; len];

                stream.read_exact(&mut buf).await?;

                println!("Received sensor data packet of length {len} bytes");

                let sensor_data = SensorData::decode(&buf[..])
                    .map_err(|e| anyhow!("failed to decode protobuf sensor data: {e}"))?;

                let sensor_seconds_since_boot = sensor_data.timestamp;
                let co2 = sensor_data.co2 as f32;
                let air_temperature = sensor_data.bme_temperature;
                let air_pressure = sensor_data.bme_pressure;
                let _estimated_altitude = sensor_data.bme_altitude;
                let humidity = sensor_data.bme_humidity;

                // Flatten the thermal camera rows into a single Vec<f32>
                let thermal_camera_data: Vec<f32> = sensor_data
                    .row
                    .iter()
                    .flat_map(|row| row.pixel_temp.clone())
                    .collect();

                if thermal_camera_data.len() != 768 {
                    return Err(anyhow!(
                        "expected 768 thermal pixels, got {}",
                        thermal_camera_data.len()
                    ));
                }

                let mut guard = most_recent_image.lock().await;

                guard.clone_from(&thermal_camera_data);

                drop(guard);

                let row = db
                    .query_one(
                        "
                    SELECT
                        water_depth,
                        water_bitmap,
                        land_bitmap,
                        ST_Y(location::geometry) as latitude,
                        ST_X(location::geometry) as longitude
                    FROM stations
                    WHERE id = 1
                ",
                        &[],
                    )
                    .await?;

                let water_depth: f32 = row.get(0);
                let water_depth: f64 = water_depth as f64;

                use bit_vec::BitVec;
                let water_bitmap: BitVec = row.get(1);
                let water_bitmap = water_bitmap.to_bytes();

                let land_bitmap: BitVec = row.get(2);
                let land_bitmap = land_bitmap.to_bytes();

                let latitude: f64 = row.get(3);
                let longitude: f64 = row.get(4);

                let timestamp = DateTime::<Utc>::from_timestamp_millis(
                    (sensor_seconds_since_boot as f64 * 1000.0).round() as i64,
                )
                .ok_or_else(|| anyhow!("invalid sensor timestamp: {sensor_seconds_since_boot}"))?
                .naive_utc();

                fn find_average_temperature_over_selected_area(
                    thermal_camera_data: &[f32],
                    bitmap: &[u8],
                ) -> f64 {
                    let (sum, count) = bitmap
                        .iter()
                        .enumerate()
                        .map(|(n, b)| -> (f64, usize) {
                            let mut acc: f64 = 0.0;
                            let mut count: usize = 0;

                            for i in 0..8 {
                                if b & (0b1000_0000 >> i) != 0 {
                                    acc += thermal_camera_data[(n * 8) + i as usize] as f64;
                                    count += 1;
                                }
                            }

                            (acc, count)
                        })
                        .fold((0.0, 0), |(acc1, count1), (acc2, count2)| {
                            (acc1 + acc2, count1 + count2)
                        });

                    let res = sum / count as f64;

                    if res.is_nan() {
                        22.0
                    } else {
                        res
                    }
                }

                let water_temperature = find_average_temperature_over_selected_area(
                    &thermal_camera_data,
                    &water_bitmap,
                );
                let ground_temperature =
                    find_average_temperature_over_selected_area(&thermal_camera_data, &land_bitmap);

                // Pull weather from NWS using station location. If unavailable, keep pipeline alive.
                let (net_irradiance, wind_speed) = match fetch_weather_from_open_meteo(latitude, longitude).await {
                    Ok((irradiance, wind)) => (irradiance, wind),
                    Err(e) => {
                        eprintln!("Open-Meteo weather fetch failed ({e}), using fallback values");

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

                println!("Calculated rate of evaporation: {rate_of_evaporation} kg/m^2/s");

                db.execute(
                    "\
                    INSERT INTO recordings (
                        station_id,
                        timestamp,
                        co2,
                        pressure,
                        humidity,
                        air_temperature,
                        water_temperature,
                        ground_temperature,
                        wind_speed,
                        net_irradiance,
                        rate_of_evaporation
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ",
                    &[
                        &1i32,
                        &timestamp,
                        &co2,
                        &air_pressure,
                        &humidity,
                        &air_temperature,
                        &(water_temperature as f32),
                        &(ground_temperature as f32),
                        &(wind_speed as f32),
                        &(net_irradiance as f32),
                        &(rate_of_evaporation as f32),
                    ],
                )
                .await?;

                println!("Inserted sensor data into database with timestamp {timestamp}");
            }
        };

        println!("Connecting to sensor for first time...");
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
