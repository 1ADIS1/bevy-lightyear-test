use bevy::prelude::*;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        LocalAddr,
        server::{NetcodeConfig, ServerUdpIo, Start},
    },
};

use crate::shared::SERVER_ADDR;

pub struct MyServerPlugin;

impl Plugin for MyServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

/// Start the server
fn startup(mut commands: Commands) -> Result {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);

    Ok(())
}
