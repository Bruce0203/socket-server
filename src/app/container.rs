use sectorize::{EntityId, MoveSectorError, SectorId, World};

use super::socket_listener::Connection;

#[derive(Default)]
pub struct Container {
    pub world: World<Game, Player, 100>,
}

pub struct Player {}

pub struct Game {}

pub fn init_connection(app: &mut Container, connection: &mut Connection) -> Result<(), ()> {
    let player_id = app.world.create_entity(Player {}).map_err(|_| ())?;
    connection.player_id = Some(player_id);
    Ok(())
}

pub fn deinit_connection(app: &mut Container, connection: &mut Connection) -> Result<(), ()> {
    let player_id = connection.player_id.as_ref().unwrap();
    app.world.remove_entity(player_id);
    Ok(())
}

pub fn join_game(
    app: &mut Container,
    sector_id: &SectorId,
    entity_id: &EntityId,
) -> Result<(), ()> {
    app.world
        .move_entity_to_another_sector(entity_id, sector_id)
        .map_err(|MoveSectorError::MaxEntities| ())?;
    Ok(())
}

pub fn quit_game(app: &mut Container, entity_id: &EntityId) -> Result<(), ()> {
    app.world.remove_entity(entity_id);
    Ok(())
}
