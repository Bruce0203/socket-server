use qcell::LCellOwner;

use super::container::Container;

#[test]
fn test_game_join() {
    LCellOwner::scope(|mut owner| {
        let mut container = Container::new(&mut owner);
        let player_ind = container.new_player().unwrap();
        let game_ind = container.new_game().unwrap();
        let game = container.get_game(game_ind);
        let player = container.get_player(player_ind);
        container
            .player_join_game(&mut owner, game, player)
            .unwrap();
    });
}
