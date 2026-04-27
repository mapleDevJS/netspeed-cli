use clap::Parser;
use netspeed_cli::bin_errors::{
    exit_codes, is_config_error, is_list_sentinel, is_network_error, machine_error_format,
    print_error,
};
use netspeed_cli::cli::{Args, OutputFormatType};
use netspeed_cli::orchestrator::Orchestrator;

fn main() {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = e.print();
            let code = if e.use_stderr() {
                exit_codes::USAGE_ERROR
            } else {
                exit_codes::SUCCESS
            };
            std::process::exit(code);
        }
    };

    if args.no_emoji {
        unsafe {
            std::env::set_var("NO_EMOJI", "1");
        }
    }

    let machine_fmt = machine_error_format(&args);

    let file_config = netspeed_cli::config::load_config_file();
    let orchestrator = match Orchestrator::new(args, file_config) {
        Ok(o) => o,
        Err(e) => {
            print_error(&e, exit_codes::CONFIG_ERROR, machine_fmt);
            std::process::exit(exit_codes::CONFIG_ERROR);
        }
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    let exit_code = rt.block_on(run_speedtest(orchestrator, machine_fmt));
    std::process::exit(exit_code);
}

async fn run_speedtest(
    orchestrator: Orchestrator,
    machine_error_format: Option<OutputFormatType>,
) -> i32 {
    match orchestrator.run().await {
        Ok(()) => exit_codes::SUCCESS,
        Err(ref e) if is_list_sentinel(e) => exit_codes::SUCCESS,
        Err(ref e) if is_network_error(e) => {
            print_error(e, exit_codes::NETWORK_ERROR, machine_error_format);
            exit_codes::NETWORK_ERROR
        }
        Err(ref e) if is_config_error(e) => {
            print_error(e, exit_codes::CONFIG_ERROR, machine_error_format);
            exit_codes::CONFIG_ERROR
        }
        Err(e) => {
            print_error(&e, exit_codes::INTERNAL_ERROR, machine_error_format);
            exit_codes::INTERNAL_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_handles_args_parsing_error() {}

    #[test]
    fn test_run_speedtest_success() {}
}
