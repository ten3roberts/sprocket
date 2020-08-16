use super::{CommandPool, Error, Mesh, Result, Vertex, VkAllocator};
use crate::math::*;
use ash::{self, vk};
use ex::fs;
use std::{collections::HashMap, path::Path};
pub struct Model {
    meshes: HashMap<String, Mesh>,
}

impl Model {
    // Loads a model from a collada file into meshes
    pub fn load<P: AsRef<Path> + std::fmt::Display>(
        path: P,
        allocator: &VkAllocator,
        device: &ash::Device,
        queue: vk::Queue,
        commandpool: &CommandPool,
    ) -> Result<Model> {
        let root = simple_xml::from_string(&fs::read_to_string(path)?)?;
        let lib_geometries = &root.try_get_nodes("library_geometries")?[0];
        let mut meshes = HashMap::new();

        let asset = &root.try_get_nodes("asset")?[0];
        let up_axis = &asset.try_get_nodes("up_axis")?[0];
        let axis_transform = match &up_axis.content[..] {
            "Z_UP" => |v: Vec3| Vec3::new(v.x, -v.z, -v.y),
            "Y_UP" => |v: Vec3| v,
            "X_UP" => |v: Vec3| Vec3::new(v.y, v.x, v.z),
            _ => {
                log::warn!("Unrecognized up axis '{}'", up_axis.content);
                |v: Vec3| v
            }
        };

        for geometry in lib_geometries.try_get_nodes("geometry")?.iter() {
            let (name, mesh) = parse_collada_geometry(
                geometry,
                allocator,
                device,
                queue,
                commandpool,
                axis_transform,
            )?;
            meshes.insert(name, mesh);
        }

        Ok(Model { meshes })
    }

    pub fn get_mesh_index(&self, index: usize) -> Option<&Mesh> {
        self.meshes.iter().skip(index).next().map(|(_, v)| v)
    }
}

// Parses a single mesh/geometry from a collada xml structure
fn parse_collada_geometry(
    node: &simple_xml::Node,

    allocator: &VkAllocator,
    device: &ash::Device,
    queue: vk::Queue,
    commandpool: &CommandPool,
    axis_transform: fn(Vec3) -> Vec3,
) -> Result<(String, Mesh)> {
    let name = node.try_get_attribute("name")?;
    let id = node.try_get_attribute("id")?;
    let mesh = &node.try_get_nodes("mesh")?[0];
    let sources = mesh.try_get_nodes("source")?;

    let source_positions = id.to_owned() + "-positions";
    let source_normals = id.to_owned() + "-normals";
    let source_map_0 = id.to_owned() + "-map-0";

    // Create new empty array
    let mut positions: Vec<f32> = Vec::new();
    let mut _normals: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();

    // Parse all positions, normals and uvs
    for source in sources {
        let source_id = source.try_get_attribute("id")?;
        let array = &source.try_get_nodes("float_array")?[0];

        if source_id == &source_positions {
            positions = parse_xml_array(&array, None)?;
        } else if source_id == &source_normals {
            _normals = parse_xml_array(&array, None)?;
        } else if source_id == &source_map_0 {
            uvs = parse_xml_array(&array, None)?;
        }
    }

    let triangles = &mesh.try_get_nodes("triangles")?[0];

    // Parse the triangles/indices
    let triangles_list: Vec<usize> = parse_xml_array(
        &triangles.try_get_nodes("p")?[0],
        Some(
            triangles
                .try_get_attribute("count")?
                .parse::<usize>()
                .map_err(|_| Error::ParseError)?,
        ),
    )?;

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut vertex_map: HashMap<(usize, usize, usize), usize> = HashMap::new();

    for i in (0..triangles_list.len()).step_by(3) {
        let pos = triangles_list[i];
        let normal = triangles_list[i + 1];
        let uv = triangles_list[i + 2];

        indices.push(match vertex_map.get(&(pos, normal, uv)) {
            Some(i) => {
                // log::info!("Reusing vertex {},{}", pos, uv);
                *i as u32
            }
            // Create new vertex and add to map
            // Push the new node index to the indeices
            None => {
                vertices.push(Vertex {
                    // Get the correct vertex from the positions array
                    // Correctly transform
                    position: axis_transform(array_to_vec3(&positions, pos)),
                    uv: array_to_vec2(&uvs, uv),
                });
                vertex_map.insert((pos, normal, uv), vertices.len() - 1);
                (vertices.len() - 1) as u32
            }
        });
    }

    Mesh::new(allocator, device, queue, commandpool, &vertices, &indices)
        .map(|mesh| (name.to_owned(), mesh))
}

/// Creates a vector from 3 elements in an array of floats
/// The index is the nth 3 size vector
fn array_to_vec3(array: &[f32], index: usize) -> Vec3 {
    let array = &array[(index * 3)..];
    Vec3 {
        x: array[0],
        y: array[1],
        z: array[2],
    }
}

fn array_to_vec2(array: &[f32], index: usize) -> Vec2 {
    let array = &array[(index * 2)..];
    Vec2 {
        x: array[0],
        y: array[1],
    }
}

fn parse_xml_array<T: std::str::FromStr>(
    node: &simple_xml::Node,
    count: Option<usize>,
) -> Result<Vec<T>> {
    let count = match count {
        Some(c) => c,
        None => node
            .try_get_attribute("count")?
            .parse::<usize>()
            .map_err(|_| Error::ParseError)?,
    };

    let mut result = Vec::with_capacity(count);

    for val in node.content.split(" ") {
        result.push(val.parse::<T>().map_err(|_| {
            log::error!("'{}'", val);
            Error::ParseError
        })?);
    }

    Ok(result)
}
