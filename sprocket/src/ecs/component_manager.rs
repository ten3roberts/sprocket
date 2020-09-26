use std::collections::HashMap;

use super::component::*;
use super::component_array::*;
use super::entity::*;

type DynComponentArray = Box<dyn IComponentArray>;

/// Manages any type of component array for entities
/// Does dynamic dispatch on arrays of components
/// For more specific cases where only a fixed amount of types are used, prefer using raw
/// ComponentArray
pub struct ComponentManager {
    /// A map of dynamically dispatched ComponentArray
    component_arrays: HashMap<ComponentType, DynComponentArray>,
    insert_functions: HashMap<ComponentType, Box<dyn Fn(&mut DynComponentArray, ComponentUpdate)>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        ComponentManager {
            component_arrays: HashMap::new(),
            insert_functions: HashMap::new(),
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

        self.insert_functions.insert(
            ty,
            Box::new(|array, update| Self::insert_component_update::<T>(array, update)),
        );
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

// Message handling functions
impl ComponentManager {
    /// Receives and handles a component update
    pub fn on_component_update(&mut self, component_update: ComponentUpdate) -> Option<()> {
        let func = self.insert_functions.get(&component_update.ty())?;
        let component_array = self.component_arrays.get_mut(&component_update.ty())?;
        (func)(component_array, component_update);
        Some(())
    }

    /// Inserts components as raw bytes
    /// Panics
    /// If T is not the same as the internal type of component_update
    /// If component is not registered
    fn insert_component_update<T: 'static>(
        component_array: &mut DynComponentArray,
        component_update: ComponentUpdate,
    ) {
        let component_array = unsafe {
            &mut *(component_array.as_mut() as *mut dyn IComponentArray as *mut ComponentArray<T>)
        };
        let components: Vec<(Entity, T)> = component_update.into();
        component_array.insert_components(components);
    }
}
