use fast_collections::{String, Vec};

use crate::net::socket::ServerSocketListener;

#[derive(Default)]
pub struct App {
    pub games: Vec<Game, { <App as ServerSocketListener>::MAX_CONNECTIONS }>,
}

pub struct Game {
    pub name: GameName,
    pub players: Vec<Player, { <App as ServerSocketListener>::MAX_CONNECTIONS }>,
}

pub type GameName = String<32>;

pub struct Player {}

impl<'id, 'a> App {
    pub const LOBBY_ID: usize = 0;
    pub const LOBBY_NAME: GameName = String::from_array(*b"Lobby");

    pub fn new() -> Self {
        Self {
            games: Vec::from_array([(Game {
                name: Self::LOBBY_NAME,
                players: Vec::uninit(),
            })]),
        }
    }
}
