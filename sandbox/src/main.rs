use log::{debug, error, info, trace, warn};
use sprocket::*;

fn main() {
    let application = Application::new("Sandbox");

    logger::init(log::LevelFilter::Trace);
    let a = math::Vec3::new(1.0, 0.0, 0.0);
    let b = math::Vec3::new(1.0, 1.0, 0.0);

    println!("a.norm(): {}", a.norm());
    println!("b.norm(): {}", b.norm());
    println!("Dot: {}", math::Vec3::dot(&a.norm(), &b.norm()));

    info!("Created application {}", application.name());
}
