//! `cargo xtask` — workspace task runner for the x2p platform.
//!
//! Wired as the workspace task runner via the `[alias] xtask = "run --package
//! xtask --"` entry in `.cargo/config.toml`, so contributors and CI invoke it
//! as `cargo xtask <subcommand>` from anywhere inside the workspace.
//!
//! Real subcommand implementations land in later tasks:
//!
//! - [`Command::GenSchemas`]       → task 14.1 (schema generation + CI drift gate)
//! - [`Command::PackageExtension`] → task 26.2 (browser extension store packaging)
//! - [`Command::Release`]          → task 24   (release engineering)
//!
//! At this scaffold step every subcommand prints a stable `not_yet_implemented`
//! message naming the future task and exits with a non-zero status so that no
//! caller mistakes the stub for a successful run.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Stable exit code returned by every stub subcommand. A non-zero value keeps
/// CI from green-lighting a release on a stub implementation by accident.
const EXIT_NOT_YET_IMPLEMENTED: u8 = 2;

/// Top-level `cargo xtask` command.
#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    bin_name = "cargo xtask",
    about = "Workspace task runner for the x2p platform.",
    long_about = "Workspace task runner for the x2p platform.\n\n\
                  Subcommands wrap repeatable engineering tasks (schema \
                  generation, browser-extension packaging, release \
                  engineering) so that they can be invoked uniformly from \
                  developer machines and from CI.",
    version,
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// All `cargo xtask` subcommands. Each variant is implemented in a later task;
/// the variants exist now so that `cargo xtask --help` is stable and so that
/// downstream tasks (CI workflows, docs, scripts) can wire against them.
#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate JSON Schemas and TypeScript bindings from `x2p-core` types.
    ///
    /// Implementation: task 14.1.
    #[command(name = "gen-schemas")]
    GenSchemas,

    /// Build and sign browser-extension submission archives (Chrome Web Store,
    /// Firefox AMO, enterprise side-load).
    ///
    /// Implementation: task 26.2.
    #[command(name = "package-extension")]
    PackageExtension,

    /// Drive a coordinated release of the CLI, browser extension, and
    /// associated artifacts.
    ///
    /// Implementation: task 24.
    #[command(name = "release")]
    Release,
}

impl Command {
    /// Human-readable subcommand name as exposed on the CLI.
    const fn name(&self) -> &'static str {
        match self {
            Self::GenSchemas => "gen-schemas",
            Self::PackageExtension => "package-extension",
            Self::Release => "release",
        }
    }

    /// Future task number that will land the real implementation.
    const fn future_task(&self) -> &'static str {
        match self {
            Self::GenSchemas => "14.1",
            Self::PackageExtension => "26.2",
            Self::Release => "24",
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    eprintln!(
        "xtask {sub}: not_yet_implemented (lands in task {task})",
        sub = cli.command.name(),
        task = cli.command.future_task(),
    );
    ExitCode::from(EXIT_NOT_YET_IMPLEMENTED)
}
