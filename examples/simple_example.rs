#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{net::SocketAddr, time::Duration};

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};
use socket_server::{
    selector::listen,
    socket::{ServerSocketListener, Socket},
};

fn main() {
    LCellOwner::scope(|mut owner| listen(&mut owner, GameServer::new(), "[::]:0"));
}

#[derive(Default)]
pub struct GameServer<'id, 'game> {
    games: Vec<LCell<'id, GameRoom<'id, 'game>>, 10>,
}

impl GameServer<'_, '_> {
    pub fn new() -> Self {
        Self {
            games: Vec::uninit(),
        }
    }
}

pub struct GameRoom<'id, 'game> {
    players: Vec<Player<'id, 'game>, 10>,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            player_index: 0,
            player_s_game_room_index: 0,
        }
    }
}

#[derive(Default)]
pub struct Player<'id, 'game> {
    joined_game: Option<&'game LCell<'id, GameRoom<'id, 'game>>>,
}

pub struct Connection {
    player_s_game_room_index: usize,
    player_index: usize,
}

impl<'id: 'game, 'game> ServerSocketListener<'id> for GameServer<'id, 'game> {
    const MAX_CONNECTIONS: usize = 10;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    const TICK: Duration = Duration::from_millis(50);
    type Connection = Connection;

    fn tick(server: &LCell<'id, Self>, owner: &mut LCellOwner<'id>) {}

    fn accept(
        owner: &mut LCellOwner<'id>,
        server: &LCell<'id, Self>,
        connection: &mut Socket<'id, '_, Self>,
        _addr: SocketAddr,
    ) {
        let game = unsafe { server.ro(owner).games.get_unchecked(0) };
        connection.player_s_game_room_index = 0;
    }

    fn read(
        owner: &mut LCellOwner<'id>,
        server: &LCell<'id, Self>,
        connection: &mut Socket<'id, '_, Self>,
    ) {
        println!("socket read");
    }

    fn flush(
        owner: &mut LCellOwner<'id>,
        server: &LCell<'id, Self>,
        connection: &mut Socket<'id, '_, Self>,
    ) {
        println!("socket flushed");
    }

    fn close(
        owner: &mut LCellOwner<'id>,
        server: &LCell<'id, Self>,
        connection: &mut Socket<'id, '_, Self>,
    ) {
        println!("socket closed");
    }
}
