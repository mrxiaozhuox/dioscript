use clap::{Args, Parser, Subcommand};

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
    file: String,
    #[arg(long)]
    target: Option<String>,
}

pub fn main() {
    let cli = Dsc::parse();
    match &cli.command {
        Commands::Build(args) => {
            let target = args.target.clone();
            let file_name = args.file.clone();
            builder::build(file_name, target);
        }
    }
}
