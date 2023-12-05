use std::io;
use std::time::Instant;

use crate::board::Board;
use crate::perft::perft;
use crate::search::search;

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
                        //println!("{}: {}", val.to_string(), self.board.evaluate());
                    }
                })
            }
        }
        self.board = board;
    }
    fn go(&mut self, command: String) {
        let mut words = command.split_whitespace();
        words.next();
        while let Some(token) = words.next() {
            match token {
                "perft" => {
                    let start = Instant::now();
                    let result = perft(&self.board.fen(), words.next().unwrap_or("1").parse().unwrap());
                    println!("Nodes: {}", result.nodes);
                    let seconds = start.elapsed().as_secs_f32();
                    println!("Time elapsed: {}", seconds);
                    println!("Nps: {}", result.nodes as f32 / seconds);
                    return;
                }
                //"infinite" => search_option.infinite = true,
                //"depth" => {
                //    if let Some(depth_string) = words.next() {
                //        if let Ok(depth) = depth_string.parse::<u32>() {
                //            search_option.depth = depth
                //        }
                //    }
                //}
                _ => {}
            }
        }
        let search_result = search(1.0, &mut self.board);
        println!("bestmove {}", search_result.best_move.unwrap());
        println!("Depth reached: {}", search_result.depth_reached);
        println!("Leaf nodes evaluated: {}", search_result.positions_evaluated);
        println!("Transpositions: {}", search_result.transpositions);
        println!("Material balance: {}", self.board.material_balance);
        println!("Position balance: {}", self.board.position_balance);
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
