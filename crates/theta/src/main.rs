use std::process::ExitCode;

use owo_colors::OwoColorize;
use tracing_subscriber::EnvFilter;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    match theta::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            anstream::eprintln!("{} {}", "error".red().bold(), err);
            for cause in err.chain().skip(1) {
                anstream::eprintln!("  {} {}", "caused by".dimmed(), cause);
            }
            ExitCode::FAILURE
        }
    }
}
