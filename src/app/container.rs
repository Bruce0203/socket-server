use std::mem::MaybeUninit;

use fast_collections::Vec;
use ghost_cell::{GhostCell, GhostToken};

pub struct Container<'id, 'player, 'game> {
    pub players: GhostCell<'id, Repo<'id, Player<'id, 'player, 'game>>>,
    pub games: GhostCell<'id, Repo<'id, Game<'id, 'player, 'game>>>,
}

pub struct Player<'id, 'player, 'game> {
    pub joined_game: Option<&'player GhostCell<'id, Game<'id, 'player, 'game>>>,
    pub joined_game_players_index: MaybeUninit<usize>,
    pub player_index: usize,
}

impl MaxRepoLen for Player<'_, '_, '_> {
    const MAX: usize = 100;
}

#[derive(Debug)]
pub enum PlayerJoinError {
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

#[allow(type_alias_bounds)]
pub type Repo<'id, T: MaxRepoLen> = Vec<GhostCell<'id, T>, { <T as MaxRepoLen>::MAX }>;
#[allow(type_alias_bounds)]
pub type RefRepo<'id, 'a, T: MaxRepoLen> = Vec<&'a GhostCell<'id, T>, { <T as MaxRepoLen>::MAX }>;

pub trait MaxRepoLen {
    const MAX: usize;
}

impl<'id, 'player, 'game> Container<'id, 'player, 'game> {
    pub const LOBBY_GAME_INDEX: usize = 0;

    pub fn new(owner: &mut GhostToken<'id>) -> Self {
        let mut result = Self {
            players: GhostCell::new(Vec::uninit()),
            games: GhostCell::new(Vec::uninit()),
        };
        result
            .new_game(owner)
            .expect("error occured while instantiating lobby");
        result
    }

    pub fn new_game(&mut self, owner: &mut GhostToken<'id>) -> Result<usize, CreateGameError> {
        let game_index = self.games.borrow(owner).len();
        let game = GhostCell::new(Game {
            joined_players: Vec::uninit(),
            game_index,
        });
        self.games
            .borrow_mut(owner)
            .push(game)
            .map_err(|_| CreateGameError::FullOfCapacity)?;
        Ok(game_index)
    }

    pub fn get_game<'a: 'c, 'owner: 'c, 'c>(
        &'a self,
        owner: &'owner GhostToken<'id>,
        index: usize,
    ) -> &'c GhostCell<'id, Game<'id, 'player, 'game>> {
        unsafe { self.games.borrow(owner).get_unchecked(index) }
    }

    pub fn get_player<'a: 'c, 'owner: 'c, 'c>(
        &'a self,
        owner: &'owner GhostToken<'id>,
        index: usize,
    ) -> &'c GhostCell<'id, Player<'id, 'player, 'game>> {
        unsafe { self.players.borrow(owner).get_unchecked(index) }
    }

    pub fn new_player(&mut self, owner: &mut GhostToken<'id>) -> Result<usize, PlayerJoinError> {
        let player_index = self.players.borrow(owner).len();
        let player_cell = GhostCell::new(Player {
            joined_game: None,
            joined_game_players_index: MaybeUninit::uninit(),
            player_index,
        });
        self.players
            .borrow_mut(owner)
            .push(player_cell)
            .map_err(|_| PlayerJoinError::ReachedMaxPlayers)?;
        Ok(player_index)
    }

    pub fn player_join_game(
        &self,
        owner: &mut GhostToken<'id>,
        game: &GhostCell<'id, Game<'id, 'player, 'game>>,
        player: &GhostCell<'id, Player<'id, 'player, 'game>>,
    ) {
    }
}
