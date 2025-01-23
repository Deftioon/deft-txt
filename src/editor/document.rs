use std::{fs, cmp};
use crate::editor::util;

pub struct Document{
    rows: Vec<util::GapBuffer>,
    file_type: String,
    file_path: String,
}

impl Document{
    pub fn open(path: &str) -> Document{
        let mut rows = Vec::new();
        let content = fs::read_to_string(path).expect("Could not read file");
        let file_type = String::from(path.split('.').last().unwrap_or("txt"));
        for line in content.lines() {
            let row = util::GapBuffer::from_str(&line);
            rows.push(row);
        }
        
        Document{
            rows,
            file_type,
            file_path: String::from(path),
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error>{
        let content: Vec<String> = self.rows.iter().map(|row| row.to_string()).collect();
        fs::write(&self.file_path, content.join("\n"))
    }

    pub fn print(&self){
        for row in &self.rows{
            println!("{}", row.to_string());
        }
    }
    
    pub fn row(&self, index: usize) -> Option<&util::GapBuffer>{
        self.rows.get(index)
    }

    pub fn row_mut(&mut self, index: usize) -> Option<&mut util::GapBuffer>{
        self.rows.get_mut(index)
    }

    pub fn rows(&self) -> usize{
        self.rows.len()
    }

    pub fn cols(&self) -> usize{
        self.rows.iter().map(|row| row.buffer_length()).max().unwrap_or(0) as usize
    }
}