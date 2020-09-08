use super::{Material, Model, Pipeline, RenderPass, Result, Swapchain, Texture, VulkanContext};
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

/// A trait for a resource that can be loaded from a path
///
/// Requires a load function
pub trait Resource {
    /// Trait function used to load a function
    /// Provides resourcemanager for access to e.g other resources and graphics context
    /// Resource loading should not have many sideeffects and produce similar results when loaded several times from the same file
    fn load(resourcemanager: &ResourceManager, path: &str) -> Result<Self>
    where
        Self: Sized;
}

/// Manages a single type of resource
/// Used internally in ResourceManager
/// Should not be used standalone but can be used to assemble your own type of resource manager
pub struct ResourceSystem<T: Resource> {
    resources: RwLock<HashMap<String, Arc<T>>>,
    garbage: Mutex<Vec<Garbage<T>>>,
}

impl<T: Resource> ResourceSystem<T> {
    pub fn new() -> Self {
        ResourceSystem {
            resources: RwLock::new(HashMap::new()),
            garbage: Mutex::new(Vec::new()),
        }
    }

    /// Loads and stores a resource if it doesn't already exist
    /// The resource will be stored as the path name
    /// If a resource with the name already exists, the existing one will be returned
    /// Will wait for write lock of textures
    pub fn load(&self, resourcemanager: &ResourceManager, path: &str) -> Result<Arc<T>> {
        match self.resources.read().unwrap().get(path) {
            Some(resource) => return Ok(Arc::clone(resource)),
            None => {}
        }

        // Load outside match to drop RwLock read guard
        let resource = Arc::new(T::load(resourcemanager, path)?);

        self.resources
            .write()
            .unwrap()
            .insert(path.to_owned(), Arc::clone(&resource));
        Ok(resource)
    }

    /// path to return a reference to an already loaded texture
    /// Returns None if the texture isn't loaded
    pub fn get(&self, path: &str) -> Option<Arc<T>> {
        self.resources
            .read()
            .unwrap()
            .get(path)
            .map(|v| Arc::clone(v))
    }

    /// Goes through the loaded resources and places all resources with no other references in a garbage
    /// The actual resource will get deleted after garbage_cycles cleanup cycles so that it is no longer in use by a pipeline
    pub fn collect_garbage(&self, garbage_cycles: u32) {
        // Acquire a lock for the whole function to avoid having a resource getting a user midway through TOCTOU
        let mut garbage = self.garbage.lock().unwrap();
        let mut resources = self.resources.write().unwrap();

        // Remove garbage with 0 cycles remaining
        garbage.retain(|v| v.cycles_remaining > 0);
        // Remove one cycle from existing garbage
        garbage.iter_mut().for_each(|v| v.cycles_remaining -= 1);

        // Remove all elements with one 1 (self) strong reference and place into garbage
        resources.retain(|_, r| {
            if Arc::strong_count(&r) == 1 {
                garbage.push(Garbage::new(Arc::clone(&r), garbage_cycles));
                false
            } else {
                true
            }
        });
        if garbage.len() > 0 {
            log::debug!("Adding ty {} to garbage", std::any::type_name::<T>())
        }
    }

    pub fn info(&self) -> Vec<ResourceInfo> {
        let ty = std::any::type_name::<T>();
        let ty = &ty[ty.rfind("::").map(|v| v + 2).unwrap_or(0)..];
        self.resources
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| ResourceInfo {
                name: k.to_owned(),
                ty,
                strong_refs: Arc::strong_count(v),
                weak_refs: Arc::weak_count(v),
            })
            .collect()
    }
}

/// Keeps track of loaded resources across threads
/// Automatically reference counts resources and removes no longer used ones with .cleanup()
pub struct ResourceManager {
    context: Arc<VulkanContext>,
    // The current swapchain
    swapchain: RwLock<Option<Arc<Swapchain>>>,
    textures: ResourceSystem<Texture>,
    models: ResourceSystem<Model>,
    renderpasses: ResourceSystem<RenderPass>,
    pipelines: ResourceSystem<Pipeline>,
    materials: ResourceSystem<Material>,
}

impl ResourceManager {
    /// Creates a new resource manager
    /// Should only exist one per application or graphics context
    pub fn new(context: Arc<VulkanContext>) -> Self {
        ResourceManager {
            context,
            textures: ResourceSystem::new(),
            models: ResourceSystem::new(),
            swapchain: RwLock::new(None),
            renderpasses: ResourceSystem::new(),
            pipelines: ResourceSystem::new(),
            materials: ResourceSystem::new(),
        }
    }

    pub fn context(&self) -> &Arc<VulkanContext> {
        &self.context
    }

    /// Sets the current swapchain
    pub fn set_swapchain(&self, swapchain: Arc<Swapchain>) {
        self.swapchain.write().unwrap().replace(swapchain);
    }

    /// Returns the currently set swapchain if any
    pub fn get_swapchain(&self) -> Option<Arc<Swapchain>> {
        self.swapchain
            .read()
            .unwrap()
            .as_ref()
            .map(|val| Arc::clone(val))
    }

    /// Loads and stores a texture if it doesn't already exist
    /// The texture will be stored as the path name
    /// If a texture with the name already exists, the existing one will be returned
    pub fn load_texture(&self, path: &str) -> Result<Arc<Texture>> {
        self.textures.load(&self, path)
    }

