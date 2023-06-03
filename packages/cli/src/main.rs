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
}

pub fn main() {
    let cli = Dsc::parse();
    match &cli.command {
        Commands::Build(args) => {
            let r = builder::build(&args.file, &args.target, &args.out_dir);
            if let Err(e) = r {
                println!("{}", e.to_string().red().bold());
            } else {
                println!("{}", "Build finished.".blue().bold());
            }
        }
    }
}
