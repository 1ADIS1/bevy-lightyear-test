use bevy::prelude::*;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        input::native::{ActionState, InputMarker},
        server::{ClientOf, NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};

use crate::{
    client::{Player, PlayerPosition},
    protocol::Direction,
    shared::{SERVER_ADDR, SERVER_REPLICATION_INTERVAL},
};

pub struct MyServerPlugin;

impl Plugin for MyServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_new_client)
            .add_observer(handle_connected)
            .add_systems(Startup, startup)
            .add_systems(FixedUpdate, handle_player_movement);
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

pub(crate) fn handle_new_client(trigger: Trigger<OnAdd, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.target()).insert((
        ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ),
        Name::from("Client"),
    ));
}

pub(crate) fn handle_connected(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok(client_id) = query.get(trigger.target()) else {
        return;
    };

    let client_id = client_id.0;

    let entity = commands
        .spawn((
            Player,
            Sprite {
                image: asset_server.load("art/ball.png"),
                ..default()
            },
            // we replicate the Player entity to all clients that are connected to this server
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            ControlledBy {
                owner: trigger.target(),
                lifetime: Default::default(),
            },
        ))
        .id();

    info!(
        "Create player entity {:?} for client {:?}",
        entity, client_id
    );
}

/// Read client inputs and move players in server therefore giving a basis for other clients
pub fn handle_player_movement(
    mut position_query: Query<(&mut Transform, &mut PlayerPosition, &ActionState<Direction>)>,
    time: Res<Time>,
) {
    for (mut transform, mut position, inputs) in position_query.iter_mut() {
        crate::shared::move_player(
            (&mut transform, &mut position, &inputs.0),
            time.delta_secs(),
        );
    }
}
