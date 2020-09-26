use super::Entity;
use std::any::TypeId;

/// The type of a component type id
/// Is determined by the pointer to std::any::type_id
/// Should not be serialized to file or sent over network as it may differ
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct ComponentType(TypeId);

impl ComponentType {
    pub fn get<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

impl std::fmt::Debug for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Represents an update of component values
/// Stores the components internally as a Vec<u8> and can therefore be stored along with different
/// types
/// Ability to be converted into a list of concrete types
/// When converting with update.into::<T>(), T needs to be the same type as it was created with, or
/// else it will panic!
/// Use try_into to not panic!
pub struct ComponentUpdate {
    ty: ComponentType,
    data: Vec<u8>,
}

impl ComponentUpdate {
    pub fn new<T: 'static>(components: Vec<(Entity, T)>) -> Self {
        Self {
            ty: ComponentType::get::<T>(),
            data: unsafe { std::mem::transmute::<Vec<(Entity, T)>, Vec<u8>>(components) },
        }
    }

    pub fn ty(&self) -> ComponentType {
        self.ty
    }

    /// Does a conversion to a concrete type
    /// Returns None if the type converted into is not the same as the type it was created with
    /// Consumes self
    pub fn try_into<T: 'static>(self) -> Option<Vec<(Entity, T)>> {
        if ComponentType::get::<T>() != self.ty {
            return None;
        }

        Some(unsafe { std::mem::transmute::<Vec<u8>, Vec<(Entity, T)>>(self.data) })
    }
}

/// Converts a ComponentUpdate into a Vec of a concrete type
impl<T: 'static> From<ComponentUpdate> for Vec<(Entity, T)> {
    fn from(components: ComponentUpdate) -> Self {
        if ComponentType::get::<T>() != components.ty {
            panic!("Attempt to convert ComponentUpdate into mismatched concrete type. Expected type {:?}. Actual type {:?}", components.ty, ComponentType::get::<T>());
        }

        unsafe { std::mem::transmute::<Vec<u8>, Vec<(Entity, T)>>(components.data) }
    }
}
