use crate::runners::wgpu_runner::levenshtein_gpu;
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use levenshtein_with_gpu::{levenshtein_distance_cpu, save_to_csv, LevenshteinGPU};
mod cli;
pub mod runners;
pub mod draw_graph;

pub const WORDS_PADDING: usize = 32;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let _ = draw_graph::plot_benchmarks();
    let cli = Cli::parse();

    match cli.command {
        Commands::Compute {
            words: args,
            gpu,
            verbose,
            file,
            csv,
        } => {
            let start = std::time::Instant::now();
            let content;
            let words: Vec<&str> = if let Some(file_path) = file {
                content = std::fs::read_to_string(file_path)?;
                content.split_whitespace().collect() 
            } else {
                args.iter().map(|s| s.as_str()).collect()
            };
            let size = words.len();

            let distances = if gpu {
                println!("{}", "🚀 Using GPU acceleration".bright_green());
                let gpu = LevenshteinGPU::new(size).await;
                levenshtein_gpu(&gpu, &words).await
            } else {
                println!("{}", "🐢 Using CPU implementation".yellow());
                levenshtein_distance_cpu(&words)
            };

            if csv {
                save_to_csv(&words, &distances);
            } else {
                print_matrix(&words, &distances);
            }

            if verbose {
                println!("⏱️  Time: {:?}", start.elapsed());
            }
        }

        // Commands::Bench { size } => {
        //     println!("{}", "📊 Running benchmarks...".bright_blue());
        //     match size {
        //         small => todo!(),
        //         medium=> todo!(),
        //         large => todo!(),
        //     }
        // }
    }
    Ok(())
}

fn print_matrix(words: &[&str], distances: &[u32]) {
    println!("\n{}", "Distance Matrix:".bright_cyan().underline());
    print!("{:>8}", "");
    for word in words {
        print!("{:>8}", word);
    }
    println!();

    for (i, word) in words.iter().enumerate() {
        print!("{:>8}", word);
        for j in 0..words.len() {
            print!("{:>8}", distances[i * words.len() + j]);
        }
        println!();
    }
}
