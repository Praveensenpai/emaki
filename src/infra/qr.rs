use crate::error::{EmakiError, Result};
use fast_qr::QRBuilder;
use tracing::info;

pub fn render_terminal_qr(code: &str, timeout_secs: u64) -> Result<()> {
    let qr = QRBuilder::new(code)
        .build()
        .map_err(|e| EmakiError::WhatsApp(format!("QR generation failed: {e}")))?;

    let qr_string = qr.to_str();

    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║             SCAN THIS QR CODE IN WHATSAPP                  ║");
    info!("║  (Settings > Linked Devices > Link a Device)               ║");
    info!("║  Valid for: {:<45} ║", format!("{timeout_secs} seconds"));
    info!("╚════════════════════════════════════════════════════════════╝");
    println!("\n{qr_string}\n");

    Ok(())
}
