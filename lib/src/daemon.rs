use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Check if a process with the given PID is still running.
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map_or(false, |o| {
            String::from_utf8_lossy(&o.stdout).contains(&pid.to_string())
        })
}

/// Find the PID of a running `centy-daemon` process.
///
/// Checks the PID file at `~/.centy/daemon.pid` first, then falls back to
/// searching for the process by name.
fn find_daemon_pid(home_dir: &Path) -> Option<u32> {
    // Check PID file first
    let pid_file = home_dir.join(".centy").join("daemon.pid");
    if let Ok(contents) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = contents.trim().parse::<u32>() {
            if is_process_running(pid) {
                return Some(pid);
            }
        }
    }

    find_daemon_pid_by_name()
}

/// Search for a running `centy-daemon` process by name.
#[cfg(unix)]
fn find_daemon_pid_by_name() -> Option<u32> {
    let output = Command::new("pgrep")
        .args(["-x", "centy-daemon"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().lines().next()?.trim().parse().ok()
}

#[cfg(windows)]
fn find_daemon_pid_by_name() -> Option<u32> {
    let output = Command::new("tasklist")
        .args([
            "/FI",
            "IMAGENAME eq centy-daemon.exe",
            "/NH",
            "/FO",
            "CSV",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !line.contains("centy-daemon") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if let Some(pid_field) = parts.get(1) {
            if let Ok(pid) = pid_field.trim_matches('"').parse() {
                return Some(pid);
            }
        }
    }

    None
}

/// Stop a daemon process gracefully, falling back to a forced kill.
fn stop_daemon(pid: u32) -> Result<(), String> {
    send_term_signal(pid)?;

    // Wait up to 5 seconds for graceful shutdown
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(500));
        if !is_process_running(pid) {
            return Ok(());
        }
    }

    // Force kill if still running
    send_kill_signal(pid);

    thread::sleep(Duration::from_millis(500));

    if is_process_running(pid) {
        return Err(format!(
            "failed to stop daemon (PID {pid}) after forced kill"
        ));
    }

    Ok(())
}

#[cfg(unix)]
fn send_term_signal(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to send SIGTERM to daemon (PID {pid}): {e}"))?;

    if !status.success() {
        return Err(format!("failed to send SIGTERM to daemon (PID {pid})"));
    }

    Ok(())
}

#[cfg(windows)]
fn send_term_signal(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to terminate daemon (PID {pid}): {e}"))?;

    if !status.success() {
        return Err(format!("failed to terminate daemon (PID {pid})"));
    }

    Ok(())
}

#[cfg(unix)]
fn send_kill_signal(pid: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(windows)]
fn send_kill_signal(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Start the daemon process in the background.
fn start_daemon(binary_path: &Path) -> Result<(), String> {
    Command::new(binary_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start daemon: {e}"))?;

    Ok(())
}

/// Restart the daemon if it is currently running.
///
/// Returns `true` if the daemon was found and restarted, `false` if it was not
/// running. Errors indicate that the daemon was found but could not be stopped
/// or restarted.
pub fn restart_if_running(binary_path: &Path) -> Result<bool, String> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;

    let Some(pid) = find_daemon_pid(&home) else {
        return Ok(false);
    };

    stop_daemon(pid)?;
    start_daemon(binary_path)?;

    Ok(true)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn is_process_running_nonexistent() {
        assert!(!is_process_running(4_294_967_295));
    }

    #[cfg(unix)]
    #[test]
    fn is_process_running_self() {
        let pid = std::process::id();
        assert!(is_process_running(pid));
    }

    #[test]
    fn find_daemon_pid_stale_pid_file_skips_dead_process() {
        let tmp = tempfile::tempdir().unwrap();
        let centy_dir = tmp.path().join(".centy");
        std::fs::create_dir_all(&centy_dir).unwrap();
        // PID that almost certainly doesn't exist
        std::fs::write(centy_dir.join("daemon.pid"), "4294967295").unwrap();

        let result = find_daemon_pid(tmp.path());
        // The PID file should be ignored since the process is dead.
        // The result depends on whether a centy-daemon is actually running
        // (found via pgrep fallback), so we only verify no panic occurs.
        let _ = result;
    }

    #[test]
    fn find_daemon_pid_invalid_pid_file_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let centy_dir = tmp.path().join(".centy");
        std::fs::create_dir_all(&centy_dir).unwrap();
        std::fs::write(centy_dir.join("daemon.pid"), "not-a-number").unwrap();

        // Invalid PID file should not cause errors
        let _ = find_daemon_pid(tmp.path());
    }

    #[cfg(unix)]
    #[test]
    fn find_daemon_pid_valid_pid_file_for_running_process() {
        let tmp = tempfile::tempdir().unwrap();
        let centy_dir = tmp.path().join(".centy");
        std::fs::create_dir_all(&centy_dir).unwrap();
        // Write our own PID - it's a running process
        let pid = std::process::id();
        std::fs::write(centy_dir.join("daemon.pid"), pid.to_string()).unwrap();

        let result = find_daemon_pid(tmp.path());
        assert_eq!(result, Some(pid));
    }
}
