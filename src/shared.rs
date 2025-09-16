//! This module contains the shared code between the client and the server.

use avian2d::prelude::{Collider, DebugRender, LinearVelocity, RigidBody};
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::protocol::{Bullet, Player, PlayerAction, PlayerId};

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, shoot);
    }
}

// TODO: make system
pub fn move_player(
    transform: &mut Transform,
    action_state: &leafwing_input_manager::prelude::ActionState<PlayerAction>,
    delta: f32,
) {
    let mut direction = Vec2::ZERO;

    if action_state.pressed(&PlayerAction::Up) {
        direction.y = 1.;
    }
    if action_state.pressed(&PlayerAction::Down) {
        direction.y = -1.;
    }
    if action_state.pressed(&PlayerAction::Right) {
        direction.x = 1.;
    }
    if action_state.pressed(&PlayerAction::Left) {
        direction.x = -1.;
    }

    direction = direction.normalize_or_zero();

    transform.translation += direction.extend(0.) * 225. * delta;
}

/// This system runs on both the client and the server, and is used to shoot a bullet
/// The bullet is shot from the predicted player on the client, and from the server-entity on the server.
/// When the bullet is replicated from server to client, it will use the existing client bullet with the `PreSpawned` component
/// as its `Predicted` entity
fn shoot(
    mut commands: Commands,
    player_q: Query<
        (
            &PlayerId,
            &Transform,
            &leafwing_input_manager::prelude::ActionState<PlayerAction>,
            Option<&ControlledBy>,
        ),
        (Or<(With<Predicted>, With<Replicate>)>, With<Player>),
    >,
    asset_server: Res<AssetServer>,
) {
    for (player_id, player_transform, action_state, controlled_by) in player_q.iter() {
        if action_state.just_pressed(&PlayerAction::Shoot) {
            let is_server = controlled_by.is_some();
            let salt = player_id.0.to_bits();

            let bullet_bundle = (
                Name::new("Bullet"),
                Bullet,
                *player_id,
                Collider::circle(50.),
                DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
                Sprite {
                    image: asset_server.load("art/ball.png"),
                    ..default()
                },
                RigidBody::Kinematic,
                LinearVelocity(Vec2::new(20., 0.)),
                Transform {
                    translation: player_transform.translation,
                    scale: Vec3::splat(0.1),
                    ..default()
                },
            );

            // on the server, replicate the bullet
            if is_server {
                commands.spawn((
                    bullet_bundle,
                    // NOTE: the PreSpawned component indicates that the entity will be spawned on both client and server
                    //  but the server will take authority as soon as the client receives the entity
                    //  it does this by matching with the client entity that has the same hash
                    //  The hash is computed automatically in PostUpdate from the entity's components + spawn tick
                    //  unless you set the hash manually before PostUpdate to a value of your choice
                    //
                    // the default hashing algorithm uses the tick and component list. in order to disambiguate
                    // between the two bullets, we add additional information to the hash.
                    // NOTE: if you don't add the salt, the 'left' bullet on the server might get matched with the
                    // 'right' bullet on the client, and vice versa. This is not critical, but it will cause a rollback
                    PreSpawned::default_with_salt(salt),
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(player_id.0)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(player_id.0)),
                    *controlled_by.unwrap(),
                ));
            } else {
                // on the client, just spawn the ball
                // NOTE: the PreSpawned component indicates that the entity will be spawned on both client and server
                //  but the server will take authority as soon as the client receives the entity
                commands.spawn((bullet_bundle, PreSpawned::default_with_salt(salt)));
            }
        }
    }
}
