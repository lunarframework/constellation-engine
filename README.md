# Constellation Engine
A relativistic simulation engine written in rust.

!["Demo gif"](demo.gif)

# Structure

- `app`: Contains the framework for launching console applications (used by multiple different CLI modes)
- `components`: Sets of shared components used with the starlight ecs to hold data to be simulated or rendered
- `mode`: Code for the various subcommands that can be run.
    - `new`: Creates a new, empty, project folder.
    - `open`: Opens a project folder for editing.
    - `simulate`: Simulates the current project, storing the result in the `simulations` subfolder.
    - `view`: Views a given simulation.
- `project`: Manages project serialization and disk storage
- `render`: All code required to render data, both the ui, and the world.

# TODO