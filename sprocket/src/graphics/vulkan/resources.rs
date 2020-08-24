use super::{super::SWAPCHAIN_IMAGE_COUNT, Model, Result, Texture, VulkanContext};
use ash::version::DeviceV1_0;
use log::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

/// Represents a resource soon to be deleted
struct Garbage<T> {
    resource: Arc<T>,
    cycles_remaining: u32,
}

impl<T> Garbage<T> {
    pub fn new(resource: Arc<T>, cycles_remaining: u32) -> Self {
        Garbage {
            resource,
            cycles_remaining,
        }
    }
}

/// A stringed representation of a resource
/// Used for getting the status and info of the resource manager
#[derive(Debug)]
pub struct ResourceInfo {
    name: String,
    ty: &'static str,
    strong_refs: usize,
    weak_refs: usize,
}

/// Keeps track of loaded resources across threads
/// Automatically reference counts resources and removes no longer used ones with .cleanup()
pub struct ResourceManager {
    context: Arc<VulkanContext>,
    textures: RwLock<HashMap<String, Arc<Texture>>>,
    texture_garbage: Mutex<Vec<Garbage<Texture>>>,

    models: RwLock<HashMap<String, Arc<Model>>>,
    model_garbage: Mutex<Vec<Garbage<Model>>>,
}

impl ResourceManager {
    /// Creates a new resource manager
    /// Should only exist one per application or graphics context
    pub fn new(context: Arc<VulkanContext>) -> Self {
        ResourceManager {
            context,
            textures: RwLock::new(HashMap::new()),
            texture_garbage: Mutex::new(Vec::new()),
            models: RwLock::new(HashMap::new()),
            model_garbage: Mutex::new(Vec::new()),
        }
    }

    /// Loads and stores a texture if it doesn't already exist
    /// The texture will be stored as the path name
    /// If a texture with the name already exists, the existing one will be returned
    /// Will wait for write lock of textures
    pub fn load_texture(&self, path: &str) -> Result<Arc<Texture>> {
        Ok(Arc::clone(
            self.textures
                .write()
                .unwrap()
                .entry(path.to_owned())
                .or_insert(Arc::new(Texture::load(
                    &self.context.allocator,
                    &self.context.device,
                    self.context.graphics_queue,
                    self.context.generic_pool(),
                    path,
                )?)),
        ))
    }

    /// path to return a reference to an already loaded texture
    /// Returns None if the texture isn't loaded
    pub fn get_texture(&self, path: &str) -> Option<Arc<Texture>> {
        self.textures
            .read()
            .unwrap()
            .get(path)
            .map(|v| Arc::clone(v))
    }

    /// Loads and stores a model if it doesn't already exist
    /// The model will be stored as the path name
    /// If a model with the name already exists, the existing one will be returned
    /// Will wait for write lock of textures
    /// Will not block for write access if model is already loaded
    pub fn load_model(&self, path: &str) -> Result<Arc<Model>> {
        match self.models.read().unwrap().get(path) {
            Some(model) => return Ok(Arc::clone(model)),
            None => {}
        }

        // Load outside match to drop RwLock read guard
        let model = Arc::new(Model::load(
            path,
            &self.context.allocator,
            &self.context.device,
            self.context.graphics_queue,
            self.context.generic_pool(),
        )?);

        self.models
            .write()
            .unwrap()
            .insert(path.to_owned(), Arc::clone(&model));
        Ok(model)
    }

    /// path to return a reference to an already loaded model
    /// Returns None if the model isn't loaded
    pub fn get_model(&self, path: &str) -> Option<Arc<Model>> {
        self.models.read().unwrap().get(path).map(|v| Arc::clone(v))
    }

    /// Will place each resource no longer used in a cleanup stack
    /// The actual resource will get deleted after SWAPCHAIN_IMAGE_COUNT cleanup cycles so that it is no longer in use by a pipeline
    /// Should only be called from one thread to avoid thread blocking
    pub fn cleanup(&self) {
        self.cleanup_textures();
        self.cleanup_models();
    }

    fn cleanup_textures(&self) {
        // Acquire a lock for the whole function to avoid having a resource getting a user midway through TOCTOU
        let mut garbage = self.texture_garbage.lock().unwrap();
        let mut textures = self.textures.write().unwrap();

        // Remove garbage with 0 cycles remaining
        garbage.retain(|v| v.cycles_remaining > 0);
        // Remove one cycle from existing garbage
        garbage.iter_mut().for_each(|v| v.cycles_remaining -= 1);

        // Remove all elements with one 1 (self) strong reference and place into garbage
        textures.retain(|_, r| {
            if Arc::strong_count(&r) == 1 {
                garbage.push(Garbage::new(Arc::clone(&r), SWAPCHAIN_IMAGE_COUNT));
                false
            } else {
                true
            }
        });
    }

    fn cleanup_models(&self) {
        // Acquire a lock for the whole function to avoid having a resource getting a user midway through TOCTOU
        let mut garbage = self.model_garbage.lock().unwrap();
        let mut models = self.models.write().unwrap();

        // Remove garbage with 0 cycles remaining
        garbage.retain(|v| v.cycles_remaining > 0);
        // Remove one cycle from existing garbage
        garbage.iter_mut().for_each(|v| v.cycles_remaining -= 1);

        // Remove all elements with one 1 (self) strong reference and place into garbage
        models.retain(|_, r| {
            if Arc::strong_count(&r) == 1 {
                garbage.push(Garbage::new(Arc::clone(&r), SWAPCHAIN_IMAGE_COUNT));
                false
            } else {
                true
            }
        });
    }
    /// Returns a descripctive status about the resources currently managed
    pub fn info(&self) -> Vec<ResourceInfo> {
        let mut result = Vec::new();
        // Textures
        result.extend(
            self.textures
                .read()
                .unwrap()
                .iter()
                .map(|(k, r)| ResourceInfo {
                    name: k.to_owned(),
                    ty: "Texture",
                    strong_refs: Arc::strong_count(r),
                    weak_refs: Arc::weak_count(r),
                }),
        );

        result.extend(
            self.models
                .read()
                .unwrap()
                .iter()
                .map(|(k, r)| ResourceInfo {
                    name: k.to_owned(),
                    ty: "Model",
                    strong_refs: Arc::strong_count(r),
                    weak_refs: Arc::weak_count(r),
                }),
        );
        result
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        info!("Dropping resource manager");
        unsafe { self.context.device.device_wait_idle().unwrap() }
        // Drop all other values
    }
}
