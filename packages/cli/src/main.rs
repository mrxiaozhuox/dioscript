use std::time::Instant;

use clap::{Args, Parser, Subcommand};
use colored::*;

mod builder;

#[derive(Parser)]
#[command(name = "ds")]
#[command(author = "YuKun Liu <mrxzx.info@gmail.com>")]
#[command(version = "0.1.0")]
struct Dsc {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build(BuildArgs),
}

#[derive(Args)]
struct BuildArgs {
    /// `.ds` file path
    file: String,

    /// build target
    #[arg(long, default_value = "static")]
    target: String,

    /// output directory
    #[arg(long, default_value = ".")]
    out_dir: String,

    /// use browser open `html` entrence file
    #[arg(long, default_value_t = false)]
    open: bool,

    /// use quiet mode
    #[arg(long, default_value_t = false)]
    quiet: bool,
}

pub fn main() {
    let cli = Dsc::parse();
    match &cli.command {
        Commands::Build(args) => {
            let timer = Instant::now();
            let r = builder::build(&args.file, &args.target, &args.out_dir);
            let duration = timer.elapsed();
            match r {
                Err(e) => {
                    println!("[ds] Build failed: {}", e.to_string().red().bold());
                    std::process::exit(1);
                }
                Ok(v) => {
                    if args.open {
                        let _ = opener::open(&v);
                    }
                    if !args.quiet {
                        println!();
                        println!(
                            "📕 {} {}",
                            "HTML File: ".green().bold(),
                            v.purple().italic()
                        );
                        println!(
                            "💎 {} {}",
                            "Build Target: ".blue().bold(),
                            args.target.cyan().italic()
                        );
                        println!(
                            "⌛️ {} {}",
                            "Build Time: ".purple().bold(),
                            format!("{:?}", duration).green().italic()
                        );
                    }
                }
            }
        }
    }
}
