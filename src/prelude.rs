use crate::stream::packet::ServerBoundPacketStreamPipe;
use crate::stream::{
    mio::MioTcpStream, mock::MockStream, packet::ClientBoundPacketStreamPipe,
    readable_byte_channel::ReadableByteChannel, websocket::WebSocketServer,
    writable_byte_channel::WritableByteChannel,
};

pub use super::selector::{Selector, SelectorListener};
pub use super::stream::mock::MockSelector;
pub use super::stream::packet::WritePacket;
pub use super::stream::readable_byte_channel::{PollRead, ReceivePacket};
pub use super::stream::{Accept, Close, Flush, Id, Open, Read, ReadError, Write};

pub type MockTcpStream<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> = ReadableByteChannel<
    WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
    READ_BUF_LEN,
>;

pub type MockTcpClientBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ClientBoundPacketStreamPipe<
        WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
        PacketStream,
    >,
    READ_BUF_LEN,
>;

pub type MockTcpServerBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ServerBoundPacketStreamPipe<
        WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
        PacketStream,
    >,
    READ_BUF_LEN,
>;

pub type MockTcpWebSocketStream<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> =
    ReadableByteChannel<
        WebSocketServer<
            WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
        >,
        READ_BUF_LEN,
    >;

pub type MockTcpWebSocketClientBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ClientBoundPacketStreamPipe<
        WebSocketServer<
            WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
        >,
        PacketStream,
    >,
    READ_BUF_LEN,
>;

pub type MockTcpWebSocketServerBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ServerBoundPacketStreamPipe<
        WebSocketServer<
            WritableByteChannel<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, WRITE_BUF_LEN>,
        >,
        PacketStream,
    >,
    READ_BUF_LEN,
>;

pub type TcpStream<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> =
    ReadableByteChannel<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>, READ_BUF_LEN>;

pub type TcpClientBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ClientBoundPacketStreamPipe<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>, PacketStream>,
    READ_BUF_LEN,
>;

pub type TcpServerBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ServerBoundPacketStreamPipe<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>, PacketStream>,
    READ_BUF_LEN,
>;

pub type TcpWebSocketStream<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> =
    ReadableByteChannel<
        WebSocketServer<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>>,
        READ_BUF_LEN,
    >;

pub type TcpWebSocketClientBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ClientBoundPacketStreamPipe<
        WebSocketServer<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>>,
        PacketStream,
    >,
    READ_BUF_LEN,
>;

pub type TcpWebSocketServerBoundPacketStream<
    PacketStream,
    const READ_BUF_LEN: usize,
    const WRITE_BUF_LEN: usize,
> = ReadableByteChannel<
    ServerBoundPacketStreamPipe<
        WebSocketServer<WritableByteChannel<MioTcpStream, WRITE_BUF_LEN>>,
        PacketStream,
    >,
    READ_BUF_LEN,
>;
