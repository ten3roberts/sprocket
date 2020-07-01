use crate::event::Event;
use crate::graphics::window::{Window, WindowMode};
use log::info;
use log::warn;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Application {
    name: String,
    windows: Vec<Window>,
    event_receiver: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
}

impl Application {
    /// Creates a new blank application with the given name
    pub fn new(name: &str) -> Application {
        let (event_sender, event_receiver) = mpsc::channel::<Event>();

        Window::init_glfw();
        Application {
            name: String::from(name),
            windows: Vec::new(),
            event_receiver,
            event_sender,
        }
    }

    pub fn add_window(&mut self, title: &str, width: i32, height: i32, mode: WindowMode) {
        let window = Window::new(title, width, height, mode, self.event_sender.clone());
        self.windows.push(window);
    }

    pub fn run(&mut self) {
        while self.windows.len() > 0 {
            // Process each window for events
            self.windows
                .iter()
                .for_each(|window| window.process_events());

            self.windows.retain(|window| !window.should_close());

            // Receive and handle events
            while let Ok(event) = self.event_receiver.try_recv() {
                info!("Event: {:?}", event);
            }

            thread::sleep(Duration::from_millis(200));
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        Window::terminate_glfw();
    }
}
