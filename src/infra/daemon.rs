use crate::config::default_emaki_dir;
use crate::error::{EmakiError, Result};
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct DaemonManager;

impl DaemonManager {
    pub fn pid_file() -> PathBuf {
        default_emaki_dir().join("emaki.pid")
    }

    pub fn log_file() -> PathBuf {
        default_emaki_dir().join("emaki.log")
    }

    pub fn get_running_pid() -> Option<u32> {
        let pid_path = Self::pid_file();
        if !pid_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&pid_path).ok()?;
        let pid: u32 = content.trim().parse().ok()?;

        // Verify if PID is actually alive via kill -0 <pid>
        let is_alive = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false);

        if is_alive {
            Some(pid)
        } else {
            // Clean up stale pidfile
            let _ = std::fs::remove_file(&pid_path);
            None
        }
    }

    pub fn start_background() -> Result<()> {
        if let Some(pid) = Self::get_running_pid() {
            println!("⚠️  Emaki daemon is already running in background (PID: {pid})");
            println!("   Stop it first with: emaki stop");
            return Ok(());
        }

        let emaki_dir = default_emaki_dir();
        std::fs::create_dir_all(&emaki_dir)?;

        let current_exe = std::env::current_exe().map_err(EmakiError::Io)?;
        let log_path = Self::log_file();

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let log_file_err = log_file.try_clone()?;

        let mut cmd = Command::new(current_exe);
        cmd.args(["daemon", "--foreground"])
            .stdin(Stdio::null())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_file_err))
            .process_group(0);

        let child = cmd.spawn().map_err(EmakiError::Io)?;

        let pid = child.id();
        std::fs::write(Self::pid_file(), pid.to_string())?;

        println!("🚀 絵巻 (Emaki) daemon started in background (PID: {pid})");
        println!("  📜 Log Output   : {:?}", log_path.display());
        println!("  🔍 Check Status : emaki status");
        println!("  📋 Live Logs    : emaki logs");
        println!("  🛑 Stop Daemon  : emaki stop");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        match Self::get_running_pid() {
            Some(pid) => {
                let _ = Command::new("kill").arg(pid.to_string()).status();
                let _ = std::fs::remove_file(Self::pid_file());
                println!("🛑 絵巻 (Emaki) daemon (PID: {pid}) stopped");
            }
            None => {
                println!("ℹ️  No active Emaki daemon found running in background");
            }
        }
        Ok(())
    }

    pub fn logs() -> Result<()> {
        let log_path = Self::log_file();
        if !log_path.exists() {
            println!("ℹ️  No log file found at {:?}", log_path.display());
            return Ok(());
        }

        println!(
            "📜 Tailing live daemon logs from {:?} (Ctrl+C to quit)...",
            log_path.display()
        );
        let _ = Command::new("tail")
            .args(["-f", "-n", "50", log_path.to_str().unwrap_or("emaki.log")])
            .status();
        Ok(())
    }
}
