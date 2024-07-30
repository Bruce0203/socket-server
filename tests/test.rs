#[cfg(test)]
mod test {
    use packetize::{streaming_packets, Decode, Encode, SimplePacketStreamFormat};
    use socket_server::{
        prelude::{
            Close, Flush, Id, MockSelector, MockTcpClientBoundPacketStream,
            MockTcpServerBoundPacketStream, PollRead, ReadError, ReceivePacket, Selector,
            SelectorListener, WritePacket,
        },
        stream::{
            mock::MockStream,
            packet::{ClientBoundPacketStreamPipe, ServerBoundPacketStreamPipe},
            readable_byte_channel::ReadableByteChannel,
            writable_byte_channel::WritableByteChannel,
            write_registry::SelectorWriteRegistry,
        },
    };

    #[streaming_packets(SimplePacketStreamFormat)]
    #[derive(Default)]
    enum ConnectionState {
        #[default]
        HandShake(HandShakeC2s),
    }
    #[derive(Encode, Decode)]
    struct HandShakeC2s;

    #[derive(Default)]
    struct ServerApp;
    #[derive(Default)]
    struct ServerConnection;
    #[derive(Default)]
    struct ClientApp;
    #[derive(Default)]
    struct ClientConnection;

    const READ_BUF_LEN: usize = 1000;
    const WRITE_BUF_LEN: usize = 1000;
    const MAX_CONNECTIONS: usize = 10;
    type ServerSocket = ReadableByteChannel<
        ServerBoundPacketStreamPipe<
            WritableByteChannel<
                SelectorWriteRegistry<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, MAX_CONNECTIONS>,
                WRITE_BUF_LEN,
            >,
            ConnectionState,
        >,
        READ_BUF_LEN,
    >;
    type ClientSocket = ReadableByteChannel<
        ClientBoundPacketStreamPipe<
            WritableByteChannel<
                SelectorWriteRegistry<MockStream<WRITE_BUF_LEN, READ_BUF_LEN>, MAX_CONNECTIONS>,
                WRITE_BUF_LEN,
            >,
            ConnectionState,
        >,
        READ_BUF_LEN,
    >;

    type ServerSelector = MockSelector<
        Selector<ServerApp, ServerSocket, ServerConnection, MAX_CONNECTIONS>,
        WRITE_BUF_LEN,
        READ_BUF_LEN,
    >;
    type ClientSelector = MockSelector<
        Selector<ClientApp, ClientSocket, ClientConnection, 10>,
        WRITE_BUF_LEN,
        READ_BUF_LEN,
    >;

    //FIXME TODO impl test for socket server
    #[ignore]
    #[test]
    fn test() {
        let server = ServerSelector::default();
        let server2 = ServerSelector::default();
        server.entry_point(server2);
    }

    impl<
            T: Close
                + Flush
                + PollRead
                + WritePacket<ClientBoundPacket>
                + ReceivePacket<ServerBoundPacket>,
            const N: usize,
        > SelectorListener<T, ServerConnection, N> for ServerApp
    {
        fn tick(server: &mut Selector<Self, T, ServerConnection, N>) -> Result<(), ()> {
            todo!()
        }

        fn accept(server: &mut Selector<Self, T, ServerConnection, N>, id: Id<ServerConnection>) {
            todo!()
        }

        fn read(
            server: &mut Selector<Self, T, ServerConnection, N>,
            id: Id<ServerConnection>,
        ) -> Result<(), ReadError> {
            todo!()
        }

        fn close(server: &mut Selector<Self, T, ServerConnection, N>, id: Id<ServerConnection>) {
            todo!()
        }
    }
}
