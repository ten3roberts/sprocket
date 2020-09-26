use super::component::ComponentType;
use super::entity::Entity;
use std::{collections::HashMap, ops::Deref, ops::DerefMut};

/// Interface for the generic concrete ComponentArray
pub trait IComponentArray {
    fn component_type(&self) -> ComponentType;
}

/// Represents an array that holds a components of type T associated to entities
/// Doesn't need to be a component but can also be a tuple or struct grouping several, commonly
/// together used components together
/// Can deref and deref_mut to a slice
pub struct ComponentArray<T: 'static> {
    /// Maps an entity id to and index in the array
    /// This is necessary since the array is not sparse
    entity_map: HashMap<Entity, usize>,
    /// A non-sparse list of components, index does not map to entity id
    components: Vec<T>,
}

impl<T: 'static> ComponentArray<T> {
    /// Creates a new empty component array
    pub fn new() -> Self {
        Self {
            entity_map: HashMap::new(),
            components: Vec::new(),
        }
    }

    /// Creates a new empty component array with specified capacity of entities
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entity_map: HashMap::with_capacity(capacity),
            components: Vec::with_capacity(capacity),
        }
    }

    /// Returns a mutable component for an entity
    /// Returns None if component doesn't exist for entity
    pub fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = self.entity_map.get(&entity)?;
        Some(&mut self.components[*index])
    }

    /// Returns a component for an entity
    /// Returns None if component doesn't exist for entity
    pub fn get_component(&self, entity: Entity) -> Option<&T> {
        let index = self.entity_map.get(&entity)?;
        Some(&self.components[*index])
    }

    /// Inserts a component for entity
    /// If a component already exists for the entity, it is replaced and returned
    pub fn insert_component(&mut self, entity: Entity, component: T) -> Option<T> {
        // Component already exists; replace
        if let Some(index) = self.entity_map.get(&entity) {
            Some(std::mem::replace(&mut self.components[*index], component))
        }
        // New component
        else {
            let component_index = self.components.len();
            self.components.push(component);
            self.entity_map.insert(entity, component_index);
            None
        }
    }

    /// Inserts several components at once
    pub fn insert_components(&mut self, components: Vec<(Entity, T)>) {
        components.into_iter().for_each(|(entity, component)| {
            self.insert_component(entity, component);
            return ();
        });
    }

    /// Removes and returns (if any) a component associated to entity
    /// Returns None if component doesn't exist for entity
    pub fn remove_component(&mut self, entity: Entity) -> Option<T> {
        if let Some(index) = self.entity_map.remove(&entity) {
            Some(self.components.remove(index))
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a ComponentArray<T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.iter()
    }
}

impl<T> Deref for ComponentArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.components[..]
    }
}

impl<T> DerefMut for ComponentArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components[..]
    }
}

impl<T> IComponentArray for ComponentArray<T> {
    fn component_type(&self) -> ComponentType {
        ComponentType::get::<T>()
    }
}
