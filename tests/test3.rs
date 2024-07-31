#[cfg(feature = "aaaa")]
mod a {
    use fast_collections::Vec;
    use ghost_cell::{GhostCell, GhostToken};

    fn test() {
        struct Token(usize);
        GhostToken::new(|mut token| {
            let mut container = Container {
                games: Vec::uninit(),
                players: Vec::uninit(),
                registry: GhostCell::new(Vec::uninit()),
            };
            let player_index = container.create_player();
            let game_index = container.create_game();
            let game = container.get_game(game_index);
            let player = container.get_player(game_index);
            player.borrow_mut(&mut token).registry = Some(&container.registry);
            let _result = game.borrow_mut(&mut token).players.push(player);
        });
    }

    impl<'id, 'a, 'registry> Container<'id, 'a, 'registry> {
        fn get_game(&self, index: usize) -> &GhostCell<'id, Game<'id, 'a, 'registry>> {
            unsafe { self.games.get_unchecked(index) }
        }

        fn get_player(&self, index: usize) -> &GhostCell<'id, Player<'id, 'a, 'registry>> {
            unsafe { self.players.get_unchecked(index) }
        }

        fn create_game(&mut self) -> usize {
            let game_index = self.games.len();
            let game = GhostCell::new(Game {
                players: Vec::uninit(),
                game_index,
            });
            let _result = self.games.push(game);
            game_index
        }

        fn create_player(&mut self) -> usize {
            let player_index = self.players.len();
            let player = GhostCell::new(Player {
                joined_game: None,
                player_index,
                registry: None,
            });
            let _result = self.players.push(player);
            player_index
        }

        fn create_game_and_get(&mut self) -> &GhostCell<'id, Game<'id, 'a, 'registry>> {
            let game_index = self.create_game();
            unsafe { self.games.get_unchecked(game_index) }
        }

        fn create_player_and_get(&mut self) -> &GhostCell<'id, Player<'id, 'a, 'registry>> {
            let game_index = self.create_player();
            unsafe { self.players.get_unchecked(game_index) }
        }
    }

    struct Container<'id, 'a, 'registry> {
        games: Vec<GhostCell<'id, Game<'id, 'a, 'registry>>, 100>,
        players: Vec<GhostCell<'id, Player<'id, 'a, 'registry>>, 100>,
        registry: GhostCell<'a, Vec<usize, 100>>,
    }

    struct Player<'id, 'a, 'registry> {
        player_index: usize,
        joined_game: Option<&'a GhostCell<'id, Game<'id, 'a, 'registry>>>,
        registry: Option<&'registry GhostCell<'id, Vec<usize, 100>>>,
    }

    struct Game<'id, 'a, 'registry> {
        pub game_index: usize,
        pub players: Vec<&'a GhostCell<'id, Player<'id, 'a, 'registry>>, 100>,
    }
}
