use std::{convert::Infallible, net::Ipv6Addr, time::Duration};

use anyhow::Result;
use axum::{
    Router,
    extract::State,
    response::{Sse, sse::Event},
    routing::get,
};
use futures_util::{Stream, StreamExt};
use tokio::{net::TcpListener, time};
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

    // Create an Axum handler that creates an SSE stream on the root path.
    let app = Router::new()
        .route("/", get(sse))
        .with_state(shutdown.clone());

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

// Simple SSE endpoint that will continuously send items to the client until the client disconnects
// by itself or the server receives a shutdown signal.
//
// The client connection will be handled gracefully, even in case of a shutdown. Once it's
// initiated, the stream simply ends and the connection to the client is closed.
pub async fn sse(
    State(shutdown): State<Shutdown>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut counter = 0;

        loop {
            // Increase the counter, sleep for a second and send a new event.
            // We sleep here to not send to many events at once.
            counter += 1;
            time::sleep(Duration::from_secs(1)).await;
            yield Ok(Event::default().data(format!("event {counter}")));
        }
    };

    // The `take_until` make sure that the stream won't emit any new items once the given
    // `Future` completes. Once a shutdown signal is received, the stream is stopped.
    Sse::new(stream.take_until(shutdown.handle()))
}
