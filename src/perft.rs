use crate::board::Board;

pub fn perft(depth: u32) -> u32 {
    let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    search(&mut board, depth)
}

fn search(board: &mut Board, depth: u32) -> u32 {
    if depth == 0 {
        return 1;
    }
    let mut node_count = 0;
    for mov in board.generate_moves() {
        board.make_move(mov);
        node_count += search(board, depth - 1);
        board.unmake_move(mov);
    }
    node_count
}
