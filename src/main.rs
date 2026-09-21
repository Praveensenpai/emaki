mod config;
mod domain;
mod error;
mod infra;

use config::Config;
use error::Result;
use infra::WhatsAppBot;
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

    print_banner();

    let config = Config::load_or_default("emaki.toml");
    let bot = WhatsAppBot::new(config.clone());

    match command {
        "login" => {
            info!("Running in interactive login mode...");
            bot.start_login().await?;
        }
        "status" => {
            print_status(&config).await?;
        }
        "daemon" | "run" => {
            info!("Running in background daemon mode...");
            bot.start_daemon().await?;
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

fn print_help() {
    println!(
        r#"Usage: emaki [COMMAND]

Commands:
  login     Start interactive QR pairing to authenticate with WhatsApp
  daemon    Start the background reel relay daemon (verifies session first)
  status    Check login credentials status and 5GB cache storage usage
  help      Display this help menu

Examples:
  emaki login      # Scan QR code on your local terminal
  emaki daemon     # Run as a daemon (systemd or foreground)
  emaki status     # Check if session credentials are valid
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
