use std::net::Ipv6Addr;

use anyhow::Result;
use axum::{Router, response::IntoResponse, routing::get};
use tokio::net::TcpListener;
use tokio_shutdown::Shutdown;
use tracing::{Level, info};
use tracing_subscriber::{filter::Targets, prelude::*};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Setup logging so we can see the shutdown event happening.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            Targets::new()
                .with_target("tokio_shutdown", Level::TRACE)
                .with_default(Level::INFO),
        )
        .init();

    // Create a new shutdown handle, that can be cheaply cloned and shared across threads.
    let shutdown = Shutdown::new()?;

    // Create an Axum handler that returns plain text on the root path.
    let app = Router::new().route("/", get(index)).with_state(());

    // Create the server with a random port.
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).await?;

    // Print out process ID and listening address.
    // The ID is important to signal the server with a shutdown like `kill -s SIGINT <pid>`.
    // Try connecting to the server with `curl <address>`.
    info!("process ID is {}", std::process::id());
    info!("listening on {}", listener.local_addr()?);

    // Pass a shutown handle so the server stops receiving new requests and handles ongoing ones
    // nicely, once the signal is received.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown.handle())
        .await?;

    Ok(())
}

// A simple hello world response.
async fn index() -> impl IntoResponse {
    "Hello, World!"
}
