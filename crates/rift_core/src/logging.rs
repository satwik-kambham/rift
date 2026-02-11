pub fn initialize_tracing() {
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push("rift_logs");

    std::fs::create_dir_all(&tmp_dir).ok();

    let log_path = tmp_dir.join("rift.log");
    std::fs::remove_file(log_path).ok();

    let file_appender = tracing_appender::rolling::never(tmp_dir, "rift.log");
    tracing_subscriber::fmt()
        .with_env_filter("info,tarpc=error")
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .init();
}
