mod client;
mod editor;
mod protocol;
mod server;
mod shared;

use std::time::Duration;

use avian2d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::prelude::*;
use clap::{Parser, Subcommand};
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};

use crate::{
    client::MyClientPlugin,
    editor::EditorPlugin,
    protocol::{CliClientOptions, ProtocolPlugin},
    server::MyServerPlugin,
    shared::{FIXED_TIMESTEP_HZ, SharedPlugin},
};

/// CLI options to create an [`App`]
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    Client {
        #[arg(short, long, default_value_t = 0)]
        id: u64,
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
    Server,
}

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();
    let resolution = (640., 480.).into();

    match cli.mode {
        Mode::Client { id, port: _ } => {
            app.add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Client"),
                        resolution,
                        ..default()
                    }),
                    ..default()
                }),
                ClientPlugins {
                    tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
                },
                MyClientPlugin,
            ));

            app.world_mut().spawn(CliClientOptions { id });
        }
        Mode::Server => {
            // TODO: just minimal plugins?
            app.add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Server"),
                        resolution,
                        ..default()
                    }),
                    ..default()
                }),
                ServerPlugins {
                    tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
                },
                MyServerPlugin,
            ));
        }
    }

    // both client and server need physics
    // (the client also needs the physics plugin to be able to compute predicted bullet hits)
    app.add_plugins((
        PhysicsPlugins::default()
            .build()
            // disable Sync as it is handled by lightyear_avian
            .disable::<avian2d::sync::SyncPlugin>(),
        PhysicsDebugPlugin::default(),
    ))
    .insert_resource(avian2d::prelude::Gravity(Vec2::ZERO));

    app.add_plugins((SharedPlugin, ProtocolPlugin, EditorPlugin));

    app.add_systems(Startup, spawn_camera);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
