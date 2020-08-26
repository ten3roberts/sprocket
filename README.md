# Sprocket
Sprocket is a game engine written in Rust aiming both Windows and Linux

The aim for the engine is to have the user in control. The programming is done directly within Rust using Sprocket as a library.
Currently, not much is implemented

## The plan
The plan for Sprocket is to be a powerful multi-threaded game engine for 2D as well as 3D

### Types
Many types have an additional type in the same module called {TYPE_NAME}Spec which is a serializable, non-context dependable representation of the resource. This is used to serialize and describe types in JSON

### Main features
* Vulkan rendering
* Multithreading with little to no blocking synchronization between different parts
* Configurable, the engine is very low level and should allow you to tamper with the underlying systems
* Scene editor
