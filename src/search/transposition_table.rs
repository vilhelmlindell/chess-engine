use ctor::ctor;

use crate::board::piece_move::Move;
use std::mem::MaybeUninit;

pub const TABLE_SIZE: usize = mb_to_count(256);

pub const fn mb_to_count(mb: usize) -> usize {
    (mb * 1_004_858) / std::mem::size_of::<TranspositionEntry>()
}

pub static mut TRANSPOSITION_TABLE: MaybeUninit<TranspositionTable> = MaybeUninit::uninit();

// For global static access - inherently unsafe
pub fn global_store(entry: TranspositionEntry) {
    unsafe {
        (*TRANSPOSITION_TABLE.as_mut_ptr()).store(entry);
    }
}

pub fn global_probe(hash: u64) -> Option<TranspositionEntry> {
    unsafe { (*TRANSPOSITION_TABLE.as_mut_ptr()).probe(hash) }
}

// Then in your initialization function with the ctor library:
#[ctor]
fn initialize_table() {
    unsafe {
        TRANSPOSITION_TABLE.write(TranspositionTable::new());
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    pub table: Box<[Option<TranspositionEntry>; TABLE_SIZE]>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![None; TABLE_SIZE].into_boxed_slice().try_into().unwrap(),
        }
    }
    pub fn store(&mut self, entry: TranspositionEntry) {
        let index = self.get_index(entry.hash);
        self.table[index] = Some(entry);
        //if let Some(existing) = self.table[index] {
        //        self.table[index] = Some(entry);
        //} else {
        //    self.table[index] = Some(entry);
        //}
    }

    pub fn probe(&self, hash: u64) -> Option<TranspositionEntry> {
        let index = self.get_index(hash);
        self.table[index]
    }

    pub fn clear(&mut self) {
        *self.table = [None; TABLE_SIZE]
    }

    pub fn filled_count(&self) -> usize {
        self.table.iter().filter(|entry| entry.is_some()).count()
    }

    pub fn filled_percentage(&self) -> f64 {
        let filled = self.filled_count();
        (filled as f64 / TABLE_SIZE as f64) * 100.0
    }

    fn get_index(&self, hash: u64) -> usize {
        (hash % (TABLE_SIZE) as u64) as usize
    }
}

//impl Default for TranspositionTable {
//    fn default() -> Self {
//        Self {
//            table: vec![None; TABLE_SIZE].into_boxed_slice().try_into().unwrap(),
//        }
//    }
//}

#[derive(Clone, Copy, Debug)]
pub struct TranspositionEntry {
    pub depth: u8,
    pub eval: i16,
    pub best_move: Move,
    pub node_type: Bound,
    pub hash: u64,
}

impl TranspositionEntry {
    pub fn new(depth: u8, eval: i16, best_move: Move, node_type: Bound, hash: u64) -> Self {
        Self { depth, eval, best_move, node_type, hash }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}
