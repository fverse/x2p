use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod capture;
mod render;

#[derive(Parser)]
#[command(name = "x2p", version, about = "anything-to-prompt: capture a page, render a prompt")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Fetch a URL and emit a Bundle as JSON.
    Capture {
        url: String,
        /// Write bundle JSON to file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Render a Bundle JSON file to a markdown prompt on stdout.
    Render {
        bundle: PathBuf,
        /// Optional token budget (cl100k). Blocks are pruned until the prompt fits.
        #[arg(long)]
        budget: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Capture { url, output } => capture::run(&url, output.as_deref()).await,
        Cmd::Render { bundle, budget } => render::run(&bundle, budget),
    }
}
