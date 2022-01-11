# Constellation Engine
A relativistic simulation engine written in rust.

!["Demo gif"](demo.gif)

# Structure

At the highest level, the application is split into `Rust` and `C++`
The core project is written in rust, but all simulation work is done by calling into the
dynamic library named `spacetime` produced by the build process. Of course, this functionality
is not yet available on non desktop platforms.

- `crates/`: Contains all rust subcrates
- `native/`: Contains all c++ code/dependencies for the engine
- `src/`: Contains the main rust launcher
    - `app/`: Contains the framework for launching console applications (used by multiple different CLI modes)
    - `components/`: Sets of shared components used with the starlight ecs to hold data to be simulated or rendered
    - `mode/`: Code for the various subcommands that can be run.
        - `new/`: Creates a new, empty, project folder.
        - `open/`: Opens a project folder for editing.
        - `simulate/`: Simulates the current project, storing the result in the `simulations` subfolder.
        - `view/`: Views a given simulation.
    - `project/`: Manages project serialization and disk storage.
    - `render/`: All code required to render data, both the ui, and the world.
    - `ui/`: Shared Ui Elements.
- `.gitignore`: Standard git ignore file
- `.gitmodules`: Standard git module file
- `Cargo.toml`: Cargo config file
- `README.md`: ReadMe file.

# TODO