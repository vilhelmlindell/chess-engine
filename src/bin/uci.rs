use std::io::{self, BufRead};
use std::option::Option;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, spawn, JoinHandle};
use std::time::Instant;

use chess_engine::bench;
use chess_engine::board::zobrist_hash::initialize_zobrist_tables;
use chess_engine::board::{Board, Side};
use chess_engine::move_generation::attack_tables::{get_between_ray, get_checkmask_between, initialize_tables};
use chess_engine::move_generation::generate_moves;
use chess_engine::perft::perft;
use chess_engine::search::{Search, SearchMode, SearchParams, SearchResult};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(arg) = args.get(1) {
        if arg == "bench" {
            bench::bench();
        }
        return;
    }
    Uci::start();
}

fn spawn_stdin_channel() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

pub struct Uci {
    name: String,
    author: String,
    board: Board,
    is_debug: bool,
    is_running: bool,
    should_quit_search: Arc<AtomicBool>,
    search_thread: Option<JoinHandle<SearchResult>>,
}

impl Uci {
    pub fn start() {
        let mut uci = Self {
            name: "".to_string(),
            author: "".to_string(),
            board: Board::start_pos(),
            is_debug: false,
            is_running: true,
            should_quit_search: Arc::new(AtomicBool::new(false)),
            search_thread: None,
        };

        let stdin_channel = spawn_stdin_channel();

        while uci.is_running {
            match stdin_channel.try_recv() {
                Ok(line) => {
                    uci.parse_command(line);
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }

            // Check if search has finished
            uci.check_search_result();

            // Small sleep to prevent CPU spinning
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn check_search_result(&mut self) {
        if let Some(handle) = self.search_thread.take() {
            if handle.is_finished() {
                match handle.join() {
                    Ok(result) => {
                        if let Some(mov) = result.pv.first() {
                            println!("bestmove {}", mov);
                        } else {
                            println!("bestmove (none)");
                        }
                    }
                    Err(e) => eprintln!("Search thread panicked: {:?}", e),
                }
            } else {
                // Put the handle back if search isn't finished
                self.search_thread = Some(handle);
            }
        }
    }

    fn parse_command(&mut self, full_command: String) {
        let command = full_command.split_whitespace().next().unwrap_or("");
        match command {
            "uci" => self.identify(),
            "debug" => self.set_debug(full_command),
            "isready" => self.synchronize(),
            "position" => self.set_position(full_command),
            "go" => self.go(full_command),
            "fen" => {
                println!("{}", self.board.fen());
            }
            "stop" => {
                self.should_quit_search.store(true, Ordering::SeqCst);
            }
            "quit" => {
                self.should_quit_search.store(true, Ordering::SeqCst);
                self.quit();
            }
            _ => {}
        }
    }

    fn identify(&self) {
        println!("id name chess_engine");
        println!("id name vilhelm lindell");
        println!("uciok");
    }
    //fn register(&self, command: String) {
    //let words = command.split_whitespace();
    //words.next();
    //while let token = words.next() {
    //    match token {
    //        "later" => return,
    //        ""
    //    }
    //}
    //}
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
                        println!("{} {}", mov.to_string(), board.fen());
                        //println!("{}: {}", val.to_string(), self.board.evaluate());
                    }
                })
            }
        }
        self.board = board;
    }

    fn go(&mut self, command: String) {
        // Stop any existing search
        if let Some(handle) = self.search_thread.take() {
            self.should_quit_search.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }

        let mut words = command.split_whitespace().peekable();
        words.next();
        let mut search_params = SearchParams::default();
        search_params.use_book = true;

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
                    search_params.search_mode = SearchMode::Clock;
                    if let Some(&time_str) = words.peek() {
                        if let Ok(time_left) = time_str.parse() {
                            search_params.clock.time[Side::White] = time_left;
                            words.next();
                        }
                    }
                }
                "btime" => {
                    search_params.search_mode = SearchMode::Clock;
                    if let Some(&time_str) = words.peek() {
                        if let Ok(time_left) = time_str.parse() {
                            search_params.clock.time[Side::Black] = time_left;
                            words.next();
                        }
                    }
                }
                "winc" => {
                    if let Some(&inc_str) = words.peek() {
                        if let Ok(inc) = inc_str.parse() {
                            search_params.clock.inc[Side::White] = inc;
                            words.next();
                        }
                    }
                }
                "binc" => {
                    if let Some(&inc_str) = words.peek() {
                        if let Ok(inc) = inc_str.parse() {
                            search_params.clock.inc[Side::Black] = inc;
                            words.next();
                        }
                    }
                }
                "movetime" => {
                    if let Some(&time_str) = words.peek() {
                        if let Ok(move_time) = time_str.parse() {
                            search_params.move_time = move_time;
                            search_params.search_mode = SearchMode::MoveTime;
                            words.next();
                        }
                    }
                }
                "infinite" => search_params.search_mode = SearchMode::Infinite,
                "nobook" => search_params.use_book = false,
                _ => {}
            }
        }

        let mut search = Search {
            board: self.board.clone(),
            ..Default::default()
        };
        search.should_quit = self.should_quit_search.clone();

        self.search_thread = Some(thread::spawn(move || {
            let result = search.search(search_params);
            result
        }));

        //let mut search = Search::default();

        //// Time the cloning of the board
        ////let clone_start = Instant::now();
        //let mut board_clone = self.board.clone();
        ////let clone_duration = clone_start.elapsed();
        ////println!("Cloning the board took: {:?}", clone_duration);

        //search.should_quit = self.should_quit_search.clone();

        //// Time the search itself inside the thread
        //self.search_thread = Some(thread::spawn(move || {
        //    //let search_start = Instant::now();
        //    let result = search.search(search_params, &mut board_clone);
        //    //let search_duration = search_start.elapsed();
        //    //println!("Search took: {:?}", search_duration);
        //    result
        //}));
    }

    fn ponder(&self) {}
    fn quit(&mut self) {
        self.is_running = false;
    }
}
