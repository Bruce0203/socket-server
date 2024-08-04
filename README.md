example
```rust 
use std::time::Duration;

use qcell::LCellOwner;
use socket_server::{
    selector::listen,
    socket::{ServerSocketListener, Socket},
};

fn main() {
    LCellOwner::scope(|mut owner| listen(&mut owner, GameServer, "[::]:0"));
}

pub struct GameServer;
#[derive(Default)]
pub struct Player {}

impl<'id> ServerSocketListener<'id> for GameServer {
    const MAX_CONNECTIONS: usize = 10;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    const TICK: Duration = Duration::from_millis(50);
    type Connection = Player;

    fn tick(&mut self, owner: &mut LCellOwner<'id>) {
        todo!()
    }

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        todo!()
    }

    fn read(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        todo!()
    }

    fn flush(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        todo!()
    }

    fn close(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {
        todo!()
    }
}
```
