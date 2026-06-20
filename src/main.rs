mod errors;
mod models;
mod service;

use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Sets up dual logging: pretty logs to stdout, JSON lines to logs/app.log.
/// The returned guard must stay alive for the duration of the program, or
/// the background writer thread is dropped and buffered logs are lost.
fn init_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    std::fs::create_dir_all("logs")?;
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = fmt::layer().with_target(false);
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

/// Routes panics (e.g. unwraps, indexing failures) through tracing so they land
/// in logs/app.log instead of only printing to stderr and disappearing.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".to_string());

        error!(panic.location = %location, panic.message = %message, "process panicked");
    }));
}

fn main() -> anyhow::Result<()> {
    let _guard = init_logging()?;
    install_panic_hook();

    info!("starting billing reconciliation batch");

    let rates = service::region_tax_rates();
    let invoices = service::sample_invoices();

    let mut succeeded = 0;
    let mut failed = 0;

    for invoice in &invoices {
        match service::calculate_total(invoice, &rates) {
            Ok(total) => {
                succeeded += 1;
                info!(invoice_id = invoice.id, total, "invoice reconciled");
            }
            Err(err) => {
                failed += 1;
                warn!(invoice_id = invoice.id, error = %err, "invoice skipped due to validation error");
            }
        }
    }

    info!(succeeded, failed, total = invoices.len(), "batch complete");

    Ok(())
}
