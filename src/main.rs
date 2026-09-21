mod config;
mod domain;
mod error;
mod infra;

use config::Config;
use error::Result;
use infra::{CompletionGenerator, DaemonManager, ServiceManager, WhatsAppBot};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("daemon");

    if command != "completion" {
        print_banner();
    }

    let mut config = Config::load_auto();
    config.ensure_directories()?;
    let bot = WhatsAppBot::new(config.clone());

    match command {
        "login" => {
            info!("Running in interactive login mode...");
            bot.start_login().await?;
        }
        "status" => {
            print_status(&config).await?;
        }
        "config" => {
            handle_config_command(&args, &mut config).await?;
        }
        "whitelist" => {
            handle_whitelist_command(&args, &mut config).await?;
        }
        "service" => {
            let action = args.get(2).map(|s| s.as_str());
            if action == Some("install") {
                match WhatsAppBot::check_login_status(&config.session_db).await? {
                    Some(jid) => info!("Verified active session: {jid}"),
                    None => {
                        eprintln!("❌ Cannot install service: WhatsApp session is not paired!");
                        eprintln!("👉 Run 'emaki login' first to authenticate.");
                        std::process::exit(1);
                    }
                }
            }
            ServiceManager::handle_command(action)?;
        }
        "daemon" | "start" => {
            let is_foreground = args.iter().any(|a| a == "--foreground" || a == "-f");

            // Verify login status first!
            match WhatsAppBot::check_login_status(&config.session_db).await? {
                Some(jid) => {
                    info!("Verified active WhatsApp session: {jid}");
                }
                None => {
                    eprintln!(
                        "❌ No active WhatsApp session found in {:?}",
                        config.session_db.display()
                    );
                    eprintln!("👉 Run 'emaki login' first to link your WhatsApp account before starting the daemon!");
                    std::process::exit(1);
                }
            }

            if is_foreground {
                info!("Running in foreground daemon mode...");
                bot.start_daemon().await?;
            } else {
                DaemonManager::start_background()?;
            }
        }
        "run" => {
            match WhatsAppBot::check_login_status(&config.session_db).await? {
                Some(jid) => {
                    info!("Verified active WhatsApp session: {jid}");
                }
                None => {
                    eprintln!(
                        "❌ No active WhatsApp session found in {:?}",
                        config.session_db.display()
                    );
                    eprintln!("👉 Run 'emaki login' first to link your WhatsApp account before starting the daemon!");
                    std::process::exit(1);
                }
            }
            info!("Running in foreground daemon mode...");
            bot.start_daemon().await?;
        }
        "stop" => {
            DaemonManager::stop()?;
        }
        "logs" => {
            DaemonManager::logs()?;
        }
        "completion" => {
            let shell = args.get(2).map(|s| s.as_str()).unwrap_or("bash");
            let script = CompletionGenerator::generate(shell)?;
            print!("{script}");
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        unknown => {
            eprintln!("Unknown command: {unknown}\n");
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn print_status(config: &Config) -> Result<()> {
    println!("🔍 Checking Emaki Session & Cache Status...");
    println!("───────────────────────────────────────────");

    match WhatsAppBot::check_login_status(&config.session_db).await? {
        Some(jid) => {
            println!("  ✅ WhatsApp Session : Active (Linked Device: {jid})");
            println!("  📁 Database Path    : {:?}", config.session_db.display());
        }
        None => {
            println!("  ❌ WhatsApp Session : Not Paired / Missing");
            println!("  👉 Run 'emaki login' in your terminal to link WhatsApp!");
        }
    }

    match DaemonManager::get_running_pid() {
        Some(pid) => {
            println!("  🤖 Daemon Process   : Running in background (PID: {pid})");
        }
        None => {
            println!("  🤖 Daemon Process   : Not running in background");
        }
    }

    let cache_size = compute_dir_size(&config.cache_dir).await;
    println!(
        "  🗄️ Cache Directory  : {:?} ({} MB / {} GB limit)",
        config.cache_dir.display(),
        cache_size / (1024 * 1024),
        config.max_cache_size_gb
    );
    println!("───────────────────────────────────────────");
    Ok(())
}

async fn compute_dir_size(dir: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(mut rd) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            if let Ok(meta) = e.metadata().await {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }
    total
}

async fn handle_config_command(args: &[String], config: &mut Config) -> Result<()> {
    let active_path = Config::active_config_path();
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("show");

    match sub {
        "show" | "list" => {
            println!("⚙️  Emaki Configuration");
            println!("───────────────────────────────────────────");
            println!("  📄 Config File       : {:?}", active_path.display());
            println!("  📁 Session Database  : {:?}", config.session_db.display());
            println!("  🗄️ Cache Directory   : {:?}", config.cache_dir.display());
            println!("  📦 Max Cache Size    : {} GB", config.max_cache_size_gb);
            println!("  🎥 Max File Size     : {} MB", config.max_file_size_mb);
            println!("  🎬 Caption Prefix    : \"{}\"", config.caption_prefix);
            println!("  📂 Temp Directory    : {:?}", config.temp_dir.display());
            println!(
                "  🔒 Whitelisted Groups: {}",
                if config.whitelist_groups.is_empty() {
                    "All groups allowed (unrestricted)".to_string()
                } else {
                    format!("{} group(s)", config.whitelist_groups.len())
                }
            );
            println!("───────────────────────────────────────────");
        }
        "set" => {
            let key = args.get(3).map(|s| s.as_str());
            let val = args.get(4).map(|s| s.as_str());
            match (key, val) {
                (Some(k), Some(v)) => {
                    config.set_value(k, v)?;
                    config.save_to_file(&active_path)?;
                    println!("✅ Set '{k}' = '{v}' in {:?}", active_path.display());
                }
                _ => {
                    eprintln!("Usage: emaki config set <key> <value>");
                    eprintln!("Valid keys: max_file_size_mb, max_cache_size_gb, caption_prefix, temp_dir, cache_dir, session_db");
                    std::process::exit(1);
                }
            }
        }
        unknown => {
            eprintln!("Unknown config action: '{unknown}'. Use 'emaki config' or 'emaki config set <key> <value>'");
            std::process::exit(1);
        }
    }
    Ok(())
}

async fn handle_whitelist_command(args: &[String], config: &mut Config) -> Result<()> {
    let active_path = Config::active_config_path();
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("list");

    match sub {
        "list" | "show" => {
            println!("🔒 WhatsApp Group Whitelist");
            println!("───────────────────────────────────────────");
            if config.whitelist_groups.is_empty() {
                println!("  (No whitelist configured — bot responds in ALL groups where added)");
            } else {
                for (i, jid) in config.whitelist_groups.iter().enumerate() {
                    println!("  {}. {jid}", i + 1);
                }
            }
            println!("───────────────────────────────────────────");
        }
        "add" => match args.get(3).map(|s| s.as_str()) {
            Some(j) => {
                if config.add_whitelist_group(j.to_string()) {
                    config.save_to_file(&active_path)?;
                    println!("✅ Added '{j}' to whitelist in {:?}", active_path.display());
                } else {
                    println!("ℹ️  Group '{j}' is already in the whitelist");
                }
            }
            None => {
                eprintln!("Usage: emaki whitelist add <group_jid>");
                std::process::exit(1);
            }
        },
        "remove" | "rm" => match args.get(3).map(|s| s.as_str()) {
            Some(j) => {
                if config.remove_whitelist_group(j) {
                    config.save_to_file(&active_path)?;
                    println!(
                        "✅ Removed '{j}' from whitelist in {:?}",
                        active_path.display()
                    );
                } else {
                    eprintln!("❌ Group '{j}' not found in whitelist");
                }
            }
            None => {
                eprintln!("Usage: emaki whitelist remove <group_jid>");
                std::process::exit(1);
            }
        },
        unknown => {
            eprintln!("Unknown whitelist action: '{unknown}'. Use: emaki whitelist [list | add <jid> | remove <jid>]");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn print_help() {
    println!(
        r#"Usage: emaki [COMMAND] [OPTIONS]

Commands:
  login                 Start interactive QR pairing to authenticate with WhatsApp
  daemon [--foreground] Start the reel daemon (runs in background by default; use -f for foreground)
  stop                  Stop the background daemon
  logs                  Follow live background daemon logs
  status                Check login credentials, background daemon state, and cache
  config [show|set]     View or update configuration settings (e.g. max_file_size_mb)
  whitelist [list|add|remove]
                        Manage whitelisted WhatsApp group JIDs
  service [install|status|logs|restart|stop|start|uninstall]
                        Manage 24/7 background systemd service
  help                  Display this help menu

Examples:
  emaki login                                   # Scan QR code on your terminal
  emaki daemon                                  # Start daemon in background (non-blocking)
  emaki daemon --foreground                     # Run in foreground (debug/systemd)
  emaki stop                                    # Stop the background daemon
  emaki logs                                    # Follow live background logs
  emaki status                                  # Verify session & check if daemon is running
  emaki service install                         # Install & enable 24/7 auto-boot systemd service
  emaki config set max_file_size_mb 200         # Update file size limit
  emaki whitelist add 120363024819283746@g.us   # Allow only this group
  emaki whitelist list                          # List whitelisted groups
"#
    );
}

fn print_banner() {
    println!(
        r#"
  ███████╗███╗   ███╗ █████╗ ██╗  ██╗██╗
  ██╔════╝████╗ ████║██╔══██╗██║ ██╔╝██║
  █████╗  ██╔████╔██║███████║█████╔╝ ██║
  ██╔══╝  ██║╚██╔╝██║██╔══██║██╔═██╗ ██║
  ███████╗██║ ╚═╝ ██║██║  ██║██║  ██╗██║
  ╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝
  絵巻 — WhatsApp Instagram Reel Relay Daemon
"#
    );
}
