use avian2d::prelude::{
    AngularVelocity, Collider, ColliderDensity, ComputedMass, ExternalForce, ExternalImpulse,
    LinearVelocity, Position, RigidBody, Rotation,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};
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
                rebroadcast_inputs: true,
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

        // Fully replicated, but not visual, so no need for lerp/corrections:
        // NOTE: interpolation/correction is only needed for components that are visually displayed!
        // we still need prediction to be able to correctly predict the physics on the client
        app.register_component::<LinearVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<AngularVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalForce>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalImpulse>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ComputedMass>()
            .add_prediction(PredictionMode::Full);

        // Set up visual interp plugins for Position/Rotation. Position/Rotation is updated in FixedUpdate
        // by the physics plugin so we make sure that in PostUpdate we interpolate it
        app.add_plugins(FrameInterpolationPlugin::<avian2d::prelude::Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<avian2d::prelude::Rotation>::default());

        // Observers that add VisualInterpolationStatus components to entities
        // which receive a Position and are predicted
        app.add_observer(add_visual_interpolation_components);

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

/// Add the VisualInterpolateStatus::<Transform> component to non-floor entities with
/// component `Position`. Floors don't need to be visually interpolated because we
/// don't expect them to move.
///
/// We query Without<Confirmed> instead of With<Predicted> so that the server's
/// gui will also get some visual interpolation. But we're usually just
/// concerned that the client's Predicted entities get the interpolation
/// treatment.
fn add_visual_interpolation_components(
    // We use Position because it's added by avian later, and when it's added
    // we know that Predicted is already present on the entity
    trigger: Trigger<OnAdd, Position>,
    query: Query<Entity, With<Predicted>>,
    mut commands: Commands,
) {
    if !query.contains(trigger.target()) {
        return;
    }
    commands.entity(trigger.target()).insert((
        FrameInterpolate::<Position> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
        FrameInterpolate::<Rotation> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
    ));
}
