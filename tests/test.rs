#[cfg(test)]
mod test {
    use packetize::{streaming_packets, Decode, Encode, SimplePacketStreamFormat};
    use socket_server::prelude::{
        Close, Flush, Id, MockSelector, MockTcpClientBoundPacketStream,
        MockTcpServerBoundPacketStream, PollRead, ReadError, ReceivePacket, Selector,
        SelectorListener, WritePacket,
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
    type ServerSocket =
        MockTcpServerBoundPacketStream<ConnectionState, READ_BUF_LEN, WRITE_BUF_LEN>;
    type ClientSocket =
        MockTcpClientBoundPacketStream<ConnectionState, READ_BUF_LEN, WRITE_BUF_LEN>;

    type ServerSelector = MockSelector<Selector<ServerApp, ServerSocket, ServerConnection, 10>>;
    type ClientSelector = MockSelector<Selector<ClientApp, ClientSocket, ClientConnection, 10>>;

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
        > SelectorListener<T, ServerConnection> for ServerApp
    {
        fn tick<const N: usize>(
            server: &mut Selector<Self, T, ServerConnection, N>,
        ) -> Result<(), ()> {
            todo!()
        }

        fn accept<const N: usize>(
            server: &mut Selector<ServerApp, T, ServerConnection, N>,
            id: Id<ServerConnection>,
        ) {
            todo!()
        }

        fn read<const N: usize>(
            server: &mut Selector<ServerApp, T, ServerConnection, N>,
            id: Id<ServerConnection>,
        ) -> Result<(), ReadError> {
            todo!()
        }

        fn close<const N: usize>(
            server: &mut Selector<ServerApp, T, ServerConnection, N>,
            id: Id<ServerConnection>,
        ) {
            todo!()
        }
    }
}
