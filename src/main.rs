use clap::{Parser, Subcommand};

mod ai;
mod llm;
mod config;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "An AI-driven tool designed to simplify your Git commit process."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a commit message based on the current state of the repository
    Ai {
        /// test argument, generate commit message but not commit
        #[arg(short, long)]
        dry_run: bool,
    },
    Config {},
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ai { dry_run }) => {
            ai::handler(*dry_run);
        }
        Some(Commands::Config {}) => {
            config::handler();
        }
        None => {
            println!("No subcommand provided.");
        }
    }
}
