use crate::event::Event;
use crate::graphics;
use crate::graphics::window::{Window, WindowMode};
use graphics::vulkan::renderer::Renderer;
use log::{error, info};
use std::sync::mpsc;

pub struct Application {
    name: String,
    windows: Vec<Window>,
    event_receiver: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    graphics_context: Option<graphics::GraphicsContext>,
    renderer: Option<Renderer>,
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
            graphics_context: None,
            renderer: None,
        }
    }

    pub fn init_graphics(&mut self) {
        self.graphics_context = match graphics::init(graphics::Api::Vulkan, &self.windows[0]) {
            Ok(context) => Some(context),
            Err(msg) => {
                error!("Failed to initialize graphics '{}'", msg);
                None
            }
        };

        // Create vulkan renderer if vulkan
        if let graphics::GraphicsContext::Vulkan(context) = self.graphics_context.as_ref().unwrap()
        {
            self.renderer = match Renderer::new(context.clone()) {
                Ok(renderer) => Some(renderer),
                Err(e) => {
                    error!("Failed to create renderer '{}'", e);
                    None
                }
            };
        } else {
        }
    }

    pub fn add_window(&mut self, title: &str, width: i32, height: i32, mode: WindowMode) {
        let window = Window::new(title, width, height, mode, self.event_sender.clone());
        self.windows.push(window);
    }

    pub fn run(&mut self) {
        while !self.windows.is_empty() {
            // Process each window for events
            self.windows
                .iter()
                .for_each(|window| window.process_events());

            if let Some(renderer) = &mut self.renderer {
                renderer.draw_frame();
            }

            // Receive and handle events
            while let Ok(event) = self.event_receiver.try_recv() {
                if let Event::MousePosition(_, _) = event {
                } else {
                    info!("Event: {:?}", event);
                }
            }
            self.windows.retain(|window| !window.should_close());
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
