use std::{fs::File, io::Write, path::PathBuf, time::Instant};

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
    Playground(PlaygroundArgs),
}

#[derive(Args)]
pub struct BuildArgs {
    /// `.ds` file path
    file: String,

    /// build target
    #[arg(long, default_value = "static")]
    target: String,

    /// html template for build
    #[arg(long)]
    template: Option<String>,

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

#[derive(Args)]
pub struct PlaygroundArgs {}

pub fn main() {
    let cli = Dsc::parse();
    match &cli.command {
        Commands::Build(args) => {
            let timer = Instant::now();
            let r = builder::build(&args);
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
        Commands::Playground(_args) => {
            println!("\n{}", "Welcome to `Dioscript` playground!".blue().bold());
            println!(
                "{}",
                "Use `.execute` command to execute input code.\n"
                    .green()
                    .bold()
            );
            let mut record = String::new();
            let mut code_buffer: Vec<_> = Vec::new();
            let mut readline = rustyline::DefaultEditor::new().expect("init stdin failed.");
            loop {
                let input = readline.readline(">> ").unwrap();
                if input == ".execute" || input == "." {
                    let code = code_buffer.join("\n");
                    let ast = dioscript_parser::ast::DioscriptAst::from_string(&code);
                    match ast {
                        Ok(ast) => {
                            let mut runtime = dioscript_runtime::Runtime::new();
                            let _ = runtime
                                .add_function(
                                    "to_html",
                                    dioscript_runtime::function::element_to_html(),
                                )
                                .unwrap();
                            let result = runtime.execute_ast(ast);
                            match result {
                                Ok(r) => {
                                    println!("\n[ds] Result: {:#?}\n", r);
                                }
                                Err(e) => {
                                    println!(
                                        "\n[ds] Runtime error: {}\n",
                                        e.to_string().red().bold()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("\n[ds] Parse failed: {}\n", e.to_string().red().bold());
                        }
                    }
                    record = code;
                    code_buffer = Vec::new();
                } else if input == ".undo" || input == ".u" {
                    if code_buffer.len() > 0 {
                        println!("\nUndo code input: {}\n", code_buffer.last().unwrap());
                        code_buffer.remove(code_buffer.len() - 1);
                    }
                } else if input == ".clear" || input == ".c" {
                    code_buffer.clear();
                    println!(
                        "\n🚀 {}\n",
                        "deleted all recorded code line.".yellow().bold()
                    );
                } else if input == ".save" || input == ".s" {
                    let file = PathBuf::from("./playground.ds");
                    let mut output = File::create(file).unwrap();
                    write!(output, "{}", record).unwrap();
                    println!("\n🔰 {}\n", "`playground.ds` file created.".cyan().bold());
                } else if input == ".quit" || input == ".q" {
                    println!("\n👋 {}\n", "Bye!".green().bold());
                    break;
                } else {
                    code_buffer.push(input);
                }
            }
        }
    }
}
