use crate::error::{EmakiError, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct ServiceManager;

impl ServiceManager {
    pub fn handle_command(action: Option<&str>) -> Result<()> {
        match action.unwrap_or("status") {
            "install" => Self::install(),
            "uninstall" | "remove" => Self::uninstall(),
            "status" => Self::status(),
            "start" => Self::systemctl_action("start"),
            "stop" => Self::systemctl_action("stop"),
            "restart" => Self::systemctl_action("restart"),
            "logs" => Self::logs(),
            unknown => {
                eprintln!("Unknown service action: '{unknown}'");
                eprintln!("Usage: emaki service [install | status | start | stop | restart | logs | uninstall]");
                std::process::exit(1);
            }
        }
    }

    pub fn install() -> Result<()> {
        let bin_path = std::env::current_exe().map_err(EmakiError::Io)?;
        let home = std::env::var("HOME")
            .map_err(|_| EmakiError::Config("Could not find HOME environment variable".into()))?;

        let service_dir = PathBuf::from(&home).join(".config/systemd/user");
        std::fs::create_dir_all(&service_dir)?;
        let service_file = service_dir.join("emaki.service");

        let unit_content = format!(
            r#"[Unit]
Description=絵巻 (Emaki) WhatsApp Reel Daemon
After=network.target

[Service]
Type=simple
WorkingDirectory=%h/.emaki
ExecStart={} daemon --foreground
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
"#,
            bin_path.display()
        );

        std::fs::write(&service_file, unit_content)?;

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        let enable_status = Command::new("systemctl")
            .args(["--user", "enable", "--now", "emaki"])
            .status()
            .map_err(EmakiError::Io)?;

        if !enable_status.success() {
            return Err(EmakiError::WhatsApp(
                "Failed to enable systemd user service via systemctl".into(),
            ));
        }

        if let Ok(user) = std::env::var("USER") {
            let _ = Command::new("loginctl")
                .args(["enable-linger", &user])
                .status();
        }

        println!("✅ Systemd user service installed and started!");
        println!("  📄 Unit File    : {:?}", service_file.display());
        println!("  🚀 Auto-Restart  : Enabled (auto-starts on boot & restarts on failure)");
        println!("  📜 Live Logs     : emaki service logs");
        println!("  🔍 Status Check  : emaki service status");
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        let _ = Command::new("systemctl")
            .args(["--user", "disable", "--now", "emaki"])
            .status();

        if let Ok(home) = std::env::var("HOME") {
            let service_file = PathBuf::from(home).join(".config/systemd/user/emaki.service");
            if service_file.exists() {
                std::fs::remove_file(&service_file)?;
            }
        }

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        println!("✅ Systemd user service uninstalled and stopped");
        Ok(())
    }

    pub fn status() -> Result<()> {
        let _ = Command::new("systemctl")
            .args(["--user", "status", "emaki"])
            .status();
        Ok(())
    }

    pub fn logs() -> Result<()> {
        let _ = Command::new("journalctl")
            .args(["--user", "-u", "emaki", "-f", "-n", "50"])
            .status();
        Ok(())
    }

    fn systemctl_action(action: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .args(["--user", action, "emaki"])
            .status()
            .map_err(EmakiError::Io)?;

        if status.success() {
            println!("✅ emaki service {action} succeeded");
        } else {
            eprintln!("❌ emaki service {action} failed");
        }
        Ok(())
    }
}
