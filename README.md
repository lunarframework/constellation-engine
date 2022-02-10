# Constellation Engine

# Dependencies

- `Godot`: The framework this engine is written, used for visualization, ui, etc.
- `Cargo`: Rust package manager used to compile rust gdnative scripts that drives much of the engine.
- `CMake`: C/C++ package manager used to compile c++ code that drives some of the PDE solvers.

# Setup

Currently this must done manually, though I am writing a python script to do this automatically.

- Copy the dealii 9.3 library into `gdnative/engine/engine-sys/third-party`
- Run `cargo build` in `gdnative/engine`
- Copy the dll from `gdnative/engine/target` into `gdnative/lib/platform`

# Design

Constellation engine is based on the concepts of `systems` and `solvers`. 

A `system` contains the data completely specifying a physical situation. This is all that is needed to both simulate and visualize the given situation. For example, in an n-body simulation, this might include the positions, velocities, masses, and temperatures of every star involved in the simulation.

Each system contains the following:
- A collection of subsystems and the required context for those subsystems (position and velocity for example)
- Some set of properties that are read-writable by the current system, but read-only for higher systems
- A solver category

Each solver attached to a system can:
- Manipulate its properites
- Loop through and update the context assigned to each subsystem
- Merge subsystems if they are too close to one another using the subsystem's properties

## Example

Galaxy is a system, containing many types of smaller systems. It contains no extra properties
NBodyStar is a system, which can be added to Galaxy. It only contains a mass.
Its associated in context is velocity and position.

# TODO