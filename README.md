# Constellation Engine
A relativistic simulation engine written in rust.

!["Demo gif"](demo.gif)

# Structure

- `base/`: Rust library of all shared functionality between the engine and the viewer.
- `crates/`: All local rust dependencies (that are part of the `constellation` framework).
- `editor/`: The editor application which provides a visual interface for viewing and editing a project.
- `native/`: All c++ code/dependencies for the engine
- `src/`: Main engine. This is a rust cli tool that is used to create, edit, and simulate projects. And is the primary component of the `constellation` ecosystem.

# TODO