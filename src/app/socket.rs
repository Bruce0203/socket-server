use std::time::Duration;

use qcell::{LCell, LCellOwner};

use crate::{
    net::socket::{ServerSocketListener, Socket},
    websocket::{websocket_flush, websocket_read, ReadError, WebSocketState},
};

use super::app::{App, Player};

#[derive(Default)]
pub struct Connection<'id> {
    pub game_id: Option<usize>,
    pub player_id: Option<usize>,
    pub websocket: LCell<'id, WebSocketState>,
}

impl<'id> ServerSocketListener<'id> for App {
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
                let entity_id =
                    unsafe { connection.connection.player_id.as_ref().unwrap_unchecked() };
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

fn init_connection(app: &mut App, connection: &mut Connection) -> Result<(), ()> {
    let game = unsafe { app.games.get_unchecked_mut(App::LOBBY_ID) };
    let player = Player {};
    let index = game.players.len();
    game.players.push(player).map_err(|_| ())?;
    connection.player_id = Some(index);
    connection.game_id = Some(App::LOBBY_ID);
    Ok(())
}

fn deinit_connection(app: &mut App, connection: &mut Connection) {
    unsafe {
        let game = app.games.get_unchecked_mut(connection.game_id.unwrap());
        let player_id = connection.player_id.unwrap();
        game.players.swap_remove_unchecked(player_id);
    }
}
