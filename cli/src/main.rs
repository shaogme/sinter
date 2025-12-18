mod compiler;
mod themes;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build the site
    Build(BuildArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Input directory
    #[arg(short, long, default_value = "./posts")]
    input: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "./web/sinter_data")]
    output: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Path to config file
    #[arg(short, long, default_value = "./sinter.toml")]
    config: PathBuf,

    /// Path to themes configuration
    #[arg(long, default_value = "themes/themes.toml")]
    themes_config: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build(args) => {
            // Initialize logging
            let log_level = if args.verbose {
                Level::DEBUG
            } else {
                Level::INFO
            };

            let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");

            info!("Starting Sinter compilation...");

            // Process themes
            let web_themes_dir = Path::new("web/themes");
            if args.themes_config.exists() {
                themes::process_themes(&args.themes_config, web_themes_dir)?;
            } else {
                info!(
                    "Themes configuration not found at {:?}, skipping theme build.",
                    args.themes_config
                );
            }

            info!("Input directory: {:?}", args.input);
            info!("Output directory: {:?}", args.output);

            // Implement core compilation logic here
            compiler::compile(&args.input, &args.output, &args.config)?;
        }
    }

    Ok(())
}
