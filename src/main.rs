#[macro_use]
extern crate rocket;
extern crate py_apify_macro;

use py_apify_macro::apify;

mod logger;
use logger::setup_logger;

#[launch]
fn rocket() -> _ {
    setup_logger();
    apify! {"src/*.py"}
}
