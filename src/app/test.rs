use ghost_cell::GhostToken;

use super::container::Container;

#[test]
fn test_game_join() {
    GhostToken::new(|mut token| {
        let mut container = Container::new(&mut token);
        let player_ind = container.new_player(&mut token).unwrap();
        let game_ind = container.new_game(&mut token).unwrap();
        let game = container.get_game(&token, game_ind);
        let player = container.get_player(&token, player_ind);
        let a = (&container, game, &token, player);
    });
}
