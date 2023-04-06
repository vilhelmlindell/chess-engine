use crate::board::Board;

pub fn perft(depth: &u32) -> u32 {
    let mut board = Board::start_pos();
    search(&mut board, depth)
}

fn search(board: &mut Board, depth: &u32) -> u32 {
    if *depth == 0 {
        return 1;
    }
    let mut node_count = 0;
    for mov in board.generate_moves() {
        board.make_move(mov);
        node_count += search(board, &(*depth - 1));
        board.unmake_move(mov);
    }
    node_count
}
