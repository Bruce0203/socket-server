use std::mem::MaybeUninit;

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};

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

impl MaxRepoLen for Player<'_, '_, '_> {
    const MAX: usize = 100;
}

#[derive(Debug)]
pub enum PlayerJoinServerError {
    ReachedMaxPlayers,
}

pub struct Game<'id, 'player, 'game> {
    pub joined_players: RefRepo<'id, 'game, Player<'id, 'player, 'game>>,
    pub game_index: usize,
}

impl MaxRepoLen for Game<'_, '_, '_> {
    const MAX: usize = 10;
}

#[derive(Debug)]
pub enum CreateGameError {
    FullOfCapacity,
}

#[derive(Debug)]
pub enum GameJoinError {
    ReachedMaxGamePlayers,
}

#[allow(type_alias_bounds)]
pub type Repo<'id, T: MaxRepoLen> = Vec<LCell<'id, T>, { <T as MaxRepoLen>::MAX }>;
#[allow(type_alias_bounds)]
pub type RefRepo<'id, 'a, T: MaxRepoLen> = Vec<&'a LCell<'id, T>, { <T as MaxRepoLen>::MAX }>;

pub trait MaxRepoLen {
    const MAX: usize;
}

impl<'id, 'player, 'game> Container<'id, 'player, 'game> {
    pub const LOBBY_GAME_INDEX: usize = 0;

    pub fn new(owner: &mut LCellOwner<'id>) -> Self {
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
        let (game, player) = owner.rw2(game_cell, player_cell);
        let game_player_index = game.joined_players.len();
        game.joined_players
            .push(&player_cell)
            .map_err(|_| GameJoinError::ReachedMaxGamePlayers)?;
        player.joined_game_players_index = MaybeUninit::new(game_player_index);
        player.joined_game = Some(&game_cell);
        Ok(())
    }
}
