//! mayfly — short-lived single-purpose agent hatches.

mod adapters;
mod fuzz;
mod hatch;
mod prompt;
mod store;
mod task;
mod ttl;
mod worktree;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::hatch::{expire_hatch, plan_hatch, run_hatch, HatchOpts};
use crate::store::HatchStore;
use crate::task::MayflyTask;
use crate::worktree::WorktreeSpec;

#[derive(Parser, Debug)]
#[command(
    name = "mayfly",
    about = "Short-lived agents. One task. Then gone.",
    version
)]
struct Cli {
    /// State directory for hatch records (default: ~/.local/state/mayfly)
    #[arg(long, global = true, env = "MAYFLY_STATE_DIR")]
    state_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate a task (schema + fuzziness). Does not spawn.
    Validate {
        /// Task JSON file, or `-` for stdin
        task: PathBuf,
    },
    /// Hatch a mayfly for one purpose
    Hatch {
        /// Task JSON file, or `-` for stdin
        task: PathBuf,
        /// Print plan only; do not launch the harness
        #[arg(long)]
        dry_run: bool,
        /// Skip fuzziness gate (not recommended)
        #[arg(long)]
        allow_fuzzy: bool,
        /// Provision cwd via `buckets worktree create` against this repo
        #[arg(long)]
        worktree: Option<PathBuf>,
        /// Branch for --worktree (default: mayfly/<hatch-id>)
        #[arg(long)]
        branch: Option<String>,
        /// Base ref for --worktree (passed to buckets --from)
        #[arg(long)]
        from: Option<String>,
        /// Keep the buckets worktree after hatch (default: remove --force)
        #[arg(long)]
        keep_worktree: bool,
    },
    /// Show status of a hatch
    Status { id: String },
    /// Force a mayfly to expire
    Expire {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// List local hatches
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = HatchStore::open(cli.state_dir)?;

    match cli.command {
        Commands::Validate { task } => {
            let t = load_task(&task)?;
            match fuzz::check(&t) {
                Ok(()) => {
                    println!("ok: task is concrete enough for a mayfly");
                    println!("  harness: {}", t.harness);
                    println!("  ttl:     {}", t.ttl);
                    println!("  done:    {}", t.done_when.summary());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("reject: {e}");
                    eprintln!("hint: give one concrete purpose and a mechanical done_when");
                    std::process::exit(2);
                }
            }
        }
        Commands::Hatch {
            task,
            dry_run,
            allow_fuzzy,
            worktree,
            branch,
            from,
            keep_worktree,
        } => {
            let t = load_task(&task)?;
            if !allow_fuzzy {
                if let Err(e) = fuzz::check(&t) {
                    eprintln!("reject: {e}");
                    eprintln!("(pass --allow-fuzzy to hatch anyway — not recommended)");
                    std::process::exit(2);
                }
            }
            let opts = HatchOpts {
                worktree: worktree.map(|repo| WorktreeSpec {
                    repo,
                    branch,
                    from,
                    keep: keep_worktree,
                }),
            };
            let plan = plan_hatch(&store, &t, &opts)?;
            if dry_run {
                println!("{}", serde_json::to_string_pretty(&plan)?);
                return Ok(());
            }
            let report = run_hatch(&store, &t, plan, &opts)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.success {
                std::process::exit(report.exit_code.unwrap_or(1));
            }
            Ok(())
        }
        Commands::Status { id } => {
            let rec = store
                .get(&id)?
                .with_context(|| format!("no hatch with id {id}"))?;
            println!("{}", serde_json::to_string_pretty(&rec)?);
            Ok(())
        }
        Commands::Expire { id, reason } => {
            let reason = reason.unwrap_or_else(|| "forced expire".into());
            expire_hatch(&store, &id, &reason)?;
            eprintln!("expired: {id} ({reason})");
            Ok(())
        }
        Commands::List => {
            for rec in store.list()? {
                println!(
                    "{}\t{}\t{}\t{}",
                    rec.id,
                    rec.state,
                    rec.harness,
                    rec.task.chars().take(60).collect::<String>()
                );
            }
            Ok(())
        }
    }
}

fn load_task(path: &PathBuf) -> Result<MayflyTask> {
    let raw = if path.as_os_str() == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("read task {}", path.display()))?
    };
    let task: MayflyTask = serde_json::from_str(&raw).context("parse MayflyTask JSON")?;
    if task.task.trim().is_empty() {
        bail!("task string is empty");
    }
    Ok(task)
}
