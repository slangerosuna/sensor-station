use anyhow::Result;
use axum::Router;
use rustls::{ClientConfig, RootCertStore, pki_types::CertificateDer};
use rustls_native_certs::load_native_certs;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio_postgres_rustls::MakeRustlsConnect;

mod api;
mod config;
mod sensor;

use api::ApiState;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    println!("Connecting to database at {}...", config.database_url);

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to install rustls crypto provider"))?;

    let mut root_store = RootCertStore::empty();

    let certs = load_native_certs();
    for cert in certs.certs {
        let der: CertificateDer<'static> = cert;
        _ = root_store.add(der);
    }

    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let tls = MakeRustlsConnect::new(tls_config);

    let (client, connection) = tokio_postgres::connect(&config.database_url, tls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {e}");
        }
    });

    client.batch_execute(include_str!("setup.sql")).await?;

    let client = Arc::new(client);
    let most_recent_image = vec![0.0f32; 768];
    let most_recent_image = Arc::new(Mutex::new(most_recent_image));

    let state = ApiState::new(client.clone(), most_recent_image.clone());

    let static_files = tower_http::services::ServeDir::new(&config.static_files_dir)
        .append_index_html_on_directories(true);

    let router = Router::new()
        .nest("/api", api::routes())
        .fallback_service(static_files)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    println!("Listening on http://{}", config.addr);

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let shutdown_signal = async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        tx.send(()).ok();

        println!("Shutdown signal received, exiting...");
    };

    tokio::join!(
        axum::serve(listener, router).with_graceful_shutdown(shutdown_signal),
        sensor::handle_sensor_data(client, most_recent_image, rx),
    );

    Ok(())
}
