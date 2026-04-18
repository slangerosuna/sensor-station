CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS stations (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name VARCHAR(255) NOT NULL UNIQUE,

  water_depth REAL NOT NULL DEFAULT 0.0,

  water_bitmap BIT(768) NOT NULL DEFAULT B'0'::bit(768),
  land_bitmap BIT(768) NOT NULL DEFAULT B'0'::bit(768),

  last_updated TIMESTAMP NOT NULL DEFAULT now(),
  boot_timestamp TIMESTAMP NOT NULL DEFAULT now(),

  location GEOGRAPHY(POINT, 4326) NOT NULL DEFAULT ST_SetSRID(ST_MakePoint(0, 0), 4326)::geography
);

-- seeds this because, for now, since it only reads from one serial port, it only has the one station
INSERT INTO stations (name)
SELECT 'default_station'
WHERE NOT EXISTS (
  SELECT 1 FROM stations
);

CREATE TABLE IF NOT EXISTS recordings (
  station_id INTEGER NOT NULL REFERENCES stations(id) ON DELETE CASCADE,
  -- ppm
  co2 REAL NOT NULL,

  -- hPa
  pressure REAL NOT NULL,
  -- %RH
  humidity REAL NOT NULL,

  -- degrees C
  air_temperature REAL NOT NULL,
  water_temperature REAL NOT NULL,
  ground_temperature REAL NOT NULL,

  -- m/s
  wind_speed REAL NOT NULL,

  -- W/m^2
  net_irradiance REAL NOT NULL,

  -- kg m^-2 s^-1
  rate_of_evaporation REAL NOT NULL,

  timestamp TIMESTAMP NOT NULL DEFAULT now()
);

