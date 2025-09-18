//! This module contains the shared code between the client and the server.

use avian2d::math::{AdjustPrecision, Scalar};
use avian2d::prelude::{
    Collider, ColliderOf, Collisions, DebugRender, LinearVelocity, NarrowPhaseSet, PhysicsSchedule,
    RigidBody, Sensor,
};
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::*;

use crate::protocol::{Bullet, Player, PlayerAction, PlayerId, Wall};

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_level)
            .add_systems(FixedUpdate, shoot);

        app.add_systems(
            // Run collision handling after collision detection.
            //
            // NOTE: The collision implementation here is very basic and a bit buggy.
            //       A collide-and-slide algorithm would likely work better.
            PhysicsSchedule,
            kinematic_controller_collisions.in_set(NarrowPhaseSet::Last),
        );
    }
}

fn prepare_level(mut commands: Commands) {
    commands.spawn((
        Wall,
        Collider::rectangle(20., 450.),
        Transform::from_xyz(-250., 0., 0.),
    ));

    commands.spawn((
        Wall,
        Collider::rectangle(20., 450.),
        Transform::from_xyz(250., 0., 0.),
    ));

    commands.spawn((
        Wall,
        Collider::rectangle(600., 20.),
        Transform::from_xyz(0., 200., 0.),
    ));

    commands.spawn((
        Wall,
        Collider::rectangle(600., 20.),
        Transform::from_xyz(0., -200., 0.),
    ));

    commands.spawn((Wall, Transform::from_xyz(0., 100., 0.)));
}

// TODO: make system
pub fn move_player(
    velocity: &mut avian2d::prelude::LinearVelocity,
    action_state: &leafwing_input_manager::prelude::ActionState<PlayerAction>,
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

    direction = direction.normalize_or_zero() * 150.;

    *velocity = LinearVelocity(direction);
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

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system handles collision response for kinematic character controllers
/// by pushing them along their contact normals by the current penetration depth,
/// and applying velocity corrections in order to snap to slopes, slide along walls,
/// and predict collisions using speculative contacts.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<(&RigidBody, Option<&Player>)>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<
        (&mut avian2d::prelude::Position, &mut LinearVelocity),
        (With<RigidBody>, With<Player>),
    >,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                // let my = bodies.get(rb1).unwrap().1;
                character_rb = *bodies.get(rb1).unwrap().0;
                is_other_dynamic = bodies.get(rb2).is_ok_and(|(rb, _)| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap().0;
                is_other_dynamic = bodies.get(rb1).is_ok_and(|(rb, _)| rb.is_dynamic());
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers.
        if !character_rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            // if deepest_penetration > 0.0 {
            //     // The character is intersecting an unclimbable object, like a wall.
            //     // We want the character to slide along the surface, similarly to
            //     // a collide-and-slide algorithm.

            //     // Don't apply an impulse if the character is moving away from the surface.
            //     if linear_velocity.dot(normal) > 0.0 {
            //         continue;
            //     }

            //     // Slide along the surface, rejecting the velocity along the contact normal.
            //     let impulse = linear_velocity.reject_from_normalized(normal);
            //     linear_velocity.0 = impulse;
            // }
            // else {
            // The character is not yet intersecting the other object,
            // but the narrow phase detected a speculative collision.

            // We need to push back the part of the velocity
            // that would cause penetration within the next frame.

            let normal_speed = linear_velocity.dot(normal);

            // Don't apply an impulse if the character is moving away from the surface or staying still.
            if normal_speed >= 0.0 {
                continue;
            }

            // Compute the impulse to apply.
            let impulse_magnitude =
                normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
            let impulse = impulse_magnitude * normal;

            linear_velocity.0 -= impulse;
            // }
        }
    }
}
