use clap::Parser;
use netspeed_cli::progress::no_color;
use netspeed_cli::{CliArgs, SpeedTestOrchestrator, SpeedtestError};
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() {
    if let Err(e) = run_speedtest().await {
        // __list_displayed__ is a sentinel meaning "—list was shown, exit normally"
        if matches!(&e, SpeedtestError::Context { msg, .. } if msg == "__list_displayed__") {
            return;
        }

        let nc = no_color();
        if nc {
            eprintln!("\nError: {e}");
            eprintln!("For more information, run: netspeed-cli --help");
        } else {
            eprintln!("\n{}", format!("Error: {e}").red().bold());
            eprintln!(
                "{}",
                "For more information, run: netspeed-cli --help".bright_black()
            );
        }
        std::process::exit(1);
    }
}

async fn run_speedtest() -> Result<(), SpeedtestError> {
    let args = CliArgs::parse();
    let orchestrator = SpeedTestOrchestrator::new(args)?;
    orchestrator.run().await
}
