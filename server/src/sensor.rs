use anyhow::Result;
use std::sync::Arc;
use tokio_postgres::Client;

pub async fn handle_sensor_data(
    db: Arc<Client>,
    rx: tokio::sync::oneshot::Receiver<()>,
    sensor_uri: String,
) -> anyhow::Result<()> {

    todo!()
}
