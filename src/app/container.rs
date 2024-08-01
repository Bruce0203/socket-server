use fast_collections::Vec;
use qcell::LCellOwner;

use super::{
    repo::{Id, Repo},
    socket_listener::Connection,
};

pub struct Container<'id> {
    games: Repo<'id, Game<'id>>,
    players: Repo<'id, Player<'id>>,
}

pub struct Player<'id> {
    joined_game: Option<Id<'id, Game<'id>>>,
}

pub struct Game<'id> {
    joined_players: Vec<Id<'id, Player<'id>>, 100>,
}

pub fn init_connection<'id>(
    owner: &mut LCellOwner<'id>,
    app: &mut Container<'id>,
    connection: &mut Connection<'id>,
) -> Result<(), ()> {
    let player = app.players.add(owner, Player { joined_game: None })?;
    connection.player_index = Some(player);
    Ok(())
}
