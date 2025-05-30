use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "levenshtein")]
#[command(about = "🦀 Fast Levenshtein distances with CPU/GPU", long_about = None)]
#[command(color = clap::ColorChoice::Always)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compute distances between words
    Compute {
        /// Words to compare (space-separated)
        #[arg(required = true)]
        words: Vec<String>,

        /// Input file (alternative to CLI words)
        #[arg(short, long, conflicts_with = "words")]
        file: Option<PathBuf>,

        /// Use GPU acceleration
        #[arg(short, long)]
        gpu: bool,

        /// Show detailed timing
        #[arg(short, long)]
        verbose: bool,
    },

    /// Benchmark CPU vs GPU
    Bench {
        /// Test cases: "small", "medium", or "large"
        #[arg(default_value = "small")]
        size: String,
    },
}
