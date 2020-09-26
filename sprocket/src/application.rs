use crate::ecs::*;
use crate::math::*;
use crate::physics::Transform;
use crate::{event::Event, graphics};
use crate::{
    graphics::window::{Window, WindowMode},
    Time, Timer,
};

use graphics::vulkan::{renderer::Renderer, ResourceManager};
use log::{error, info};

use std::{
    sync::{mpsc, Arc},
    time,
};

pub struct Application {
    name: String,
    windows: Vec<Window>,
    event_receiver: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    renderer: Option<Renderer>,
    graphics_context: Option<graphics::GraphicsContext>,
    resource_manager: Option<Arc<ResourceManager>>,
    component_manager: ComponentManager,
    entity_manager: EntityManager,
    time: Time,
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
            renderer: None,
            graphics_context: None,
            resource_manager: None,
            component_manager: ComponentManager::new(),
            entity_manager: EntityManager::new(),
            time: Time::new(),
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
            self.resource_manager = Some(Arc::new(ResourceManager::new(Arc::clone(context))));
            self.renderer = match Renderer::new(
                Arc::clone(context),
                &self.windows[0],
                Arc::clone(&self.resource_manager.as_ref().unwrap()),
            ) {
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
        let mut garbage_timer = Timer::with_target(time::Duration::from_secs(2));
        let mut timer = Timer::with_target(time::Duration::from_secs(5));

        // Create some entities
        let entity = self.entity_manager.create_entity();
        let entity2 = self.entity_manager.create_entity();
        self.component_manager
            .insert_component(entity, Transform::new(Vec3::zero()));

        let renderer = self.renderer.as_mut().unwrap();

        while !self.windows.is_empty() {
            renderer.insert_entity(
                entity,
                Transform::new(Vec3::new(0.0, self.time.elapsed_f32().sin(), 0.0)),
            );
            renderer.insert_entity(
                entity2,
                Transform::new(Vec3::new(self.time.elapsed_f32().sin() * 3.0, 2.0, -4.0)),
            );

            if garbage_timer.signaled() {
                self.resource_manager.as_ref().unwrap().collect_garbage(5); // Change to swapchain.image_count() in renderer system
                garbage_timer.restart();
            }
            if timer.signaled() {
                info!(
                    "Frame: {}, elapsed: {}, delta: {}, fr: {}, us: {}",
                    self.time.framecount(),
                    self.time.elapsed_f32(),
                    self.time.delta_f32(),
                    self.time.framerate(),
                    self.time.delta_us(),
                );
                info!(
                    "Resources: {:?}",
                    self.resource_manager.as_ref().unwrap().info()
                );
                timer.restart();
            }
            // Process each window for events
            self.windows
                .iter()
                .for_each(|window| window.process_events());

            renderer.draw_frame(&self.windows[0], &self.time);

            // Receive and handle events
            while let Ok(event) = self.event_receiver.try_recv() {
                if let Event::MousePosition(_, _) = event {
                } else {
                    info!("Event: {:?}", event);
                }
            }
            self.windows.retain(|window| !window.should_close());
            self.time.update();
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.resource_manager = None;
        Window::terminate_glfw();
    }
}
