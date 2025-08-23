use bevy::prelude::*;
// use bevy_egui::EguiPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use haalka::HaalkaPlugin;

pub mod card;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // EguiPlugin::default(),
            // WorldInspectorPlugin::new(),
            // HaalkaPlugin,
        ))
        .add_plugins(card::plugin)
        .add_systems(Startup, spawn_camera)
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
