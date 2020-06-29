use log::{debug, error, info, trace, warn};
use sprocket::*;

fn main() {
    let application = Application::new("Sandbox");

    logger::init(log::LevelFilter::Trace);
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec4::new(1.0, 1.0, 0.0, 5.0);
    let pos = b.xyz();
    println!("a.norm(): {}", a.norm());
    println!("b.norm(): {}", b.norm());
    println!("Dot: {}", Vec3::dot(&a.norm(), &pos.norm()));
    let v = Vec2::new(1.0, 1.0).norm();
    println!("v: {}", v);
    let b = Vec2::right();
    println!("b: {}", b);
    let other = v + b;
    println!("v: {}", other);

    println!(
        "{}",
        Vec3::new(1.0, 2.0, 0.0).norm() + Vec3::forward()
    );

    info!("Created application {}", application.name());
}
