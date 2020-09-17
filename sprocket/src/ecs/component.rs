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
