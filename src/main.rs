use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gwt")]
#[command(about = "A git worktree manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure gwt
    Config,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config => {
            println!("Config command executed");
        }
    }
}
