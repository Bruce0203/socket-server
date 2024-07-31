#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{io::Read, time::Duration};

use fast_collections::Cursor;
use qcell::LCellOwner;
use socket_server::socket_server::{entry_point, Socket, SocketListener};

fn main() {
    #[derive(Default)]
    struct Server {
        acc: u32,
    }
    #[derive(Default)]
    struct Connection {
        read_buf: Cursor<u8, 100>,
    }

    impl SocketListener for Server {
        const MAX_CONNECTIONS: usize = 5000;
        const READ_BUFFFER_LEN: usize = 100;
        const WRITE_BUFFER_LEN: usize = 100;
        const TICK: Duration = Duration::from_millis(50);
        type Connection = Connection;

        fn tick<'id>(&mut self, owner: &mut LCellOwner<'id>) {
            println!("acc: {}", self.acc);
        }

        fn accept<'id>(
            &mut self,
            owner: &mut LCellOwner<'id>,
            connection: &mut Socket<'id, '_, Self>,
        ) {
            self.acc += 1;
        }

        fn read<'id>(
            &mut self,
            owner: &mut LCellOwner<'id>,
            connection: &mut Socket<'id, '_, Self>,
        ) {
            let stream = &mut connection.stream;
            Cursor::push_from_read(&mut connection.read_buf, stream);
        }

        fn flush<'id>(
            &mut self,
            owner: &mut LCellOwner<'id>,
            connection: &mut Socket<'id, '_, Self>,
        ) {
        }

        fn close<'id>(
            &mut self,
            owner: &mut LCellOwner<'id>,
            connection: &mut Socket<'id, '_, Self>,
        ) {
            self.acc -= 1;
        }
    }

    run_with_stack_size(64 * 1024 * 1024, || {
        let addr = "[::]:25555".parse().unwrap();
        entry_point(Server::default(), addr);
    });
}

fn run_with_stack_size<F>(stack_size: usize, f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(stack_size)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}
