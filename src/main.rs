mod client;
mod server;
mod shared;

use std::time::Duration;

use bevy::prelude::*;
use clap::{Parser, Subcommand};
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};

use crate::{
    client::{MyClientPlugin, Player},
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
        id: usize,
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
    Server,
}

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();

    match cli.mode {
        Mode::Client { id, port } => {
            app.add_plugins((
                DefaultPlugins,
                ClientPlugins {
                    tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
                },
                MyClientPlugin,
            ));

            app.world_mut().spawn(Player { id, port });
        }
        Mode::Server => {
            app.add_plugins((
                MinimalPlugins,
                ServerPlugins {
                    tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
                },
                MyServerPlugin,
            ));
        }
    }

    app.add_plugins(SharedPlugin);

    app.run();
}
