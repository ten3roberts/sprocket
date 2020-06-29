use log::{debug, error, info, trace, warn};
use sprocket::*;
use std::thread;
use std::time::Duration;
fn main() {
    let application = Application::new("Sandbox");
    info!("Created application {}", application.name());

    logger::init(log::LevelFilter::Trace);
    Window::init_glfw();
    let window = Window::new("Sandbox", 800, 600);
    info!("Window title '{}'", window.title());
    loop {
        window.process_events();
        thread::sleep(Duration::from_millis(200));
    }
    // let mut window =
    //     Window::new("Sandbox", 800, 600, WindowMode::FullScreen).expect("Failed to create window");

    // loop {
    //     thread::sleep(Duration::from_millis(500));
    //     window.poll_events();
    // }
}
