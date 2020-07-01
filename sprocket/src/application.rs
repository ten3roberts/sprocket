use crate::event::Event;
use crate::graphics;
use crate::graphics::window::{Window, WindowMode};
use log::{debug, error, info, trace, warn};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Application {
    name: String,
    windows: Vec<Window>,
    event_receiver: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    graphics_context: Option<graphics::GraphicsContext>,
}

impl Application {
    /// Creates a new blank application with the given name
    pub fn new(name: &str) -> Application {
        let (event_sender, event_receiver) = mpsc::channel::<Event>();

        Application {
            name: String::from(name),
            windows: Vec::new(),
            event_receiver,
            event_sender,
            graphics_context: None,
        }
    }

    pub fn init(&mut self) {
        Window::init_glfw();
        self.graphics_context = match graphics::init(graphics::Api::Vulkan) {
            Ok(context) => Some(context),
            Err(msg) => {
                error!("Failed to initialize graphics {}", msg);
                None
            }
        };
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
                if let Event::MousePosition(_, _) = event {
                } else {
                    info!("Event: {:?}", event);
                }
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
