use super::env_parser::Config;

pub fn setup_logs(config: &Config) {
    // let file_appender = tracing_appender::rolling::hourly(&config.logs, "server_logs,txt");
    // let (_non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    if config.testcase != "optimize" {
        tracing_subscriber::fmt()
            // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
            .with_writer(std::io::stderr)
            // .with_writer(non_blocking)
            .init();
    }
}
