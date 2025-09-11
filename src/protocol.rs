use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::client::{Player, PlayerPosition};

// Channels
pub struct Channel1;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Direction>();

        app.add_plugins(lightyear::input::native::plugin::InputPlugin::<Direction>::default());

        app.register_component::<PlayerPosition>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<Player>();

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}

/// The different directions that the player can move the box
#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct Direction {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

// All inputs need to implement the `MapEntities` trait
impl MapEntities for Direction {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}