    /// path to return a reference to an already loaded texture
    /// Returns None if the texture isn't loaded
    pub fn get_texture(&self, path: &str) -> Option<Arc<Texture>> {
        self.textures.get(path)
    }

    /// Loads and stores a model if it doesn't already exist
    /// The model will be stored as the path name
    /// If a model with the name already exists, the existing one will be returned
    /// Will not block for write access if model is already loaded
    pub fn load_model(&self, path: &str) -> Result<Arc<Model>> {
        self.models.load(self, path)
    }

    /// path to return a reference to an already loaded model
    /// Returns None if the model isn't loaded
    pub fn get_model(&self, path: &str) -> Option<Arc<Model>> {
        self.models.get(path)
    }

    /// path to return a reference to an already loaded model
    /// Returns None if the model isn't loaded
    pub fn get_pipeline(&self, path: &str) -> Option<Arc<Pipeline>> {
        self.pipelines.get(path)
    }

    /// Loads and stores a renderpass from json if it doesn't already exist
    /// The renderpass will be stored as the path name
    /// If a renderpass with the name already exists, the existing one will be returned
    pub fn load_pipeline(&self, path: &str) -> Result<Arc<Pipeline>> {
        self.pipelines.load(&self, path)
    }

    /// Loads and stores a renderpass from json if it doesn't already exist
    /// The renderpass will be stored as the path name
    /// If a renderpass with the name already exists, the existing one will be returned
    pub fn load_renderpass(&self, path: &str) -> Result<Arc<RenderPass>> {
        self.renderpasses.load(self, path)
    }

    /// path to return a reference to an already loaded model
    /// Returns None if the renderpass isn't loaded
    pub fn get_renderpass(&self, path: &str) -> Option<Arc<RenderPass>> {
        self.renderpasses.get(path)
    }

    /// Loads and stores a material from json if it doesn't already exist
    /// The material will be stored as the path name
    /// If a material with the name already exists, the existing one will be returned
    pub fn load_material(&self, path: &str) -> Result<Arc<Material>> {
        self.materials.load(self, path)
    }

    /// path to return a reference to an already loaded model
    /// Returns None if the renderpass isn't loaded
    pub fn get_material(&self, path: &str) -> Option<Arc<Material>> {
        self.materials.get(path)
    }
    /// Will place each resource no longer used in a garbage list
    /// The actual resource will get deleted after garbage_cycles cleanup cycles so that it is no longer in use by a pipeline
    /// Should only be called from one thread to avoid thread blocking
    pub fn collect_garbage(&self, garbage_cycles: u32) {
        self.textures.collect_garbage(garbage_cycles);
        self.models.collect_garbage(garbage_cycles);
        self.renderpasses.collect_garbage(garbage_cycles);
        self.pipelines.collect_garbage(garbage_cycles);
        self.materials.collect_garbage(garbage_cycles);
    }

    pub fn recreate(&self) -> Result<()> {
        let swapchain = self.swapchain.read().unwrap();
        let swapchain = swapchain.as_ref().unwrap();

        let color_format = swapchain.format();
        let depth_format = swapchain.depth_format();

        // Separate into separate scopes to drop Write lock
        {
            let mut renderpasses = self.renderpasses.resources.write().unwrap();

            let new_renderpasses: HashMap<_, _> = renderpasses
                .iter()
                .map(|(k, v)| match v.recreate(color_format, depth_format) {
                    Ok(v) => Ok((k.to_owned(), Arc::new(v))),
                    Err(e) => Err(e),
                })
                .collect::<Result<HashMap<_, _>>>()?;

            // Replace the old with the new
            let _ = std::mem::replace(&mut *renderpasses, new_renderpasses);
        }
        {
            let mut pipelines = self.pipelines.resources.write().unwrap();
            // Now recreate the pipelines
            // They will query self for the renderpasses which are now replaced
            let new_pipelines: HashMap<_, _> = pipelines
                .iter()
                .map(|(k, v)| match v.recreate(&self) {
                    Ok(v) => Ok((k.to_owned(), Arc::new(v))),
                    Err(e) => Err(e),
                })
                .collect::<Result<HashMap<_, _>>>()?;

            let _ = std::mem::replace(&mut *pipelines, new_pipelines);
        }
        {
            let mut materials = self.materials.resources.write().unwrap();
            // Now recreate the pipelines
            // They will query self for the renderpasses which are now replaced
            let new_materials: HashMap<_, _> = materials
                .iter()
                .map(|(k, v)| match v.recreate(&self) {
                    Ok(v) => Ok((k.to_owned(), Arc::new(v))),
                    Err(e) => Err(e),
                })
                .collect::<Result<HashMap<_, _>>>()?;

            let _ = std::mem::replace(&mut *materials, new_materials);
        }
        Ok(())
    }

    /// Returns a descripctive status about the resources currently managed
    pub fn info(&self) -> Vec<ResourceInfo> {
        let mut result = Vec::new();
        result.extend(self.textures.info());
        result.extend(self.models.info());
        result.extend(self.renderpasses.info());
        result.extend(self.pipelines.info());
        result.extend(self.materials.info());

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
