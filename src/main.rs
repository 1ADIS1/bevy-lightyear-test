// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

fn main() {
    App::new()
        .add_plugins((
            // Embedd assets folder into binary file.
            EmbeddedAssetPlugin {
                mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(
                    // Comment this if you're not using pixel art.
                    // This sets image filtering to nearest
                    // This is done to prevent textures with low resolution (e.g. pixel art) from being blurred
                    // by linear filtering.
                    ImagePlugin::default_nearest(),
                )
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy project template".into(),
                        // mode: bevy::window::WindowMode::Fullscreen(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::Main)
                .load_collection::<GameAssets>(),
        )
        .add_systems(Startup, (spawn_2d_camera,))
        .run();
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "art/ball.png")]
    _ball: Handle<Image>,
}

#[derive(States, Clone, Copy, Eq, PartialEq, Debug, Hash, Default, Reflect)]
enum AppState {
    #[default]
    AssetLoading,
    Main,
}

fn spawn_2d_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
