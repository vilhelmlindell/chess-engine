use std::io;
use std::time::Instant;

use chess_engine::board::{Board, Side};
use chess_engine::move_generation::generate_moves;
use chess_engine::perft::perft;
use chess_engine::search::{Search, SearchArgs};

fn main() {
    Uci::start();
}

pub struct Uci {
    name: String,
    author: String,
    board: Board,
    is_debug: bool,
    is_running: bool,
    search: Search,
}

impl Uci {
    pub fn start() {
        let mut uci = Self {
            name: "".to_string(),
            author: "".to_string(),
            board: Board::start_pos(),
            is_debug: false,
            is_running: true,
            search: Search::default(),
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
        println!("{}", full_command);
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
                let all_moves = generate_moves(&board);
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
        let mut words = command.split_whitespace().peekable();
        words.next();
        let mut search_args = SearchArgs::default();
        while let Some(token) = words.next() {
            match token {
                "perft" => {
                    let start = Instant::now();
                    let result = perft(&self.board.fen(), words.next().unwrap_or("1").parse().unwrap());
                    let seconds = start.elapsed().as_secs_f32();
                    println!("Nodes: {}", result.nodes);
                    println!("Time elapsed: {}", seconds);
                    println!("Nps: {}", result.nodes as f32 / seconds);
                    return;
                }
                "wtime" => {
                    let time_left = words.peek().expect("No time given for wtime, incorrect uci command").parse::<u32>().expect("Time was not given as a number");
                    search_args.time_left[Side::White] = time_left;
                }
                "btime" => {
                    let time_left = words.peek().expect("No time given for btime, incorrect uci command").parse::<u32>().expect("Time was not given as a number");
                    search_args.time_left[Side::Black] = time_left;
                }
                "winc" => {
                    let time_increment = words.peek().expect("No time given for winc, incorrect uci command").parse::<u32>().expect("Time was not given as a number");
                    search_args.time_increment[Side::White] = time_increment;
                }
                "binc" => {
                    let time_increment = words.peek().expect("No time given for binc, incorrect uci command").parse::<u32>().expect("Time was not given as a number");
                    search_args.time_increment[Side::Black] = time_increment;
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
        let result = self.search.search(search_args, &mut self.board);
        print!(
            "info depth {} score cp {} time {} nodes {} nps {} ",
            result.depth_reached, result.highest_eval, result.time, result.nodes, result.nodes
        );
        print!("pv");
        for mov in &result.pv {
            print!(" {}", mov);
        }
        println!();
        println!(
            "bestmove {}",
            result.pv.first().expect("No best move found")
        );
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
