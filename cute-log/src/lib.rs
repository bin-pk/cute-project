use log::LevelFilter;

pub fn init_logger() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    /*
    let logfile = log4rs::append::file::FileAppender::builder().
        encoder(Box::new(log4rs::encode::pattern::PatternEncoder::default())).build(log_path).unwrap();

    log4rs::init_config(log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("logfile",Box::new(logfile)))
        .build(log4rs::config::Root::builder().appender("logfile").build(LevelFilter::Info)).unwrap()
    ).unwrap();
    */
}
