//! `gs` — a small, opinionated command line interface backed by `grit-lib`.
//!
//! `gs` deliberately does not mirror Git's UX. It favors a single obvious way
//! to do the common thing, plain-language output, and a status screen that
//! doubles as the home base: running `gs` with no arguments shows you where you
//! are, what's changed, and what to do next.

mod commands;
mod context;
mod net;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// A simplified alternative to the Git-compatible `grit` command line.
#[derive(Debug, Parser)]
#[command(name = "gs", version, about = "A simple Grit-powered CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Subcommands of `gs remote`.
#[derive(Debug, Subcommand)]
enum RemoteAction {
    /// Add a new remote.
    Add {
        /// Short name for the remote (e.g. origin).
        name: String,
        /// The remote's URL or path.
        url: String,
    },
}

/// Top-level `gs` commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Create a new, empty repository.
    Init {
        /// Where to create the repository (defaults to the current directory).
        path: Option<String>,
        /// Create a bare repository (no working tree).
        #[arg(long)]
        bare: bool,
    },
    /// Copy a remote repository into a new directory.
    Clone {
        /// The repository to clone (URL or local path).
        url: String,
        /// Directory to clone into (defaults to the repository name).
        dir: Option<String>,
    },
    /// List remotes, or add one.
    Remote {
        #[command(subcommand)]
        action: Option<RemoteAction>,
    },
    /// Show recent commits reachable from HEAD.
    Log {
        /// Continue listing from before this commit (for paging).
        #[arg(long)]
        before: Option<String>,
    },
    /// Show what's changed and where you are (this is the default).
    #[command(alias = "st")]
    Status,
    /// List the commits on this branch that aren't on the target branch yet.
    #[command(alias = "sl")]
    Shortlog,
    /// Stage changes. With no paths, stages everything.
    Add {
        /// Files or directories to stage. Omit to stage all changes.
        paths: Vec<String>,
    },
    /// Record the staged changes as a new commit.
    Commit {
        /// Commit message (you can also pass it with -m).
        message: Option<String>,
        /// Commit message.
        #[arg(short = 'm', long = "message", conflicts_with = "message")]
        message_flag: Option<String>,
        /// Stage every change first, then commit.
        #[arg(short = 'a', long = "all")]
        all: bool,
    },
    /// List branches, or create / delete one.
    Branch {
        /// Name of the branch to create. Omit to list branches.
        name: Option<String>,
        /// Delete the named branch instead of creating it.
        #[arg(short = 'd', long = "delete")]
        delete: bool,
    },
    /// Switch to another branch.
    #[command(alias = "checkout", alias = "co")]
    Switch {
        /// Branch to switch to.
        name: String,
        /// Create the branch first, then switch to it.
        #[arg(short = 'c', long = "create")]
        create: bool,
    },
    /// Merge another branch into the current one.
    Merge {
        /// Branch to merge in.
        branch: String,
    },
    /// Download refs and objects from a remote.
    Fetch {
        /// Remote to fetch from (defaults to origin).
        remote: Option<String>,
    },
    /// Fetch from the remote and integrate it into the current branch.
    Pull,
    /// Publish the current branch to its remote.
    Push,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Status) {
        Command::Init { path, bare } => commands::init::run(path, bare),
        Command::Clone { url, dir } => commands::clone::run(&url, dir),
        Command::Remote { action } => {
            let add = action.map(|RemoteAction::Add { name, url }| (name, url));
            commands::remote::run(add)
        }
        Command::Log { before } => commands::log::run(before),
        Command::Status => commands::status::run(),
        Command::Shortlog => commands::shortlog::run(),
        Command::Add { paths } => commands::add::run(&paths),
        Command::Commit {
            message,
            message_flag,
            all,
        } => commands::commit::run(message.or(message_flag), all),
        Command::Branch { name, delete } => commands::branch::run(name, delete),
        Command::Switch { name, create } => commands::switch::run(&name, create),
        Command::Merge { branch } => commands::merge::run(&branch),
        Command::Fetch { remote } => commands::fetch::run(remote),
        Command::Pull => commands::pull::run(),
        Command::Push => commands::push::run(),
    }
}
