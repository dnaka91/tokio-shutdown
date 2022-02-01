use tokio_shutdown::Shutdown;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let shutdown = Shutdown::new().unwrap();
            shutdown.handle().await;
        });

    std::process::exit(15);
}
