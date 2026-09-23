//! mayfly — short-lived single-purpose agent hatches.

mod adapters;
mod fuzz;
mod hatch;
mod project;
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
    /// State directory for hatch records. Default: `<project>/.jagent/local/mayfly` inside
    /// a repo that has adopted `.jagent/`, else ~/.local/state/mayfly.
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
        /// Merge this project runner's defaults under the task (.jagent/agents/mayfly/<name>.json)
        #[arg(long)]
        runner: Option<String>,
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
        /// Merge this project runner's defaults under the task (.jagent/agents/mayfly/<name>.json)
        #[arg(long)]
        runner: Option<String>,
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
    /// List this project's runners (.jagent/agents/mayfly/*.json)
    Runners,
    /// Scaffold this project's runners (read.json + README.md); never overwrites
    InitRunners,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("current dir")?;
    let explicit = cli.state_dir;

    match cli.command {
        Commands::Validate { task, runner } => {
            let t = load_task(&task, runner.as_deref(), &cwd)?;
            if let Err(e) = adapters::check_supported(&t) {
                eprintln!("reject: {e}");
                std::process::exit(2);
            }
            for note in adapters::unenforced(&t) {
                eprintln!("note: {note}");
            }
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
            runner,
        } => {
            let t = load_task(&task, runner.as_deref(), &cwd)?;
            // Records go with the project the task runs in (its cwd), not the caller's.
            let store = open_store(explicit, &cwd.join(&t.cwd))?;
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
                // A failed hatch never exits 0, even when the harness itself did
                // (e.g. it exited cleanly but done_when failed): callers gate on this.
                let code = report.exit_code.filter(|c| *c != 0).unwrap_or(1);
                std::process::exit(code);
            }
            Ok(())
        }
        Commands::Status { id } => {
            let store = open_store(explicit, &cwd)?;
            let rec = store
                .get(&id)?
                .with_context(|| format!("no hatch with id {id}"))?;
            println!("{}", serde_json::to_string_pretty(&rec)?);
            Ok(())
        }
        Commands::Expire { id, reason } => {
            let store = open_store(explicit, &cwd)?;
            let reason = reason.unwrap_or_else(|| "forced expire".into());
            expire_hatch(&store, &id, &reason)?;
            eprintln!("expired: {id} ({reason})");
            Ok(())
        }
        Commands::List => {
            let store = open_store(explicit, &cwd)?;
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
        Commands::Runners => {
            let root = require_project(&cwd)?;
            let runners = project::list_runners(&root)?;
            if runners.is_empty() {
                eprintln!(
                    "no runners in {} — `mayfly init-runners` scaffolds one",
                    root.join(project::RUNNERS_DIR).display()
                );
            }
            for r in &runners {
                println!("{}", project::describe(r));
            }
            Ok(())
        }
        Commands::InitRunners => {
            let root = require_project(&cwd)?;
            let (written, notes) = project::init_runners(&root)?;
            if written.is_empty() {
                println!(
                    "runners already present in {} (nothing overwritten)",
                    root.join(project::RUNNERS_DIR).display()
                );
            }
            for p in written {
                println!("wrote {}", p.display());
            }
            for n in notes {
                eprintln!("note: {n}");
            }
            Ok(())
        }
    }
}

fn open_store(explicit: Option<PathBuf>, cwd: &std::path::Path) -> Result<HatchStore> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let (dir, warn) = project::resolve_state_dir(explicit, cwd, home.as_deref())?;
    if let Some(w) = warn {
        eprintln!("{w}");
    }
    HatchStore::open(Some(dir))
}

/// Where this worktree's runner definitions live; exit 2 with a hint outside an adopted repo.
fn require_project(cwd: &std::path::Path) -> Result<PathBuf> {
    match project::defs_root(cwd) {
        Some(root) => Ok(root),
        None => {
            eprintln!(
                "reject: {} is not inside a repo that has adopted .jagent/ — runners live in <repo>/{} (run jagent-adopt first)",
                cwd.display(),
                project::RUNNERS_DIR
            );
            std::process::exit(2);
        }
    }
}

fn load_task(path: &PathBuf, runner: Option<&str>, cwd: &std::path::Path) -> Result<MayflyTask> {
    let raw = if path.as_os_str() == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(path).with_context(|| format!("read task {}", path.display()))?
    };
    let mut value: serde_json::Value = serde_json::from_str(&raw).context("parse task JSON")?;
    if let Some(name) = runner {
        let root = require_project(cwd)?;
        let r = match project::load_runner(&root, name) {
            Ok(r) => r,
            Err(e) => {
                let known: Vec<String> = project::list_runners(&root)
                    .map(|rs| rs.into_iter().map(|r| r.name).collect())
                    .unwrap_or_default();
                eprintln!("reject: runner {name:?}: {e:#}");
                eprintln!(
                    "known runners: {}",
                    if known.is_empty() {
                        "(none — mayfly init-runners)".into()
                    } else {
                        known.join(", ")
                    }
                );
                std::process::exit(2);
            }
        };
        value = project::merge_under(&r.defaults, value);
    }
    let task: MayflyTask = serde_json::from_value(value).context("parse MayflyTask JSON")?;
    if task.task.trim().is_empty() {
        bail!("task string is empty");
    }
    Ok(task)
}
