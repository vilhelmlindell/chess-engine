use std::io;

use crate::board::Board;
use crate::perft::perft;
use crate::piece_move::Move;

pub struct SearchOption {
    depth: u32,
    infinite: bool,
}

pub struct Uci {
    name: String,
    author: String,
    board: Board,
    is_debug: bool,
    is_running: bool,
}

impl Uci {
    pub fn start() {
        let mut uci = Self {
            name: "".to_string(),
            author: "".to_string(),
            board: Board::start_pos(),
            is_debug: false,
            is_running: true,
        };
        while uci.is_running {
            uci.process_input();
        }
    }
    fn process_input(&mut self) {
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        self.parse_command(command);
    }
    fn parse_command(&mut self, full_command: String) {
        let command = full_command.split_whitespace().collect::<Vec<&str>>()[0];
        match command {
            "uci" => self.identify(),
            "debug" => self.set_debug(full_command),
            "isready" => self.synchronize(),
            //"setoption" => self.set_option(full_command),
            //"register" => self.register(full_command),
            //"ucinewgame" => self.new_game(),
            "position" => self.set_position(full_command),
            "go" => self.go(full_command),
            //"stop" => self.stop(),
            //"ponderhit" => ,
            "quit" => self.quit(),
            _ => {}
        }
    }
    fn identify(&self) {
        println!("id name chess_engine");
        println!("id name vilhelm lindell");
        println!("uciok");
    }
    fn register(&self, command: String) {
        //let words = command.split_whitespace();
        //words.next();
        //while let token = words.next() {
        //    match token {
        //        "later" => return,
        //        ""
        //    }
        //}
    }
    fn set_debug(&mut self, command: String) {
        if command.ends_with("on") {
            self.is_debug = true;
        } else if command.ends_with("off") {
            self.is_debug = false;
        }
    }
    fn synchronize(&self) {
        println!("readyok");
    }
    fn set_position(&mut self, command: String) {
        let words: Vec<&str> = command.split_whitespace().collect();
        let mut moves_index = 8;
        let mut board = {
            if words[1] == "startpos" {
                moves_index = 2;
                Board::start_pos()
            } else if words[1] == "fen" {
                let fen = words[2..8].join(" ");
                Board::from_fen(&fen)
            } else {
                return;
            }
        };
        if let Some(move_string) = words.get(moves_index) {
            if move_string != &"moves" {
                return;
            }
            let moves = &words[(moves_index + 1)..];
            for mov in moves {
                let all_moves = board.generate_moves();
                all_moves.iter().enumerate().for_each(|(i, val)| {
                    if val.to_string() == *mov.to_string() {
                        board.make_move(all_moves[i]);
                    }
                })
            }
        }
        self.board = board;
    }
    fn go(&mut self, command: String) {
        let mut search_option = SearchOption { depth: 4, infinite: false };
        let mut words = command.split_whitespace();
        words.next();
        while let Some(token) = words.next() {
            match token {
                "perft" => {
                    perft(&self.board.fen(), words.next().unwrap_or("1").parse().unwrap());
                    return;
                }
                "infinite" => search_option.infinite = true,
                "depth" => {
                    if let Some(depth_string) = words.next() {
                        if let Ok(depth) = depth_string.parse::<u32>() {
                            search_option.depth = depth
                        }
                    }
                }
                _ => {}
            }
        }
        let best_move = self.search(search_option);
        println!("bestmove {best_move}");
        println!("{}", self.board.material_balance);
    }
    fn search(&mut self, search_option: SearchOption) -> Move {
        let mut lowest_score = i32::MAX;
        let mut best_move: Option<Move> = None;
        let moves = self.board.generate_moves();
        for mov in moves {
            self.board.make_move(mov);
            let score = self.board.alpha_beta_search(i32::MIN + 1, i32::MAX, search_option.depth);
            self.board.unmake_move(mov);

            if score < lowest_score {
                lowest_score = score;
                best_move = Some(mov);
            }
        }
        best_move.unwrap()
    }
    fn ponder(&self) {}
    fn quit(&mut self) {
        self.is_running = false;
    }
}

//pub fn start() {
//    loop {
//        let command = String::new();
//        io::stdin().read_line(&mut command);
//        handle_command(&command);
//    }
//}
