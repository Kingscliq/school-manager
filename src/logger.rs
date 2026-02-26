use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
pub struct AppLogger;

impl AppLogger {
    pub fn init() {
        // env filters setup
        let env_filters: EnvFilter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        // console filters setup
        let console_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .with_file(false);

        // combine then using the tracing registry

        tracing_subscriber::registry()
            .with(env_filters)
            .with(console_layer)
            .init();
    }

    pub fn info(message: &str) {
        tracing::info!("{}", message);
    }
    // pub fn warn(message: &str) {
    //     tracing::warn!("{}", message);
    // }
    // pub fn debug(message: &str) {
    //     tracing::debug!("{}", message);
    // }
    pub fn error(message: &str) {
        tracing::error!("{}", message);
    }
}
