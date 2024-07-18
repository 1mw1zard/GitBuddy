use clap::Command;

use crate::ai::{get_command, handle_command};

mod ai;
mod llm;
mod config;

fn main() {
    let matches = Command::new("gitbuddy")
        .version("0.1")
        .about("An AI-driven tool designed to simplify your Git commit process.")
        .subcommands(vec![get_command()])
        .get_matches();

    match matches.subcommand() {
        Some(("ai", m)) => {
            handle_command(m)
        }
        _ => {
            println!("No subcommand provided.");
        }
    }
}