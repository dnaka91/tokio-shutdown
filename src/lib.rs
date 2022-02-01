//! # Tokio Shutdown
//!
//! Tiny crate that allows to wait for a stop signal across multiple threads. Helpful mostly in
//! server applications that run indefinitely and need a signal for graceful shutdowns.
//!
//! # Examples
//!
//! This example installs the global shutdown handler and will print a message once a single is
//! received. For demonstration purposes it creates other tasks that wait for shutdown as well.
//!
//! ```no_run
//! use tokio_shutdown::Shutdown;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let shutdown = Shutdown::new().expect("shutdown creation works on first call");
//!
//!     // Pass a copy of the shutdown handler to another task.
//!     // Clones of `Shutdown` are cheap.
//!     let clone = shutdown.clone();
//!     tokio::spawn(async move {
//!         clone.handle().await;
//!         println!("task 1 shutting down");
//!     });
//!
//!     // Alternatively, pass a new handle to the new task.
//!     // Both this and the above way work, choose whatever works best for your use case.
//!     let handle = shutdown.handle();
//!     tokio::spawn(async move {
//!         handle.await;
//!         println!("task 2 shutting down");
//!     });
//!
//!     shutdown.handle().await;
//!     println!("application shutting down");
//! }
//! ```
//!
//! Please have a look at the examples directory of this project for further usage instructions.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]

use std::{
    error::Error,
    fmt::{self, Display},
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use tokio::{signal, sync::watch};

/// Error that occurs when the [`Shutdown::new`] function is called more than once in a process
/// lifetime.
#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyCreatedError;

impl Error for AlreadyCreatedError {}

impl Display for AlreadyCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("shutdown handler already created")
    }
}

static CREATED: AtomicBool = AtomicBool::new(false);

/// The global shutdown handler for an application. It can be cloned cheaply wherever needed.
///
/// New handles can be created with the [`handle`](Self::handle) function, which creates futures
/// that will complete once a shutdown signal is received.
#[derive(Clone)]
pub struct Shutdown {
    receiver: watch::Receiver<()>,
}

impl Shutdown {
    /// Create a new shutdown handle. This can only be called once per application instance.
    ///
    /// Signal handles can only be registered once for the duration of the entire process and
    /// creating another shutdown handler would break the previous one without notice.
    ///
    /// # Errors
    ///
    /// If this function is called more than once during the lifetime of a process, an error will be
    /// returned.
    pub fn new() -> Result<Shutdown, AlreadyCreatedError> {
        if (CREATED).swap(true, Ordering::SeqCst) {
            return Err(AlreadyCreatedError);
        }

        let (tx, rx) = watch::channel(());
        let handle = register_handlers();

        tokio::spawn(async move {
            handle.await;
            tx.send(()).ok();
        });

        Ok(Self { receiver: rx })
    }

    /// Create a new handle that can be awaited on. The future will complete once a shutdown signal
    /// is received.
    pub fn handle(&self) -> impl Future<Output = ()> {
        let mut rx = self.receiver.clone();

        async move {
            rx.changed().await.ok();
        }
    }
}

fn register_handlers() -> impl Future<Output = ()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    async {
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        #[cfg(feature = "tracing")]
        tracing::info!("shutdown signal received");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fail_create_two_instances() {
        assert!(Shutdown::new().is_ok());
        assert_eq!(Some(AlreadyCreatedError), Shutdown::new().err());
    }
}
