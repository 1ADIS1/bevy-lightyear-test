use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use avian2d::prelude::{Collider, DebugRender};
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
    protocol::{ClientId, Player, PlayerAction},
    shared::{self, SERVER_ADDR},
};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (setup, shoot)).add_systems(
            FixedPreUpdate,
            // Inputs have to be buffered in the WriteClientInputs set
            buffer_input.in_set(InputSet::WriteClientInputs),
        );

        app.add_systems(FixedUpdate, (player_movement, move_bullet));

        app.add_observer(on_predicted_player_connect);

        app.add_observer(on_interpolated_player_spawn);
    }
}

#[derive(Component)]
pub struct Bullet;

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
    mut query: Query<&mut ActionState<PlayerAction>, With<InputMarker<PlayerAction>>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut action_state) = query.single_mut() {
        let mut direction = PlayerAction {
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
    mut position_query: Query<(&mut Transform, &ActionState<PlayerAction>), With<Predicted>>,
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

/// Predicted - is our player.
///
/// We should manipulate only a predicted copy of the player.
///
/// NOTE: this is called twice
fn on_predicted_player_connect(
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
        InputMarker::<PlayerAction>::default(),
    ));

    warn!("Predicted player spawned!");
}

/// Interpolated - are other players.
///
/// These players positions are smoothly interpolated, so that is why we should be displaying only them.
fn on_interpolated_player_spawn(
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

    warn!("Interpolated player spawned!");
}

fn shoot(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    player_q: Query<&Transform, (With<Predicted>, With<Player>)>,
    asset_server: Res<AssetServer>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    commands.spawn((
        Name::new("Bullet"),
        Bullet,
        Collider::circle(50.),
        DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
        Sprite {
            image: asset_server.load("art/ball.png"),
            ..default()
        },
        Transform {
            translation: player_transform.translation,
            scale: Vec3::splat(0.1),
            ..default()
        },
    ));
}

fn move_bullet(mut bullet_q: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut transform in bullet_q.iter_mut() {
        transform.translation.x += 10. * time.delta_secs();
    }
}
