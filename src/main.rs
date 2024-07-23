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
    Config {
        #[arg(value_enum)]
        vendor: llm::PromptModel,
        #[arg(long)]
        api_key: String,
        #[arg(long)]
        model: Option<String>,
    },
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ai { dry_run }) => {
            ai::handler(*dry_run);
        }
        Some(Commands::Config { vendor, api_key, model }) => {
            let model = if let Some(model) = model {
                model.to_string()
            } else {
                vendor.default_model().to_string()
            };

            config::handler(vendor, api_key, model.as_str()).unwrap();
        }
        None => {
            println!("No subcommand provided.");
        }
    }
}
