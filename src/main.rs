use clap::{Parser, Subcommand};

mod biggest;
mod caches;
mod display;

#[derive(Parser)]
#[command(name = "cleanmac", about = "macOS cleanup utility", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Caches {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Biggest {
        path: Option<String>,
        #[arg(long)]
        depth: Option<usize>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Caches { all, dry_run } => caches::run(all, dry_run),
        Commands::Biggest { path, depth } => biggest::run(path, depth),
    }
}
