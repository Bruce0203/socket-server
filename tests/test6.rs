use std::{hint::black_box, marker::PhantomData, thread::sleep, time::Duration};

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};

pub struct Game<'id, 'a> {
    pub players: Vec<LCell<'id, &'a Player<'id, 'a>>, 100>,
    _marker: PhantomData<&'id ()>,
}

pub struct App<'id, 'a> {
    pub available_games: Vec<LCell<'id, Game<'id, 'a>>, 100>,
    pub online_players: Vec<LCell<'id, Player<'id, 'a>>, 100>,
}

pub struct Player<'id, 'a> {
    joined_game: Option<&'a LCell<'id, Game<'id, 'a>>>,
}

fn create_player<'id, 'a, 'new_player, 'owner: 'new_player, 'app: 'new_player>(
    owner: &'owner mut LCellOwner<'id>,
    app: &'app LCell<'id, App<'id, 'a>>,
) -> &'new_player LCell<'id, Player<'id, 'a>> {
    let playr_index = app.ro(owner).online_players.len();
    let player = owner.cell(Player { joined_game: None });
    app.rw(owner).online_players.push(player);
    unsafe { app.ro(owner).online_players.get_unchecked(playr_index) }
}

fn create_game<'id, 'a, 'owner: 'new_game, 'app: 'new_game, 'new_game>(
    owner: &'owner mut LCellOwner<'id>,
    app: &'app LCell<'id, App<'id, 'a>>,
) -> &'new_game LCell<'id, Game<'id, 'a>> {
    let game_index = app.ro(owner).available_games.len();
    let game = owner.cell(Game {
        players: Vec::uninit(),
        _marker: PhantomData,
    });
    app.rw(owner).available_games.push(game);
    unsafe { app.ro(owner).available_games.get_unchecked(game_index) }
}

fn player_join_game<'id, 'a>(
    owner: &mut LCellOwner<'id>,
    app: &mut App<'id, 'a>,
    player: &'a LCell<'id, Player<'id, 'a>>,
    game: &'a LCell<'id, Game<'id, 'a>>,
) {
    player.rw(owner).joined_game = Some(game);
}

#[test]
fn main() {
    LCellOwner::scope(|mut owner| {
        let mut app = App {
            available_games: Vec::uninit(),
            online_players: Vec::uninit(),
        };
    });
}

fn init<'id, 'a>(owner: &mut LCellOwner<'id>, app: &LCell<'id, App<'id, 'a>>) {
    let game = create_game(owner, app);
    let player = create_player(owner, app);
}
