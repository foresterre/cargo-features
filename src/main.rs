extern crate anyhow;
extern crate cargo_metadata;
extern crate colored;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;

mod discover;
mod powerset;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Features(FeaturesCli),
}

#[derive(clap::Args)]
#[command(author, version, about = "Power tools for working with Cargo features")]
#[command(propagate_version = true)]
struct FeaturesCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover which crates enabled specific conditional compilation features
    Discover {
        /// Show all available features even if not currently enabled
        #[arg(short, long)]
        all: bool,

        /// Path to Cargo.toml (defaults to current directory)
        #[arg(short, long)]
        manifest_path: Option<std::path::PathBuf>,
    },

    /// Run a cargo command for every combination in the feature powerset
    Powerset {
        /// Skip the empty feature set (no features enabled)
        #[arg(long)]
        skip_empty: bool,

        /// Specific features to include in powerset (default: all)
        #[arg(short, long)]
        features: Vec<String>,

        /// Path to Cargo.toml
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,

        /// Cargo command to run (e.g., "build", "test", "check")
        #[arg(required = true, num_args = 1..)]
        cargo_command: Vec<String>,
    },
}

fn main() {
    let CargoCli::Features(cli) = CargoCli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run(cli: FeaturesCli) -> Result<()> {
    match cli.command {
        Commands::Discover { all, manifest_path } => discover::run(!all, manifest_path),
        Commands::Powerset {
            skip_empty,
            features,
            manifest_path,
            cargo_command,
        } => powerset::run(skip_empty, features, manifest_path, cargo_command),
    }
}
