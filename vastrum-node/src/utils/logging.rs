use std::fs::OpenOptions;
use tracing_subscriber::{EnvFilter, Layer, fmt, filter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging() {
    let log_dir = dirs::data_dir().unwrap().join("vastrum").join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let log_file =
        OpenOptions::new().create(true).append(true).open(log_dir.join("node.log")).unwrap();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_filter(filter::Targets::new()
                    .with_target("vastrum_node::execution", tracing::Level::INFO)),
        )
        .init();
}
