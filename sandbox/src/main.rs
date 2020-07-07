use sprocket::*;
use std::env;
fn main() {
    let path = env::current_exe().unwrap();
    println!("Path: {:?}", path);

    utils::normalize_working_dir();
    env::set_current_dir("../../sandbox").expect("Failed to set working directory");

    let mut application = Application::new("Sandbox");
    info!("Created application {}", application.name());

    logger::init(log::LevelFilter::Trace);

    application.add_window("Sandbox", 800, 600, WindowMode::Windowed);

    application.init_graphics();
    application.run();

    info!("Terminating application");

    // let mut window =
    //     Window::new("Sandbox", 800, 600, WindowMode::FullScreen).expect("Failed to create window");

    // loop {
    //     thread::sleep(Duration::from_millis(500));
    //     window.poll_events();
    // }
}
