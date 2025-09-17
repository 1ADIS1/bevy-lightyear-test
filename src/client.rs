use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient},
    prelude::{client::NetcodeConfig, *},
};

use crate::{
    protocol::{CliClientOptions, Player, PlayerAction},
    shared::{self, SERVER_ADDR},
};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (setup,));

        app.add_systems(FixedUpdate, (player_movement,));

        app.add_observer(on_predicted_player_connect);

        app.add_observer(on_interpolated_player_spawn);

        app.add_observer(add_ball_physics);
    }
}

fn setup(
    mut commands: Commands,
    client_added_q: Query<(Entity, &CliClientOptions), Added<CliClientOptions>>,
) {
    for (client_entity, client_id) in client_added_q.iter() {
        let auth = Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id: client_id.id,
            private_key: Key::default(),
            protocol_id: 0,
        };

        let clien_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

        commands.entity(client_entity).insert((
            Name::new(format!("Netcode client {}", client_id.id)),
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

/// Blueprint pattern: when the ball gets replicated from the server, add all the components
/// that we need that are not replicated.
/// (for example physical properties that are constant, so they don't need to be networked)
///
/// We only add the physical properties on the ball that is displayed on screen (i.e the Predicted ball)
/// We want the ball to be rigid so that when players collide with it, they bounce off.
///
/// However we remove the Position because we want the balls position to be interpolated, without being computed/updated
/// by the physics engine? Actually this shouldn't matter because we run interpolation in PostUpdate...
fn add_ball_physics(
    trigger: Trigger<OnAdd, crate::protocol::Ball>,
    mut commands: Commands,
    ball_query: Query<(), With<Predicted>>,
) {
    if let Ok(()) = ball_query.get(trigger.target()) {
        commands
            .entity(trigger.target())
            .insert(crate::protocol::Ball::get_physics_bundle());
    }
}

/// The client input only gets applied to predicted entities that we own
/// This works because we only predict the user's controlled entity.
/// If we were predicting more entities, we would have to only apply movement to the player owned one.
fn player_movement(
    // timeline: Single<&LocalTimeline>,
    mut position_query: Query<
        (
            &mut avian2d::prelude::LinearVelocity,
            &leafwing_input_manager::prelude::ActionState<PlayerAction>,
        ),
        With<Predicted>,
    >,
) {
    // let tick = timeline.tick();
    for (mut velocity, input) in position_query.iter_mut() {
        // trace!(?tick, ?position, ?input, "client");
        // NOTE: be careful to directly pass Mut<PlayerPosition>
        // getting a mutable reference triggers change detection, unless you use `as_deref_mut()`
        shared::move_player(&mut velocity, input);
    }
}

/// Predicted - is our player.
///
/// We should manipulate only a predicted copy of the player.
/// NOTE: this is called twice
fn on_predicted_player_connect(
    trigger: Trigger<OnAdd, (Player, Predicted)>,
    player_q: Query<Entity, (With<Predicted>, With<Player>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if player_q.get(trigger.target()).is_err() {
        return;
    }

    let mut input_map = leafwing_input_manager::prelude::InputMap::new([
        (PlayerAction::Up, KeyCode::KeyW),
        (PlayerAction::Down, KeyCode::KeyS),
        (PlayerAction::Right, KeyCode::KeyD),
        (PlayerAction::Left, KeyCode::KeyA),
    ]);

    input_map.insert(PlayerAction::Shoot, KeyCode::Space);

    commands.entity(trigger.target()).insert((
        Sprite {
            image: asset_server.load("art/ball.png"),
            ..default()
        },
        input_map,
        Player::get_physics_bundle(),
    ));

    warn!("Predicted player spawned!");
}

/// Interpolated - are other players.
///
/// These players positions are smoothly interpolated, so that is why we should be displaying only them.
fn on_interpolated_player_spawn(
    trigger: Trigger<OnAdd, Player>,
    player_q: Query<Entity, (With<Interpolated>, With<Player>)>,
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
        Player::get_physics_bundle(),
    ));

    warn!("Interpolated player spawned!");
}
