use log::{debug, error, info, trace, warn};
use sprocket::*;
fn main() {
    let mut application = Application::new("Sandbox");
    info!("Created application {}", application.name());

    logger::init(log::LevelFilter::Trace);
    Window::init_glfw();
    let window = Window::new("Sandbox", 800, 600, WindowMode::Windowed);
    info!("Window title '{}'", window.title());

    application.add_window(window);
    application.add_window(Window::new("Other Window", 100, 100, WindowMode::Windowed));
    application.run();

    info!("Terminating application");

    // let mut window =
    //     Window::new("Sandbox", 800, 600, WindowMode::FullScreen).expect("Failed to create window");

    // loop {
    //     thread::sleep(Duration::from_millis(500));
    //     window.poll_events();
    // }
}
