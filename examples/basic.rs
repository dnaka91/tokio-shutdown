use anyhow::Result;
use axum::{response::IntoResponse, routing::get, Router, Server};
use tokio_shutdown::Shutdown;
use tracing::{info, Level};
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
    let app = Router::new().route("/", get(index));

    // Create the server with a random port.
    let server = Server::bind(&([127, 0, 0, 1], 0).into()).serve(app.into_make_service());

    // Print out process ID and listening address.
    // The ID is important to signal the server with a shutdown like `kill -s SIGINT <pid>`.
    // Try connecting to the server with `curl <address>`.
    info!("process ID is {}", std::process::id());
    info!("listening on {}", server.local_addr());

    // Pass a shutown handle so the server stops receiving new requests and handles ongoing ones
    // nicely, once the signal is received.
    server.with_graceful_shutdown(shutdown.handle()).await?;

    Ok(())
}

// A simple hello world response.
async fn index() -> impl IntoResponse {
    "Hello, World!"
}
