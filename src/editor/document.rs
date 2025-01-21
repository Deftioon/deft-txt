use std::fs;
use crate::editor::row;

pub struct Document{
    rows: Vec<row::GapBuffer>,
}

impl Document{
    pub fn open(path: &str) -> Document{
        let mut rows = Vec::new();
        let content = fs::read_to_string(path).expect("Could not read file");
        for line in content.lines() {
            let row = row::GapBuffer::from_str(&line);
            rows.push(row);
        }
        
        Document{rows}
    }
    pub fn print(&self){
        for row in &self.rows{
            println!("{}", row.to_string());
        }
    }
    
    pub fn row(&self, index: usize) -> Option<&row::GapBuffer>{
        self.rows.get(index)
    }

    pub fn rows(&self) -> usize{
        self.rows.len()
    }
}