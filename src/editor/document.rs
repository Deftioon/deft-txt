use std::{fs, cmp};
use crate::editor::util;

pub struct Document{
    rows: Vec<util::GapBuffer>,
}

impl Document{
    pub fn open(path: &str) -> Document{
        let mut rows = Vec::new();
        let content = fs::read_to_string(path).expect("Could not read file");
        for line in content.lines() {
            let row = util::GapBuffer::from_str(&line);
            rows.push(row);
        }
        
        Document{rows}
    }
    pub fn print(&self){
        for row in &self.rows{
            println!("{}", row.to_string());
        }
    }
    
    pub fn row(&self, index: usize) -> Option<&util::GapBuffer>{
        self.rows.get(index)
    }

    pub fn rows(&self) -> usize{
        self.rows.len()
    }

    pub fn cols(&self) -> usize{
        self.rows.iter().map(|row| row.buffer_length()).max().unwrap_or(0) as usize
    }
}