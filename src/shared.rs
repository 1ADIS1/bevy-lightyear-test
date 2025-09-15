//! This module contains the shared code between the client and the server.

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::protocol::PlayerAction;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

pub struct Channel1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Register your protocol, which is shared between client and server
        app.add_message::<Message1>()
            .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}

pub fn move_player(player: (&mut Transform, &PlayerAction), delta: f32) {
    let mut direction = Vec2::ZERO;

    if player.1.up {
        direction.y = 1.;
    }
    if player.1.down {
        direction.y = -1.;
    }
    if player.1.right {
        direction.x = 1.;
    }
    if player.1.left {
        direction.x = -1.;
    }

    direction = direction.normalize_or_zero();

    player.0.translation += direction.extend(0.) * 225. * delta;
}
