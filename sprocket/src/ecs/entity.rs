#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]

pub struct Entity(usize);

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity {}", self.0)
    }
}

/// Converts an entity id to the underlying index type
impl From<Entity> for usize {
    fn from(e: Entity) -> Self {
        e.0
    }
}

/// Handles and distributes entity IDs
/// Does not keep track of the components associated to entities. That is the job of
/// ComponentManager
pub struct EntityManager {
    alive_count: usize,

    /// A list of recently freed entity IDs that are available for use
    /// If this list is empty, all freed spots are taken and new IDs can be taken numerically
    /// Empty also means there are no holes in the IDs, and new IDs will be alive_count+1
    free_ids: Vec<Entity>,
}

impl EntityManager {
    /// Creates a new entity manager
    /// Should only exist one in the whole application
    /// Threads should not have their own entity managers
    /// This is to ensure the distributed IDs across threads always are valid
    /// Rather, entity creation should be a "command" that is later executed at the application
    /// level
    pub fn new() -> Self {
        Self {
            alive_count: 0,
            free_ids: Vec::new(),
        }
    }

    /// Create a new entity
    /// Returns the new entity id
    /// Entities currently do not have names
    /// May be implemented with an Info component (to keep things consistent)
    pub fn create_entity(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            log::debug!("Reusing entity id {}", id);
            id
        } else {
            let id = self.alive_count;
            log::debug!("Creating new entity id {}", id);
            self.alive_count += 1;
            Entity(id)
        }
    }

    /// Destroys an entity
    /// Entity should not be used afterwards
    pub fn destroy_entity(&mut self, entity: Entity) {
        if entity.0 >= self.alive_count {
            log::error!("Invalid entity handle {}", entity);
            return;
        }

        log::debug!("Destroying entity with id {}", entity);
        self.free_ids.push(entity)
    }
}
