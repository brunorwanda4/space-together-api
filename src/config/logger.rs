use env_logger::Env;
use std::env;

/// Initialize logging for the application.
///
/// Usage:
/// ```rust
/// config::logger::init();
/// ```
pub fn init() {
    // Set default log level if RUST_LOG is not already set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug,actix_web=debug,mongodb=info");
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("✅ Logger initialized. Logs are now active.");
}
