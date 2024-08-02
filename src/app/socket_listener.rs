use std::time::Duration;

use qcell::{LCell, LCellOwner};
use sectorize::{EntityId, SectorId};

use crate::{
    socket_server::{Socket, SocketListener},
    websocket::{websocket_flush, websocket_read, ReadError, WebSocketState},
};

use super::container::{deinit_connection, init_connection, Container, Player};

#[derive(Default)]
pub struct Connection<'id> {
    pub player_id: Option<EntityId>,
    pub websocket: LCell<'id, WebSocketState>,
}

impl<'id, 'game, 'player> SocketListener<'id> for Container {
    const MAX_CONNECTIONS: usize = 5000;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    const TICK: Duration = Duration::from_millis(50);
    type Connection = Connection<'id>;

    fn tick(&mut self, owner: &mut LCellOwner<'id>) {}

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        match init_connection(self, connection) {
            Ok(_) => {}
            Err(_) => {
                connection.register_close_event(owner);
            }
        }
    }

    fn read(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        match websocket_read(
            owner,
            &connection.websocket,
            &connection.read_buf,
            &connection.write_buf,
        ) {
            Ok(_) => {
                let entity_id = connection.connection.player_id.as_ref().unwrap();
                println!("HI");
                self.world
                    .move_entity_to_another_sector(entity_id, &0.into())
                    .unwrap();
                //let player = self.get_player(connection.player_index.unwrap());
                //let game = self.get_game(0); self.player_join_game(owner, game, player);
            }
            Err(err) => match err {
                ReadError::NotFullRead => {}
                ReadError::FlushRequest => connection.register_flush_event(owner),
                ReadError::CloseRequest => connection.register_close_event(owner),
            },
        }
    }

    fn flush(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        match websocket_flush(owner, &connection.websocket, &connection.write_buf) {
            Ok(()) => {}
            Err(()) => connection.register_close_event(owner),
        };
    }

    fn close(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        deinit_connection(self, connection);
    }
}
