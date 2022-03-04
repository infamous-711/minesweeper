use bevy::{input::system::exit_on_esc_system, prelude::*};
use board_plugin::{resources::BoardOptions, BoardPlugin};

#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();

    // window setup
    app.insert_resource(WindowDescriptor {
        title: "Mine Sweeper!".to_string(),
        width: 700.,
        height: 800.,
        ..Default::default()
    });

    app.add_plugins(DefaultPlugins); // Bevy default plugins

    app.add_startup_system(camera_setup); // setup cameras

    app.add_system(exit_on_esc_system); // exit when escape key is pressed

    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.add_plugin(BoardPlugin);

    // Board plugin options
    app.insert_resource(BoardOptions {
        map_size: (20, 20),
        bomb_count: 40,
        tile_padding: 3.0,
        ..Default::default()
    });

    // run the game
    app.run();
}

fn camera_setup(mut cmds: Commands) {
    // 2d orthographic camera
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());
}
