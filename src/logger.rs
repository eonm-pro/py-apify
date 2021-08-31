use env_logger;
use log::LevelFilter;

use env_logger::Builder;

pub fn setup_logger() {
    Builder::new().filter(None, LevelFilter::Info).init();
}
