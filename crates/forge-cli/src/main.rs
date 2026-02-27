//! Forge CLI — invoke from tests by running the binary and asserting on output.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "Forge — full-stack web framework for Rust")]
struct Args {
    #[arg(long)]
    version: bool,
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();
    if args.version {
        println!("forge {}", env!("CARGO_PKG_VERSION"));
        return std::process::ExitCode::SUCCESS;
    }
    // Default: show help or subcommands (TBD)
    std::process::ExitCode::SUCCESS
}
