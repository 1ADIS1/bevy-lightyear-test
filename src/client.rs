use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::{
    input::client::InputSet,
    netcode::{Key, NetcodeClient},
    prelude::{
        client::NetcodeConfig,
        input::native::{ActionState, InputMarker},
        *,
    },
};

use crate::{
    protocol::{ClientId, Direction, Player},
    shared::{self, SERVER_ADDR},
};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (setup,)).add_systems(
            FixedPreUpdate,
            // Inputs have to be buffered in the WriteClientInputs set
            buffer_input.in_set(InputSet::WriteClientInputs),
        );

        app.add_systems(FixedUpdate, player_movement);

        app.add_observer(on_player_connect);

        app.add_observer(on_other_player_interpolate);
    }
}

fn setup(mut commands: Commands, client_added_q: Query<(Entity, &ClientId), Added<ClientId>>) {
    for (client_entity, client_id) in client_added_q.iter() {
        let auth = Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id: client_id.0,
            private_key: Key::default(),
            protocol_id: 0,
        };

        let clien_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

        commands.entity(client_entity).insert((
            Name::new(format!("Netcode client {}", client_id.0)),
            Client::default(),
            LocalAddr(clien_address),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            PredictionManager::default(),
            InterpolationManager::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ));

        commands.trigger_targets(Connect, client_entity);
    }
}

pub(crate) fn buffer_input(
    mut query: Query<&mut ActionState<Direction>, With<InputMarker<Direction>>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut action_state) = query.single_mut() {
        let mut direction = Direction {
            up: false,
            down: false,
            left: false,
            right: false,
        };

        if keypress.pressed(KeyCode::KeyW) || keypress.pressed(KeyCode::ArrowUp) {
            direction.up = true;
        }
        if keypress.pressed(KeyCode::KeyS) || keypress.pressed(KeyCode::ArrowDown) {
            direction.down = true;
        }
        if keypress.pressed(KeyCode::KeyA) || keypress.pressed(KeyCode::ArrowLeft) {
            direction.left = true;
        }
        if keypress.pressed(KeyCode::KeyD) || keypress.pressed(KeyCode::ArrowRight) {
            direction.right = true;
        }

        // we always set the value. Setting it to None means that the input was missing, it's not the same
        // as saying that the input was 'no keys pressed'
        // action_state.value = direction;
        action_state.0 = direction;
    }
}

/// The client input only gets applied to predicted entities that we own
/// This works because we only predict the user's controlled entity.
/// If we were predicting more entities, we would have to only apply movement to the player owned one.
fn player_movement(
    // timeline: Single<&LocalTimeline>,
    mut position_query: Query<(&mut Transform, &ActionState<Direction>), With<Predicted>>,
    time: Res<Time>,
) {
    // let tick = timeline.tick();
    for (mut transform, input) in position_query.iter_mut() {
        // trace!(?tick, ?position, ?input, "client");
        // NOTE: be careful to directly pass Mut<PlayerPosition>
        // getting a mutable reference triggers change detection, unless you use `as_deref_mut()`
        shared::move_player((&mut transform, &input.0), time.delta_secs());
    }
}

/// We should manipulate only a predicted copy of the player.
/// NOTE: this is called twice
fn on_player_connect(
    trigger: Trigger<OnAdd, (Player, Predicted)>,
    player_q: Query<Entity, With<Predicted>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if player_q.get(trigger.target()).is_err() {
        return;
    }

    commands.entity(trigger.target()).insert((
        Sprite {
            image: asset_server.load("art/ball.png"),
            ..default()
        },
        InputMarker::<Direction>::default(),
    ));

    warn!("Player predicted!");
}

/// Change other interpolated players.
/// These players positions are smoothly interpolated, so that is why we should be displaying only them.
fn on_other_player_interpolate(
    trigger: Trigger<OnAdd, Player>,
    player_q: Query<Entity, With<Interpolated>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if player_q.get(trigger.target()).is_err() {
        return;
    }

    commands.entity(trigger.target()).insert((Sprite {
        image: asset_server.load("art/ball.png"),
        ..default()
    },));

    warn!("Player replicated!");
}
