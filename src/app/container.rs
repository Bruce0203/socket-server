use std::mem::MaybeUninit;

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};

use super::socket_listener::Connection;

#[allow(type_alias_bounds)]
pub type Repo<'id, T: MaxRepoLen> = Vec<LCell<'id, T>, { <T as MaxRepoLen>::MAX }>;
#[allow(type_alias_bounds)]
pub type RefRepo<'id, 'a, T: MaxRepoLen> = Vec<&'a LCell<'id, T>, { <T as MaxRepoLen>::MAX }>;

pub trait MaxRepoLen {
    const MAX: usize;
}

impl MaxRepoLen for Player<'_, '_, '_> {
    const MAX: usize = 100;
}

impl MaxRepoLen for Game<'_, '_, '_> {
    const MAX: usize = 10;
}

#[derive(Default)]
pub struct Container<'id, 'player, 'game> {
    pub players: Repo<'id, Player<'id, 'player, 'game>>,
    pub games: Repo<'id, Game<'id, 'player, 'game>>,
}

pub struct Player<'id, 'player, 'game> {
    pub joined_game: Option<&'player LCell<'id, Game<'id, 'player, 'game>>>,
    pub joined_game_players_index: MaybeUninit<usize>,
    pub player_index: usize,
}

pub struct Game<'id, 'player, 'game> {
    pub joined_players: RefRepo<'id, 'game, Player<'id, 'player, 'game>>,
    pub game_index: usize,
}

#[derive(Debug)]
pub enum PlayerJoinServerError {
    ReachedMaxPlayers,
}

#[derive(Debug)]
pub enum CreateGameError {
    FullOfCapacity,
}

#[derive(Debug)]
pub enum GameJoinError {
    ReachedMaxGamePlayers,
}

impl<'id, 'player, 'game> Container<'id, 'player, 'game> {
    pub const LOBBY_GAME_INDEX: usize = 0;

    pub fn new() -> Self {
        let mut result = Self {
            players: Vec::uninit(),
            games: Vec::uninit(),
        };
        result
            .new_game()
            .expect("error occured while instantiating lobby");
        result
    }

    pub fn new_game(&mut self) -> Result<usize, CreateGameError> {
        let game_index = self.games.len();
        let game = LCell::new(Game {
            joined_players: Vec::uninit(),
            game_index,
        });
        self.games
            .push(game)
            .map_err(|_| CreateGameError::FullOfCapacity)?;
        Ok(game_index)
    }

    pub fn get_game(&self, index: usize) -> &LCell<'id, Game<'id, 'player, 'game>> {
        unsafe { self.games.get_unchecked(index) }
    }

    pub fn get_player(&self, index: usize) -> &LCell<'id, Player<'id, 'player, 'game>> {
        unsafe { self.players.get_unchecked(index) }
    }

    pub fn init_new_connection(
        &mut self,
        connection: &mut Connection<'id>,
    ) -> Result<(), PlayerJoinServerError> {
        let player_ind = self.new_player()?;
        connection.player_index = Some(player_ind);
        Ok(())
    }

    pub fn deinit_connection(
        &mut self,
        owner: &mut LCellOwner<'id>,
        connection: &mut Connection<'id>,
    ) {
        if let Some(player_index) = connection.player_index {
            let player_cell = self.get_player(player_index);
            self.player_quit_game(owner, player_cell)
        }
    }

    pub fn new_player(&mut self) -> Result<usize, PlayerJoinServerError> {
        let player_index = self.players.len();
        let player_cell = LCell::new(Player {
            joined_game: None,
            joined_game_players_index: MaybeUninit::uninit(),
            player_index,
        });
        self.players
            .push(player_cell)
            .map_err(|_| PlayerJoinServerError::ReachedMaxPlayers)?;
        Ok(player_index)
    }

    pub fn player_join_game(
        &self,
        owner: &mut LCellOwner<'id>,
        game_cell: &'game LCell<'id, Game<'id, 'player, 'game>>,
        player_cell: &'player LCell<'id, Player<'id, 'player, 'game>>,
    ) -> Result<(), GameJoinError> {
        self.player_quit_game(owner, player_cell);
        let (game, player) = owner.rw2(game_cell, player_cell);
        let game_player_index = game.joined_players.len();
        game.joined_players
            .push(&player_cell)
            .map_err(|_| GameJoinError::ReachedMaxGamePlayers)?;
        player.joined_game_players_index = MaybeUninit::new(game_player_index);
        player.joined_game = Some(&game_cell);
        Ok(())
    }

    pub fn player_quit_game(
        &self,
        owner: &mut LCellOwner<'id>,
        player_cell: &LCell<'id, Player<'id, 'player, 'game>>,
    ) {
        let player = owner.rw(player_cell);
        if let Some(joined_game) = player.joined_game {
            let (game, player) = owner.rw2(joined_game, player_cell);
            let game_player_index = unsafe { player.joined_game_players_index.assume_init() };
            unsafe { game.joined_players.swap_remove_unchecked(game_player_index) };
            player.joined_game = None;
        }
    }
}
