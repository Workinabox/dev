mod git;
mod github;
mod monitor;
mod org;
mod release;
mod reporter;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use reporter::{DynReporter, PlainReporter};
use std::path::PathBuf;
use std::time::Duration;
use std::{io::IsTerminal, sync::Arc};

#[derive(Parser, Debug)]
#[command(name = "dev")]
#[command(about = "Admin tools for the Workinabox organization")]
struct Cli {
    /// Disable the ratatui UI (use plain stderr output).
    #[arg(long, default_value_t = false)]
    no_tui: bool,

    /// Exit automatically when the command completes successfully (TUI mode only).
    #[arg(long, default_value_t = false)]
    auto_exit: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Tag and release all repos in the Workinabox organization.
    ///
    /// This tags local repos and pushes tags to origin. It then polls GitHub Releases
    /// until required assets are present and stable before proceeding.
    Release {
        /// Version/tag to create (SemVer).
        ///
        /// Examples: v1.2.3, 1.2.3, v1.2.3-rc.1, 1.2.3-rc.1
        #[arg(long)]
        version: String,

        /// Directory containing the sibling repos (.github/, dev/).
        #[arg(long)]
        repos_root: Option<PathBuf>,

        /// GitHub org/owner.
        #[arg(long, default_value = "Workinabox")]
        owner: String,

        /// Don't create or push tags; just print what would happen.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Resume a partially completed release (skip tag creation/push for repos
        /// that already have the tag on origin, but still poll assets and continue).
        #[arg(long, default_value_t = false)]
        resume: bool,

        /// Poll interval in seconds.
        #[arg(long, default_value_t = 10)]
        poll_interval_secs: u64,

        /// Timeout in seconds per repo while waiting for release assets.
        #[arg(long, default_value_t = 45 * 60)]
        timeout_secs: u64,
    },

    /// Show a live organization monitor dashboard.
    ///
    /// This command does not perform any actions; it only displays status.
    Monitor {
        /// GitHub org/owner.
        #[arg(long, default_value = "Workinabox")]
        owner: String,

        /// Poll interval in seconds.
        #[arg(long, default_value_t = 60)]
        poll_interval_secs: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let use_tui = !cli.no_tui && std::io::stdout().is_terminal() && std::io::stderr().is_terminal();

    if use_tui {
        let (tx, rx) = crossbeam_channel::unbounded();
        let reporter: DynReporter = Arc::new(reporter::ChannelReporter::new(tx.clone()));
        reporter.step("Initializing".to_string(), "Starting dev…".to_string());
        reporter.ok("OK".to_string());

        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let command = cli.command;
        let worker = std::thread::spawn({
            let reporter = reporter.clone();
            let tx = tx.clone();
            let shutdown = shutdown.clone();
            move || {
                let is_monitor = matches!(&command, Commands::Monitor { .. });

                let result = match command {
                    Commands::Monitor {
                        owner,
                        poll_interval_secs,
                    } => monitor::run(
                        monitor::MonitorArgs {
                            owner,
                            poll_interval: Duration::from_secs(poll_interval_secs),
                        },
                        tx.clone(),
                        reporter.clone(),
                        shutdown,
                    ),
                    other => run_command(other, reporter.clone()),
                };

                if let Err(ref e) = result {
                    reporter.step(
                        "Failed".to_string(),
                        "An error occurred. See Status for details. Press q to quit.".to_string(),
                    );
                    reporter.error(format!("{e:#}"));
                }

                if !is_monitor {
                    let _ = tx.send(crate::tui::UiEvent::Finished { ok: result.is_ok() });
                }

                result
            }
        });

        let ui_res = tui::run(rx, cli.auto_exit);

        shutdown.store(true, std::sync::atomic::Ordering::SeqCst);

        let worker_res = match worker.join() {
            Ok(r) => r,
            Err(_) => Err(anyhow::anyhow!("worker thread panicked")),
        };

        ui_res?;

        if let Err(e) = worker_res {
            eprintln!("{e:?}");
            std::process::exit(1);
        }

        return Ok(());
    }

    let reporter: DynReporter = Arc::new(PlainReporter::new());
    run_command(cli.command, reporter)
}

fn run_command(command: Commands, reporter: DynReporter) -> Result<()> {
    match command {
        Commands::Release {
            version,
            repos_root,
            owner,
            dry_run,
            resume,
            poll_interval_secs,
            timeout_secs,
        } => release::run(
            release::ReleaseArgs {
                version,
                repos_root,
                owner,
                dry_run,
                resume,
                poll_interval: Duration::from_secs(poll_interval_secs),
                timeout: Duration::from_secs(timeout_secs),
            },
            reporter,
        ),

        Commands::Monitor { .. } => {
            anyhow::bail!("monitor requires a TUI. Re-run without --no-tui");
        }
    }
}
