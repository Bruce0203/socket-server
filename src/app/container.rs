use sectorize::World;

use super::{
    repo::{Element, Id, Repo},
    socket_listener::Connection,
};

#[derive(Default)]
pub struct Container {
    world: World<Game, Player, 100>,
}

pub struct Player {}

pub struct Game {}

pub fn init_connection<'id>(app: &mut Container, connection: &mut Connection) -> Result<(), ()> {
    let player_id = app.world.create_entity(Player {}).map_err(|_| ())?;
    connection.player_id = Some(player_id);
    Ok(())
}

pub fn join_game(game: &mut Element<Game>, player: &mut Element<Player>) -> Result<(), ()> {

    player.joined_game = Some(game.index);
    Ok(())
}

pub fn quit_game(app: &mut Container, player: &mut Element<Player>) -> Result<(), ()> {
    let game = app.games.get_mut(player.joined_game.ok_or_else(|| ())?);
    Ok(())
}
