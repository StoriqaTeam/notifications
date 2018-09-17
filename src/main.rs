//! Notifications is a microservice responsible for sending notifications to users.
//! This create is for running the service from `notifications_lib`. See `notifications_lib` for details.

extern crate notifications_lib;
extern crate stq_logging;

fn main() {
    let config = notifications_lib::config::Config::new().expect("Can't load app config!");

    // Prepare sentry integration
    let _sentry = notifications_lib::sentry_integration::init(config.sentry.as_ref());

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    notifications_lib::start_server(config, &None, || ());
}
