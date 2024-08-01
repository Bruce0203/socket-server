use std::time::Duration;

use qcell::{LCell, LCellOwner};

use crate::{
    socket_server::{Socket, SocketListener},
    websocket::{websocket_flush, websocket_read, ReadError, WebSocketState},
};

use super::container::App;

#[derive(Default)]
pub struct Connection<'id> {
    pub player_index: Option<usize>,
    pub websocket: LCell<'id, WebSocketState>,
}

impl<'id, 'a> SocketListener<'id> for App {
    const MAX_CONNECTIONS: usize = 5000;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    const TICK: Duration = Duration::from_millis(50);
    type Connection = Connection<'id>;

    fn tick(&mut self, owner: &mut LCellOwner<'id>) {}

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        match self.init_new_connection(connection) {
            Ok(()) => {}
            Err(PlayerJoinServerError::ReachedMaxPlayers) => connection.register_close_event(owner),
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
        //self.deinit_connection(owner, connection)
    }
}
