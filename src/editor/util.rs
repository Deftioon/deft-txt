use std::{fmt, cmp};
use std::ops::Range;

pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);
        GapBuffer {
            buffer,
            gap_start: 0,
            gap_end: capacity,
        }
    }

    pub fn buffer_length(&self) -> usize {
        self.buffer.len() - self.gap_length()
    }

    fn gap_length(&self) -> usize {
        self.gap_end - self.gap_start
    }

    fn grow_gap(&mut self, needed: usize) {
        let current_length = self.buffer.len();
        let new_size = current_length + needed.max(CHUNK_SIZE);
        let post_gap_length = current_length - self.gap_end;
        
        self.buffer.resize(new_size, 0);
        
        // Move post-gap content to new position
        let new_gap_end = new_size - post_gap_length;
        self.buffer.copy_within(
            self.gap_end..current_length,
            new_gap_end
        );
        
        self.gap_end = new_gap_end;
    }

    pub fn move_gap(&mut self, new_gap_start: usize) {
        if new_gap_start == self.gap_start {
            return;
        }
    
        let shift = new_gap_start as isize - self.gap_start as isize;
    
        if shift > 0 {
            // Positive shift: move gap to the right
            let shift = shift as usize;
            self.buffer.copy_within(
                self.gap_end..self.gap_end + shift,
                self.gap_start,
            );
        } else {
            // Negative shift: move gap to the left
            let shift_abs = shift.unsigned_abs(); // Absolute value as usize
            let source_start = self.gap_start.saturating_sub(shift_abs);
            let dest_start = self.gap_end.saturating_sub(shift_abs);
    
            self.buffer.copy_within(
                source_start..self.gap_start,  // Source range
                dest_start,                    // Destination start
            );
        }
    
        // Update gap positions with overflow protection
        self.gap_end = (self.gap_end as isize)
            .saturating_add(shift)  // Prevent overflow/underflow
            as usize;
        self.gap_start = new_gap_start;
    }

    pub fn insert_char(&mut self, offset: usize, c: char) {
        let mut bytes = [0; 4];
        let bytes = c.encode_utf8(&mut bytes).as_bytes();
        self.insert(offset, bytes);
    }

    pub fn insert(&mut self, offset: usize, content: &[u8]) {
        if content.is_empty() {
            return;
        }

        let required_space = content.len();
        if self.gap_length() < required_space {
            self.grow_gap(required_space);
        }

        self.move_gap(offset);
        self.buffer.splice(
            self.gap_start..self.gap_start + content.len(),
            content.iter().cloned()
        );
        self.gap_start += content.len();
    }

    pub fn remove(&mut self, range: Range<usize>) {
        // Calculate indices without holding references
        let pre_len = range.start;
        let post_start = range.end;
        let post_len = self.buffer.len() - post_start;
    
        // Create new buffer by directly slicing into the old buffer
        let mut new_buffer = Vec::with_capacity(pre_len + post_len);
        new_buffer.extend_from_slice(&self.buffer[0..pre_len]);
        new_buffer.extend_from_slice(&self.buffer[post_start..]);
    
        // Replace the old buffer (no outstanding references exist here)
        self.buffer = new_buffer;
    
        // Update gap positions
        self.gap_start = pre_len;
        self.gap_end = self.buffer.capacity() - post_len;
    }

    pub fn remove_char(&mut self, offset: usize) {
        self.remove(offset..offset + 1);
    }

    pub fn from_str(s: &str) -> Self {
        let mut buffer = GapBuffer::new(s.len());
        buffer.insert(0, s.as_bytes());
        buffer
    }

    pub fn str_len(&self) -> usize {
        self.to_string().chars().count()
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let combined = [&self.buffer[0..self.gap_start], &self.buffer[self.gap_end..]].concat();
        let end = cmp::min(end, combined.len());
        let start = cmp::min(start, end);
        String::from_utf8_lossy(&combined[start..end]).to_string()
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let combined = [&self.buffer[0..self.gap_start], &self.buffer[self.gap_end..]].concat();
        write!(f, "{}", String::from_utf8_lossy(&combined))
    }
}

const CHUNK_SIZE: usize = 64;