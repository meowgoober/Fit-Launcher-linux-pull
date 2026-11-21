//! FitLauncher Installer Controller
//!
//! Windows-only elevated helper process for privileged installer operations.
//! On Linux this binary is a no-op stub.

#[cfg(windows)]
pub mod automation;
#[cfg(windows)]
pub mod defender;
#[cfg(windows)]
pub mod errors;
#[cfg(windows)]
pub mod events;
#[cfg(windows)]
pub mod installer;
#[cfg(windows)]
pub mod ipc;
#[cfg(windows)]
pub mod utils;

#[cfg(windows)]
fn main() {
    use std::env;
    use tracing::{Level, error, info};
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::FmtSubscriber;

    use crate::ipc::server::IpcServer;

    fn run_server(pipe_name: &str) -> anyhow::Result<()> {
        let mut server = IpcServer::new(pipe_name)?;
        server.run()
    }

    let logs_dir = directories::BaseDirs::new()
        .expect("Could not determine base directories")
        .config_dir()
        .join("com.fitlauncher.carrotrub")
        .join("logs");

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("controller")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&logs_dir)
        .expect("Failed to build rolling file appender");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(false)
        .with_writer(file_writer)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Installer Controller starting...");

    let args: Vec<String> = env::args().collect();
    let pipe_name = args.get(1).cloned().unwrap_or_else(|| {
        r"\\.\pipe\FitLauncherService".to_string()
    });

    info!("Using pipe: {}", pipe_name);

    if let Err(e) = run_server(&pipe_name) {
        error!("Controller failed: {:#}", e);
        std::process::exit(1);
    }

    info!("Installer Controller shutting down");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("FitLauncherService (installer controller) is only supported on Windows.");
}
