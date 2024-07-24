use std::time::Duration;

use socket_server::{packet_channel::ServerBoundPacket, websocket::WebSocketServer, SocketServer};

fn main() {
    let server = SocketServer::new(WebSocketServer::new(ServerBoundPacket::new(todo!())))
        socket_server::mio::entry_point(server, 25525, Duration::from_millis(50))
}
