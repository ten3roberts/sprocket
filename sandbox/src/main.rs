use log::{debug, error, info, trace, warn};
use sprocket::*;

fn main() {
    let application = Application::new("Sandbox");

    logger::init(log::LevelFilter::Trace);
    trace!("Trace message");
    debug!("Debug message");
    info!("Info message");
    warn!("Warning message");
    error!("Error message");
    info!("Created sandbox app");
    info!("Created application {}", application.name());
}
