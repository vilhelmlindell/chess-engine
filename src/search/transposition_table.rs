use crate::board::piece_move::Move;

pub const TABLE_SIZE: usize = u64::pow(2, 24) as usize - 1;

#[derive(Clone)]
pub struct TranspositionTable {
    pub table: Box<[Option<TranspositionEntry>; TABLE_SIZE]>,
}

impl TranspositionTable {
    pub fn store(&mut self, entry: TranspositionEntry) {
        let index = self.get_index(entry.hash);
        self.table[index] = Some(entry);
    }

    pub fn probe(&self, hash: u64) -> Option<TranspositionEntry> {
        let index = self.get_index(hash);
        self.table[index]
    }

    pub fn clear(&mut self) {
        *self.table = [None; TABLE_SIZE]
    }

    fn get_index(&self, hash: u64) -> usize {
        (hash % (TABLE_SIZE) as u64) as usize
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self {
            table: vec![None; TABLE_SIZE].into_boxed_slice().try_into().unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TranspositionEntry {
    pub depth: u32,
    pub eval: i32,
    pub best_move: Move,
    pub node_type: NodeType,
    pub hash: u64,
}

impl TranspositionEntry {
    pub fn new(depth: u32, eval: i32, best_move: Move, node_type: NodeType, hash: u64) -> Self {
        Self {
            depth,
            eval,
            best_move,
            node_type,
            hash,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}
