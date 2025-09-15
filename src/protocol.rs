use bevy::{ecs::entity::MapEntities, prelude::*};
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

// Channels
pub struct Channel1;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAction>();

        app.add_plugins(input::leafwing::InputPlugin::<PlayerAction> {
            config: input::InputConfig::<PlayerAction> {
                // enable lag compensation; the input messages sent to the server will include the
                // interpolation delay of that client
                lag_compensation: true,
                ..default()
            },
        });

        app.register_component::<Transform>()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full)
            .add_interpolation_fn(transform_interpolation_fn);

        app.register_component::<Player>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Default, Reflect, PartialEq, Clone)]
pub struct Player;

#[derive(Component)]
pub struct ClientId(pub u64);

/// The different directions that the player can move the box
#[derive(Actionlike, Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
    Shoot,
}

// All inputs need to implement the `MapEntities` trait
// impl MapEntities for PlayerAction {
//     fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
// }

fn transform_interpolation_fn(a: Transform, b: Transform, value: f32) -> Transform {
    let mut my_transform = a;

    my_transform.translation = a.translation.lerp(b.translation, value);
    my_transform.rotation = my_transform.rotation.lerp(b.rotation, value);
    my_transform.scale = my_transform.scale.lerp(b.scale, value);

    my_transform
}
