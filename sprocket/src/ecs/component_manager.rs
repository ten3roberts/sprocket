use std::collections::HashMap;

use super::component::*;
use super::component_array::*;
use super::entity::*;
// use serde::{Deserialize, Serialize};

// /// Represents updated values for a type of components
// pub struct ComponentUpdate {
//     ty: ComponentType,
//     count: usize,
//     data: Vec<u8>,
// }

// impl ComponentUpdate {
//     /// Creates a new component update from passed components and serializes their value
//     /// Encodes the type of components
//     pub fn new<T: 'static + Serialize + Deserialize>(components: &[(Entity, T)]) -> Result<Self> {
//         Ok(Self {
//             ty: ComponentType::get::<T>(),
//             count: components.len(),
//             data: bincode::serialize(components)?,
//         })
//     }
// }

/// Manages any type of component array for entities
/// Does dynamic dispatch on arrays of components
/// For more specific cases where only a fixed amount of types are used, prefer using raw
/// ComponentArray
pub struct ComponentManager {
    /// A map of dynamically dispatched ComponentArray
    component_arrays: HashMap<ComponentType, Box<dyn IComponentArray>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        ComponentManager {
            component_arrays: HashMap::new(),
        }
    }

    /// Registers a new component type T and creates appropriate array to store them in
    /// Does nothing if T is already registered
    pub fn register_component<T: 'static>(&mut self) {
        let ty = ComponentType::get::<T>();
        if let Some(_) = self.component_arrays.get(&ty) {
            return;
        }

        self.component_arrays
            .insert(ty, Box::new(ComponentArray::<T>::new()));
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let component_array = self.component_array::<T>()?;
        component_array.get_component(entity)
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let component_array = self.component_array_mut::<T>()?;
        component_array.get_component_mut(entity)
    }

    pub fn insert_component<T: 'static>(&mut self, entity: Entity, component: T) -> Option<T> {
        let component_array = self.component_array_mut::<T>()?;
        component_array.insert_component(entity, component)
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        let component_array = self.component_array_mut::<T>()?;
        component_array.remove_component(entity)
    }

    //     /// Processes the events that have happened since last time, like mutation, insertion, and
    //     /// removal
    //     /// Generates a list contaning a list of changed components for each component type
    //     pub fn process_events(&mut self) -> Vec<>> {}

    fn component_array<T: 'static>(&self) -> Option<&ComponentArray<T>> {
        let ty = ComponentType::get::<T>();

        if let Some(component_array) = self.component_arrays.get(&ty) {
            let component_array = unsafe {
                &*(component_array.as_ref() as *const dyn IComponentArray
                    as *const ComponentArray<T>)
            };
            Some(component_array)
        } else {
            return None;
        }
    }

    fn component_array_mut<T: 'static>(&mut self) -> Option<&mut ComponentArray<T>> {
        let ty = ComponentType::get::<T>();

        if let Some(component_array) = self.component_arrays.get_mut(&ty) {
            let component_array = unsafe {
                &mut *(component_array.as_mut() as *mut dyn IComponentArray
                    as *mut ComponentArray<T>)
            };
            Some(component_array)
        } else {
            return None;
        }
    }
}
