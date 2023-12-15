use tracing_subscriber::EnvFilter;

pub fn init_logger() {
    let log_level = EnvFilter::from_env("LOG_LEVEL");

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
}
