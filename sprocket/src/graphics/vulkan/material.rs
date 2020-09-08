use super::{
    resources::Resource, DescriptorPool, DescriptorSet, DescriptorType, Error, Pipeline,
    ResourceManager, Result, Sampler, Texture,
};

use ash::vk;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct MaterialSpec {
    pipeline: String,
    textures: Vec<String>,
    // TODO coming features
    // color: Color,
    // reflectivity: f32,
    // smoothness: f32,
}

pub struct Material {
    pipeline: Arc<Pipeline>,
    textures: Vec<Arc<Texture>>,
    samplers: Vec<Arc<Sampler>>,
    descriptor_sets: Vec<DescriptorSet>,
    /// May be removed and replaced with descriptor pool management
    descriptor_pool: DescriptorPool,
    spec: MaterialSpec,
}

impl Resource for Material {
    fn load(resourcemanager: &ResourceManager, path: &str) -> Result<Self> {
        let spec = serde_json::from_str(&ex::fs::read_to_string(path)?)?;
        Self::new(spec, resourcemanager)
    }
}

impl Material {
    pub fn new(spec: MaterialSpec, resourcemanager: &ResourceManager) -> Result<Self> {
        let pipeline = resourcemanager.load_pipeline(&spec.pipeline)?;

        let textures: Vec<Arc<Texture>> = spec
            .textures
            .iter()
            .map(|tex| resourcemanager.load_texture(tex))
            .collect::<Result<_>>()?;

        let context = resourcemanager.context();
        let swapchain = resourcemanager.get_swapchain().unwrap();

        // Create the second per material descriptor set
        let per_material_layout = match pipeline.set_layouts().get(1) {
            Some(v) => v,
            None => return Err(Error::MissingDescriptorSet(1)),
        };

        let descriptor_pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: (per_material_layout
                    .spec()
                    .bindings
                    .iter()
                    .filter(|binding| binding.ty == DescriptorType::CombinedImageSampler)
                    .count()
                    * swapchain.image_count()) as u32,
            },
            // vk::DescriptorPoolSize {
            //     ty: vk::DescriptorType::UNIFORM_BUFFER,
            //     descriptor_count: per_material_layout
            //         .spec()
            //         .bindings
            //         .iter()
            //         .filter(|binding| binding.ty == DescriptorType::UniformBuffer)
            //         .count() as u32,
            // },
        ];

        // Create pool for this material only
        // Will be changed when implementing descriptor pool management
        let descriptor_pool = DescriptorPool::new(
            &context.device,
            &descriptor_pool_sizes,
            swapchain.image_count() as u32,
        )?;

        let descriptor_sets = DescriptorSet::new(
            &context.device,
            &descriptor_pool,
            &per_material_layout,
            swapchain.image_count() as u32,
        )?;

        let samplers = vec![Arc::new(Sampler::new(&context.device)?)];

        // Write the per material descriptor set with the textures
        DescriptorSet::write(
            &context.device,
            &descriptor_sets,
            &per_material_layout.spec(),
            [].iter(),
            textures.iter().cycle(),
            samplers.iter().cycle(),
        )?;

        Ok(Material {
            pipeline,
            textures,
            samplers,
            descriptor_sets,
            descriptor_pool,
            spec,
        })
    }

    pub fn pipeline(&self) -> &Arc<Pipeline> {
        &self.pipeline
    }

    pub fn textures(&self) -> &[Arc<Texture>] {
        &self.textures
    }

    /// Returns the per material descriptor sets for each swapchain image
    pub fn descriptor_sets(&self) -> &[DescriptorSet] {
        &self.descriptor_sets[..]
    }

    pub fn spec(&self) -> &MaterialSpec {
        &self.spec
    }

    /// Returns self created again from spec but with updated values
    /// Called when swapchain is recreated
    pub fn recreate(&self, resourcemanager: &super::ResourceManager) -> Result<Self> {
        Self::new(self.spec.clone(), resourcemanager)
    }
}
