use std::{fs, cmp};
use crate::editor::util;

pub struct Document{
    pub rows: Vec<util::GapBuffer>,
    pub file_type: String,
    pub file_path: String,
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

    pub fn new_line(&mut self, row_idx: usize, col: usize) {
        if let Some(current_row) = self.rows.get_mut(row_idx) {
            // Get the full content as a UTF-8 string
            let content = current_row.to_string();
            
            // Find the byte offset for the character position
            let mut byte_offset = content.len(); // Default to end if col is out of bounds
            let mut current_char = 0;
            for (idx, _) in content.char_indices() {
                if current_char == col {
                    byte_offset = idx;
                    break;
                }
                current_char += 1;
            }
    
            // Split the content into two parts
            let (pre_split, post_split) = content.split_at(byte_offset);
    
            // Replace current row with pre-split content
            *current_row = util::GapBuffer::from_str(pre_split);
    
            // Create new row with post-split content
            let new_row = util::GapBuffer::from_str(post_split);
            self.rows.insert(row_idx + 1, new_row);
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