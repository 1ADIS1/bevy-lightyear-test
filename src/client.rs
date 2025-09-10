use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient},
    prelude::{client::NetcodeConfig, *},
};

use crate::shared::SERVER_ADDR;

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup);
    }
}

#[derive(Component)]
pub struct Player {
    pub id: usize,
    pub port: u16,
}

fn setup(mut commands: Commands, player_added_q: Query<(Entity, &Player), Added<Player>>) {
    for (player_entity, player) in player_added_q.iter() {
        commands.spawn(Camera2d);

        let auth = Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id: player.id as u64,
            private_key: Key::default(),
            protocol_id: 0,
        };

        let clien_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

        commands.entity(player_entity).insert((
            Client::default(),
            LocalAddr(clien_address),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ));

        commands.trigger_targets(Connect, player_entity);
    }
}
