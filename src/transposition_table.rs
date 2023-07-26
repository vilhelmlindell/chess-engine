use std::collections::HashMap;

//#[derive(Clone)]
pub struct TranspositionTable {
    pub table: HashMap<u64, TranspositionEntry>,
}

pub struct TranspositionEntry {
    pub depth: u32,
    pub eval: i32,
    pub node_type: NodeType,
}

pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable { table: HashMap::new() }
    }

    pub fn store(&mut self, hash: u64, depth: u32, eval: i32, node_type: NodeType) {
        self.table.insert(hash, TranspositionEntry { depth, eval, node_type });
    }

    pub fn probe(&self, hash: u64) -> Option<&TranspositionEntry> {
        self.table.get(&hash)
    }
}
