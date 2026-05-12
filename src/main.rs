use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod ai;
mod config;
mod llm;
mod prompt;

use crate::llm::PromptModel;
use prompt::Prompt;

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

    #[arg(long)]
    vendor: Option<PromptModel>,

    #[arg(short, long)]
    model: Option<String>,

    #[arg(short='p', long, default_value_t=Prompt::P1)]
    prompt: Prompt,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a commit message based on the current state of the repository
    Ai {
        /// push the commit to the remote repository
        #[arg(short, long, default_value_t = false)]
        push: bool,
        /// test argument, generate commit message but not commit
        #[arg(long, default_value_t = false)]
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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ai { push, dry_run }) => {
            ai::handler(*push, *dry_run, false, false, cli.vendor, cli.model.clone(), cli.prompt).await?;
        }
        Some(Commands::Config { vendor, api_key, model }) => {
            let model = if let Some(model) = model {
                model.to_string()
            } else {
                vendor.default_model().to_string()
            };

            config::handler(vendor, api_key, model.as_str())?;
        }
        None => {
            ai::handler(false, false, true, true, cli.vendor, cli.model.clone(), cli.prompt).await?;
        }
    }

    Ok(())
}
