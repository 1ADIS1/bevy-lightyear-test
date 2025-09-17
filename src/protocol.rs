use avian2d::prelude::{AngularVelocity, Collider, ColliderDensity, LinearVelocity, RigidBody};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<(PlayerAction, PlayerId)>();

        app.add_plugins(input::leafwing::InputPlugin::<PlayerAction> {
            config: input::InputConfig::<PlayerAction> {
                // enable lag compensation; the input messages sent to the server will include the
                // interpolation delay of that client
                lag_compensation: true,
                ..default()
            },
        });

        // app.register_component::<Transform>()
        //     .add_prediction(PredictionMode::Full)
        //     .add_interpolation(InterpolationMode::Full)
        //     .add_interpolation_fn(transform_interpolation_fn);

        app.register_component::<Player>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<RigidBody>()
            .add_prediction(PredictionMode::Once);

        app.register_component::<Bullet>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<Ball>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<avian2d::prelude::Position>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(position_should_rollback)
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn()
            .add_linear_correction_fn();

        app.register_component::<avian2d::prelude::Rotation>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(rotation_should_rollback)
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn()
            .add_linear_correction_fn();

        // NOTE: interpolation/correction is only needed for components that are visually displayed!
        // we still need prediction to be able to correctly predict the physics on the client
        app.register_component::<LinearVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<AngularVelocity>()
            .add_prediction(PredictionMode::Full);

        // do not replicate Transform but make sure to register an interpolation function
        // for it so that we can do visual interpolation
        // (another option would be to replicate transform and not use Position/Rotation at all)
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation::<Transform>(TransformLinearInterpolation::lerp);
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation_mode::<Transform>(InterpolationMode::None);
    }
}

/// TODO: Remove this. Used just to give argument to client from CLI.
/// Inserted when CLI is parsed.
#[derive(Component)]
pub struct CliClientOptions {
    pub id: u64,
}

#[derive(Component, Serialize, Deserialize, Debug, Reflect, PartialEq, Clone)]
pub struct Bullet;

#[derive(Component, Serialize, Deserialize, Debug, Default, Reflect, PartialEq, Clone)]
pub struct Player;

impl Player {
    pub fn get_physics_bundle() -> impl Bundle {
        (RigidBody::Kinematic, Collider::circle(32.))
    }
}

/// Just a helper component for easy access of client id.
#[derive(Component, Serialize, Deserialize, Debug, Reflect, PartialEq, Clone, Copy)]
pub struct PlayerId(pub PeerId);

#[derive(Component)]
#[require(Name::new("Wall"), RigidBody::Static, Collider::rectangle(40., 40.))]
pub struct Wall;

#[derive(Component, Serialize, Deserialize, Debug, Reflect, PartialEq, Clone, Copy)]
pub struct Ball;

impl Ball {
    pub fn get_physics_bundle() -> impl Bundle {
        (
            avian2d::prelude::Position::default(),
            RigidBody::Dynamic,
            ColliderDensity(10.0),
            Collider::rectangle(40., 40.),
        )
    }
}

/// The different directions that the player can move the box
#[derive(Actionlike, Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
    Shoot,
}

fn transform_interpolation_fn(a: Transform, b: Transform, value: f32) -> Transform {
    let mut my_transform = a;

    my_transform.translation = a.translation.lerp(b.translation, value);
    my_transform.rotation = my_transform.rotation.lerp(b.rotation, value);
    my_transform.scale = my_transform.scale.lerp(b.scale, value);

    my_transform
}

fn position_should_rollback(
    this: &avian2d::prelude::Position,
    that: &avian2d::prelude::Position,
) -> bool {
    (this.0 - that.0).length() >= 0.01
}

fn rotation_should_rollback(
    this: &avian2d::prelude::Rotation,
    that: &avian2d::prelude::Rotation,
) -> bool {
    this.angle_between(*that) >= 0.01
}
