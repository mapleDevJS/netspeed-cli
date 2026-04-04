#[tokio::main]
async fn main() {
    if let Err(e) = netspeed_cli::run_speedtest().await {
        tracing::error!("{}", e);
        std::process::exit(1);
    }
}
