use super::{CommandPool, IndexBuffer, Result, Vertex, VertexBuffer, VkAllocator};
use ash::vk;
use log::info;

/// A mesh contains a vertexbuffer and an indexbuffer
pub struct Mesh {
    vertexbuffer: VertexBuffer,
    indexbuffer: IndexBuffer,
}

impl Mesh {
    /// Creates a new mesh with given vertices and indices
    pub fn new(
        allocator: &VkAllocator,
        device: &ash::Device,
        queue: vk::Queue,
        commandpool: &CommandPool,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<Mesh> {
        let vertexbuffer = VertexBuffer::new(allocator, device, queue, commandpool, vertices)?;
        let indexbuffer = IndexBuffer::new(allocator, device, queue, commandpool, indices)?;

        info!("Created new mesh");

        Ok(Mesh {
            vertexbuffer,
            indexbuffer,
        })
    }

    /// Returns the index buffer
    pub fn indexbuffer(&self) -> &IndexBuffer {
        &&self.indexbuffer
    }

    /// Returns the vertexbuffer
    pub fn vertexbuffer(&self) -> &VertexBuffer {
        &self.vertexbuffer
    }

    /// Returns the number of vertices in the mesh
    /// Equivalent to mesh.vertexbuffer().count()
    pub fn vertex_count(&self) -> u32 {
        self.indexbuffer.count()
    }

    /// Returns the number of indices in the mesh
    /// Equivalent to mesh.indexbuffer().count()
    pub fn index_count(&self) -> u32 {
        self.indexbuffer.count()
    }
}
