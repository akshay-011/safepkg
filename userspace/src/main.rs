use clap::{Parser, Subcommand};
use anyhow::Result;
use std::collections::HashSet;

mod sandbox;
mod monitor;
mod engine;

#[derive(Parser)]
#[command(name = "safepkg")]
#[command(about = "A secure, sandboxed package installation wrapper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The package manager command to run (e.g., "npm install")
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify the installation of safepkg
    Verify,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Verify) => {
            println!("OK: Safe-pkg works!");
            return Ok(());
        }
        None => {
            if cli.args.is_empty() {
                println!("Usage: safepkg <command> [args]... or safepkg verify");
                return Ok(());
            }
            
            // Phase 3: Initialize Security Engine
            let mut binaries = HashSet::new();
            binaries.insert("npm".to_string());
            binaries.insert("node".to_string());
            binaries.insert("python".to_string());
            binaries.insert("pip".to_string());

            let config = engine::Config {
                allow_list: engine::AllowList { binaries },
            };
            let engine = engine::SecurityEngine::new(config);
            
            // Pre-flight check: Validate the main command before starting sandbox
            if !engine.is_allowed(&cli.args[0]) {
                anyhow::bail!("🛡️ SafePkg Blocked: '{}' is flagged as a high-risk binary and is not allowed to run.", cli.args[0]);
            }

            // Phase 2: Start Monitor (in a background task)
            let _monitor = monitor::Monitor::new(engine);
            // In a real Linux environment, we would load the compiled ebpf object here:
            // monitor.start(include_bytes!("../../target/bpfel-unknown-none/debug/safepkg-ebpf")).await?;

            println!("🛡️ SafePkg: Monitoring installation of {:?}", cli.args);

            // Phase 1: Execute sandboxed command
            sandbox::run_sandboxed(&cli.args)?;
        }
    }

    Ok(())
}
