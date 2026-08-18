use std::collections::{HashMap, HashSet};
use std::io::Write;

pub const CHECKPOINT_EVERY_BATCHES: u64 = 100;

#[derive(Clone, Copy)]
pub(crate) struct TilePos {
    pub col: u32,
    pub row: u32,
}

#[derive(Default)]
pub(crate) struct Checkpoint {
    pub completed_zooms: HashSet<u8>,
    pub positions: HashMap<u8, TilePos>,
}

pub(crate) fn read_checkpoint(path: &str) -> Checkpoint {
    let mut cp = Checkpoint::default();
    let Ok(content) = std::fs::read_to_string(path) else {
        return cp;
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["done", z] => {
                if let Ok(z) = z.parse() {
                    cp.completed_zooms.insert(z);
                }
            }
            ["pos", z, col, row] => {
                if let (Ok(z), Ok(col), Ok(row)) = (z.parse(), col.parse(), row.parse()) {
                    cp.positions.insert(z, TilePos { col, row });
                }
            }
            _ => {}
        }
    }
    return cp;
}

pub(crate) fn append_checkpoint_line(path: &str, line: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to write checkpoint file");
    writeln!(file, "{line}").expect("Failed to write checkpoint");
}
