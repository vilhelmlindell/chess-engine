use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::board::Board;

pub struct SearchOption {
    depth: u32,
    infinite: bool,
}

pub struct UCI {
    name: Option<String>,
    author: Option<String>,
    search_option: SearchOption,
    is_debug: bool,
    is_running: bool,
}

impl UCI {
    pub fn start() {
        let mut uci = UCI {
            name: None,
            author: None,
            search_option: SearchOption { depth: 5, infinite: true },
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
        self.parse_command(&mut command);
    }
    fn parse_command(&mut self, full_command: &String) {
        let command = full_command.split_once(" ").unwrap().0;
        match command {
            "uci" => self.identify(),
            "debug" => self.set_debug(full_command),
            "is_ready" => self.synchronize(),
            //"setoption" => self.set_option(full_command),
            //"register" => self.register(full_command),
            //"ucinewgame" => self.new_game(),
            "position" => self.set_position(full_command),
            "go" => self.go(full_command),
            //"stop" => self.stop(),
            //"ponderhit" => ,
            //"quit" => self.quit(),
            _ => {}
        }
    }
    fn identify(&self) {
        println!("id name chess_engine");
        println!("id name vilhelm lindell");
        println!("uciok");
    }
    fn register(&self, command: &String) {
        //let words = command.split_whitespace();
        //words.next();
        //while let token = words.next() {
        //    match token {
        //        "later" => return,
        //        ""
        //    }
        //}
    }
    fn set_debug(&mut self, command: &String) {
        if command.ends_with("on") {
            self.is_debug = true;
        } else if command.ends_with("off") {
            self.is_debug = false;
        }
    }
    fn synchronize(&self) {
        println!("readyok");
    }
    fn set_position(&self, command: &String) {
        let words: Vec<&str> = command.split_whitespace().collect();
        let mut moves_index = 8;
        let mut board = {
            if words[1] == "startpos" {
                moves_index = 2;
                Board::start_pos()
            } else if words[1] == "fen" {
                let fen = &words[2..8].concat();
                Board::from_fen(fen)
            } else {
                return;
            }
        };
        if words[moves_index] == "moves" {
            let moves = &words[moves_index + 1..];
            for mov in moves {
                let all_moves = board.generate_moves();
                all_moves.iter().enumerate().for_each(|(i, val)| {
                    if val.to_string() == mov.to_string() {
                        board.make_move(&all_moves[i])
                    }
                })
            }
        }
    }
    fn go(&mut self, command: &String) {
        let mut words = command.split_whitespace();
        words.next();
        while let Some(token) = words.next() {
            match token {
                "infinite" => self.search_option.infinite = true,
                "depth" => {
                    if let Some(depth_string) = words.next() {
                        if let Ok(depth) = depth_string.parse::<u32>() {
                            self.search_option.depth = depth
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

//pub fn start() {
//    loop {
//        let command = String::new();
//        io::stdin().read_line(&mut command);
//        handle_command(&command);
//    }
//}
