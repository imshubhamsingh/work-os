mod cli;
mod error;

use cli::{Cli, Commands};
use clap::{Parser};

fn greet_user(name: String) -> error::Result<String> {
   if name.is_empty() {
    return Err(error::WorkOsError::Config("Name is required".to_string()));
   }
   Ok(format!("Hello, {}! 👋", name))
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> error::Result<()> {
    match cli.command {
        Commands::Hello { name } => {
            let message = greet_user(name)?;
            println!("{}", message);
            Ok(())
        }
    }
}
