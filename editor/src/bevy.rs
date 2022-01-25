mod star;

use star::StarPlugin;

use bevy::prelude::*;

fn main() {
    App::new()
        // Configuration
        .insert_resource(WindowDescriptor {
            title: "Constellation Engine".to_string(),
            width: 800.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(StarPlugin)
        .run();
}
