# Entity Component System

## Component Manager
A component manager manages several component arrays
It associates the ComponentType with a dynamically dispatched ComponentArray
Any component can be inserted and it is inserted based on its type into the correct ComponentArray

The ComponentManager is thus merely a wrapper over dynamically dispatched ComponentArrays
Specialized systems like Renderer don't use ComponentManager in favor of a raw statically dispatched ComponentArrays
This is because the types of components can be known at compile time
There will always be a central ComponentManager that stores all components registered for all entities
## Component Array
A component array stores a contiguous array of components of the same time
The components are internally statically dispatched
The component array is itself dynamically dispatched within the ComponentManager (The only dynamic dispatch)

The order of the components is unspecified but each component is associated to an entity
The association is stored in a map
## Entity Manager
Entity manager is responsible of creating and handing out entity IDs
The entity manager is the only way to construct an Entity handle

## Signature
Each entity has a signature which is a bitset of all components an entity has
The maximum size of a bitset is a constant that also determined the maximum number of types of components

The bit that a component sets can be determined by `ComponentManager.get_component_bit::<T>()`.

It is internally determined by the order in which components where registered

When an entity changes its signature, I.e; a new component is inserted, each system with the new signature gets the entity added


## System
A system stores a list of a copy of all components it needs.
It only stores entities which match in signatures, I.e; it has all components a system requires.

## System Manager
Exists one per application
Handles synchronization between systems

Distributes changes to systems that have subscribed to the component

Each registered system has a signature
When a component that matches a systems signature is changed, the changed component and entity id is sent across the channel as a ComponentUpdate event
When an entity get a component added and changes its signature, each matching system gets all matching components and entity id sent across the channel as a EntityAdded event
This is because components do not know about entities that do not match the signature of the system. I.e; the renderer will only have a collection of entities which have components required to draw an entity, like Transform, Material, and Mesh. It won't store any other entities, thus, when an entity gets all those required components, all those need to be sent to the renderer
## Synchronization
When an component is inserted in a ComponentArray a dirty flag is set for that component

At the end of each systems tick, it creates a list containing all dirty components and serializes them into one Vec<u8>
For ComponentManager, this creates a list of serialized changed components

Each group of changed component types is then sent across a channel to the application's SystemManager
