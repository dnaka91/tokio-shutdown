use std::process::ExitCode;

use tokio_shutdown::Shutdown;

fn main() -> ExitCode {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let shutdown = Shutdown::new().unwrap();
            shutdown.handle().await;
        });

    ExitCode::from(15)
}
