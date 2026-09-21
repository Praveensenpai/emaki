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

    print_banner();

    let config = Config::load_or_default("emaki.toml");
    info!("Using session database: {:?}", config.session_db.display());

    let bot = WhatsAppBot::new(config);
    bot.start().await?;

    Ok(())
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
