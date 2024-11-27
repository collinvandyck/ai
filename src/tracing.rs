pub fn init() {
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();
}
