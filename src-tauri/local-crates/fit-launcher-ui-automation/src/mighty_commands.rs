use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use specta::{Type, specta};
use std::path::PathBuf;
use tracing::info;

#[cfg(target_os = "windows")]
use tracing::error;

#[cfg(target_os = "windows")]
use std::time::Duration;

pub use fit_launcher_ipc::{ExclusionAction, ExclusionCleanupPolicy};

#[cfg(target_os = "windows")]
use crate::{
    controller_client::{ControllerCommand, ControllerEvent},
    controller_manager::ControllerManager,
    defender::{FolderExclusionEntry, load_exclusions, now_utc, save_exclusions},
};

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerType {
    Wine,
    Proton,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct LaunchRunner {
    pub id: String,
    pub name: String,
    pub runner_type: RunnerType,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct QueueStatus {
    pub queue: Vec<String>,
    pub active: Option<String>,
}

/// Returns detected Wine/Proton runners on Linux. Empty on other platforms.
#[tauri::command]
#[specta]
pub fn get_linux_launch_runners() -> Vec<LaunchRunner> {
    #[cfg(target_os = "linux")]
    {
        detect_linux_runners()
    }

    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_runners() -> Vec<LaunchRunner> {
    use std::collections::HashSet;
    use std::fs;

    let mut runners: Vec<LaunchRunner> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    let mut push_runner = |runner: LaunchRunner| {
        if seen_paths.insert(runner.path.clone()) {
            runners.push(runner);
        }
    };

    // System wine binaries from PATH
    for bin in ["wine", "wine64"] {
        if let Some(path) = find_in_path(bin) {
            let name = match bin {
                "wine64" => "System Wine (wine64)".to_string(),
                _ => "System Wine".to_string(),
            };
            push_runner(LaunchRunner {
                id: format!("system-{bin}"),
                name,
                runner_type: RunnerType::Wine,
                path,
            });
        }
    }

    let home = BaseDirs::new()
        .map(|b| b.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()));

    let mut steam_roots: Vec<PathBuf> = vec![
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
        home.join("snap/steam/common/.local/share/Steam"),
    ];

    // Deduplicate steam roots by canonical path where possible
    steam_roots.sort();
    steam_roots.dedup();

    for steam_root in steam_roots {
        if !steam_root.exists() {
            continue;
        }

        // Proton installs in steamapps/common
        let common_dir = steam_root.join("steamapps/common");
        if let Ok(entries) = fs::read_dir(&common_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let dir_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !dir_name.to_lowercase().starts_with("proton") {
                    continue;
                }
                let proton_bin = p.join("proton");
                if proton_bin.is_file() {
                    push_runner(LaunchRunner {
                        id: format!("proton:{}", proton_bin.to_string_lossy()),
                        name: format!("{dir_name} ({})", steam_root.display()),
                        runner_type: RunnerType::Proton,
                        path: proton_bin.to_string_lossy().to_string(),
                    });
                }
            }
        }

        // Custom compatibility tools
        let compat_dir = steam_root.join("compatibilitytools.d");
        if let Ok(entries) = fs::read_dir(&compat_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let dir_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let proton_bin = p.join("proton");
                if proton_bin.is_file() {
                    push_runner(LaunchRunner {
                        id: format!("compat:{}", proton_bin.to_string_lossy()),
                        name: format!("{dir_name} (compatibility tool)"),
                        runner_type: RunnerType::Proton,
                        path: proton_bin.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    runners
}

#[cfg(target_os = "linux")]
fn find_in_path(bin: &str) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

/// Start an executable using tauri::command.
///
/// On Windows, uses ShellExecuteW so UAC elevation is handled automatically.
/// On Linux, launches via the selected Wine/Proton runner.
#[tauri::command]
#[specta]
pub fn start_executable(path: String, runner: Option<LaunchRunner>) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);

    #[cfg(target_os = "windows")]
    {
        let _ = runner;
        use crate::encode_utf16le_with_null;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        use windows::core::PCWSTR;

        // Convert path to wide string
        let path_wide: Vec<u16> = encode_utf16le_with_null(&path_buf);

        // Get working directory (parent of executable)
        let current = PathBuf::from(".");
        let working_dir = path_buf.parent().unwrap_or(&current);
        let working_dir_wide: Vec<u16> = encode_utf16le_with_null(working_dir);

        // ShellExecuteW with "open" verb - Windows handles UAC automatically
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR::null(),
                PCWSTR(path_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR(working_dir_wide.as_ptr()),
                SW_SHOWNORMAL,
            )
        };

        let result_code = result.0 as isize;
        if result_code > 32 {
            info!("Executable launched via shell: {}", path_buf.display());
            Ok(())
        } else {
            let msg = format!(
                "Failed to launch {:?} via shell, error code: {}",
                path_buf, result_code
            );
            error!("{msg}");
            Err(msg)
        }
    }

    #[cfg(target_os = "linux")]
    {
        launch_with_runner_linux(&path_buf, runner)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (path_buf, runner);
        Err("Launching executables is not supported on this platform".to_string())
    }
}

#[cfg(target_os = "linux")]
fn launch_with_runner_linux(
    path: &std::path::Path,
    runner: Option<LaunchRunner>,
) -> Result<(), String> {
    use std::process::{Command, Stdio};

    let runner = runner.ok_or_else(|| {
        "No Wine/Proton runner selected. Please choose a runner to launch this game.".to_string()
    })?;

    if !path.exists() {
        return Err(format!("Executable not found: {}", path.display()));
    }

    let working_dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let base_dirs = BaseDirs::new().ok_or_else(|| "Failed to determine base directories".to_string())?;
    let data_dir = base_dirs
        .data_dir()
        .join("com.fitlauncher.carrotrub")
        .join("linux-runners");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to prepare runner data directory: {e}"))?;

    match runner.runner_type {
        RunnerType::Wine => {
            let prefix = data_dir.join("wineprefix-default");
            std::fs::create_dir_all(&prefix)
                .map_err(|e| format!("Failed to create Wine prefix: {e}"))?;

            info!(
                "Launching with Wine runner '{}' : {} {:?}",
                runner.name,
                runner.path,
                path
            );

            Command::new(&runner.path)
                .arg(path)
                .current_dir(&working_dir)
                .env("WINEPREFIX", &prefix)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to launch with Wine ({}): {e}", runner.path))?;
        }
        RunnerType::Proton => {
            let compat_dir = data_dir.join("compatdata").join("default");
            std::fs::create_dir_all(&compat_dir)
                .map_err(|e| format!("Failed to create Proton compat data directory: {e}"))?;

            info!(
                "Launching with Proton runner '{}' : {} run {:?}",
                runner.name,
                runner.path,
                path
            );

            Command::new(&runner.path)
                .arg("run")
                .arg(path)
                .current_dir(&working_dir)
                .env("STEAM_COMPAT_DATA_PATH", &compat_dir)
                .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &compat_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to launch with Proton ({}): {e}", runner.path))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
#[specta]
pub async fn folder_exclusion(action: ExclusionAction) -> Result<(), String> {
    let manager = ControllerManager::global();

    manager
        .ensure_running()
        .map_err(|e| format!("Controller not available: {}", e))?;

    info!("Sending command...");
    manager
        .send_command(&ControllerCommand::FolderExclusion {
            action: action.clone(),
        })
        .map_err(|e| format!("Failed to send command: {}", e))?;
    info!("command sent...");
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err("Folder exclusion timed out".into());
        }

        let taken_client = manager.take_client();
        let recv_result = match taken_client {
            Ok(Some(mut client)) => {
                let res = tokio::task::spawn_blocking(move || {
                    let r = client.recv_timeout(Duration::from_millis(500));
                    (client, r)
                })
                .await
                .map_err(|e| e.to_string());

                match res {
                    Ok((client_back, result)) => {
                        let _ = manager.put_client(client_back);
                        result.map_err(|e| e.to_string())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err("Controller not connected".to_string()),
        };

        match recv_result {
            Ok(Some(ControllerEvent::FolderExclusionResult { success, error })) => {
                if success {
                    info!("Folder exclusion succeeded");

                    let Some(base_dirs) = BaseDirs::new() else {
                        error!("Failed to determine base directories");
                        return Err(
                            "Folder added successfully but failed to determine base directories"
                                .to_string(),
                        );
                    };

                    let exclusions_file_path = base_dirs
                        .config_dir()
                        .join("com.fitlauncher.carrotrub")
                        .join("fitgirlConfig")
                        .join("settings")
                        .join("data")
                        .join("folder_exclusions.json");

                    let mut file = load_exclusions(&exclusions_file_path);

                    let excluded_path = match &action {
                        ExclusionAction::Add(p) | ExclusionAction::Remove(p) => p.clone(),
                    };

                    match &action {
                        ExclusionAction::Add(_) => {
                            manager.set_folder_exclusion_active(true)?;
                            if !file.entries.iter().any(|e| e.path == excluded_path) {
                                file.entries.push(FolderExclusionEntry {
                                    path: excluded_path.clone(),
                                    timestamp_utc: now_utc(),
                                });
                            }
                        }

                        ExclusionAction::Remove(_) => {
                            manager.set_folder_exclusion_active(false)?;
                            file.entries.retain(|e| e.path != excluded_path);
                        }
                    }

                    save_exclusions(&exclusions_file_path, &file)
                        .map_err(|e| format!("Failed to save exclusion file: {e}"))?;

                    info!("Path excluded and saved: {}", excluded_path);
                    return Ok(());
                } else {
                    return Err(error.unwrap_or("Folder exclusion failed".into()));
                }
            }
            Ok(Some(_)) => {
                // Ignore unrelated events
            }
            Ok(None) => {}
            Err(e) => {
                error!("Controller error: {}", e);
                return Err(e);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
#[specta]
pub async fn folder_exclusion(_action: ExclusionAction) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
#[specta]
pub async fn folder_exclusion_cleanup(policy: ExclusionCleanupPolicy) -> Result<(), String> {
    let manager = ControllerManager::global();

    manager
        .ensure_running()
        .map_err(|e| format!("Controller not available: {}", e))?;

    info!("Sending command...");
    manager
        .send_command(&ControllerCommand::CleanupPolicy {
            exclusion_folder: policy.clone(),
        })
        .map_err(|e| format!("Failed to send command: {}", e))?;
    info!("command sent...");
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err("Folder exclusion timed out".into());
        }

        let taken_client = manager.take_client();
        let recv_result = match taken_client {
            Ok(Some(mut client)) => {
                let res = tokio::task::spawn_blocking(move || {
                    let r = client.recv_timeout(Duration::from_millis(500));
                    (client, r)
                })
                .await
                .map_err(|e| e.to_string());

                match res {
                    Ok((client_back, result)) => {
                        let _ = manager.put_client(client_back);
                        result.map_err(|e| e.to_string())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err("Controller not connected".to_string()),
        };

        match recv_result {
            Ok(Some(ControllerEvent::FolderExclusionResult { success, error })) => {
                if success {
                    info!("Folder exclusion cleanup succeeded");

                    let Some(base_dirs) = BaseDirs::new() else {
                        error!("Failed to determine base directories");
                        return Err(
                            "Folder added successfully but failed to determine base directories"
                                .to_string(),
                        );
                    };

                    let exclusions_file_path = base_dirs
                        .config_dir()
                        .join("com.fitlauncher.carrotrub")
                        .join("fitgirlConfig")
                        .join("settings")
                        .join("data")
                        .join("folder_exclusions.json");

                    let mut file = load_exclusions(&exclusions_file_path);

                    let excluded_path = match &policy {
                        ExclusionCleanupPolicy::Keep(p)
                        | ExclusionCleanupPolicy::RemoveAfterInstall(p) => p.clone(),
                    };

                    match &policy {
                        ExclusionCleanupPolicy::Keep(_) => {}

                        ExclusionCleanupPolicy::RemoveAfterInstall(_) => {
                            file.entries.retain(|e| e.path != excluded_path);
                        }
                    }

                    save_exclusions(&exclusions_file_path, &file)
                        .map_err(|e| format!("Failed to save exclusion file: {e}"))?;

                    info!("Path excluded and saved: {}", excluded_path);
                    return Ok(());
                } else {
                    return Err(error.unwrap_or("Folder exclusion failed".into()));
                }
            }
            Ok(Some(_)) => {
                // Ignore unrelated events
            }
            Ok(None) => {}
            Err(e) => {
                error!("Controller error: {}", e);
                return Err(e);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
#[specta]
pub async fn folder_exclusion_cleanup(_policy: ExclusionCleanupPolicy) -> Result<(), String> {
    Ok(())
}
