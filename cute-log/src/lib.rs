use log::LevelFilter;

/// 로깅 기능을 추가. 이때 기본값으로 info 이상의 log 만 지원하도록 함.
pub fn init_logger() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
}
