use anyhow::Result;
use std::net::SocketAddr;

pub struct Config {
    pub database_url: String,
    pub addr: SocketAddr,
    pub static_files_dir: String,
    pub sensor_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;

        let server_host = std::env::var("SERVER_HOST")?;
        let server_port = std::env::var("SERVER_PORT")?;

        let addr = format!("{}:{}", server_host, server_port).parse()?;
        let addr = SocketAddr::V4(addr);

        let static_files_dir = std::env::var("STATIC_FILES_DIR")?;
        let sensor_host = std::env::var("SENSOR_HOST")?;
        let sensor_port = std::env::var("SENSOR_PORT")?;
        let sensor_addr = format!("{}:{}", sensor_host, sensor_port);

        Ok(Self {
            database_url,
            addr,
            static_files_dir,
            sensor_addr,
        })
    }
}
