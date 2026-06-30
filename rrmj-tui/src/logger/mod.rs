use tracing_subscriber::EnvFilter;

use crate::settings::Settings;

pub fn init(settings: &Settings) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(settings.log_level()))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
