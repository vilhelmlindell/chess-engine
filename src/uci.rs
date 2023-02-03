use std::io;
use std::thread;

fn main() {
    let thread = thread::spawn(|| start());
}

struct UCIEngine {}

fn start() {
    loop {
        let command = String::new();
        io::stdin().read_line(&mut command);
        handle_command(&command);
    }
}

fn handle_command(command: &String) {
    if command.starts_with("uci") {
        start_engine();
    } else if command.starts_with("debug") {
    } else if command.starts_with("isready") {
    } else if command.starts_with("setoption name") {
    } else if command.starts_with("register") {
    } else if command.starts_with("ucinewgame") {
    } else if command.starts_with("position") {
    } else if command.starts_with("go") {
    } else if command.starts_with("stop") {
    } else if command.starts_with("ponderhit") {
    } else if command.starts_with("quit") {
    }
}
fn send_command(command: &String) {}

fn start_engine() {}
