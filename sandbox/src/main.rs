use log::{debug, info};
use sprocket::*;

fn main() {
    let application = Application::new("Sandbox");

    logger::init(log::LevelFilter::Info);
    debug!("Trace message");
    info!("Created sandbox app");
    info!("Created application {}", application.name());
}
