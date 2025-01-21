
extern crate libc;

use std::{fmt, mem, cmp};
use std::ops::{Drop, Range};

const CHUNK_SIZE: isize = 8;

pub struct GapBuffer {
    buffer_start: *mut u8,
    gap_start: *mut u8,
    gap_end: *mut u8,
    buffer_end: *mut u8,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        let buffer = unsafe {
            let size = mem::size_of::<u8>() * capacity;
            libc::malloc(size) as *mut u8
        };

        if buffer.is_null() && capacity > 0 {
            panic!("Failed to allocate memory for GapBuffer");
        }

        GapBuffer{
            buffer_start: buffer,
            gap_start: buffer,
            gap_end: unsafe { buffer.offset(capacity as isize) },
            buffer_end: unsafe { buffer.offset(capacity as isize) },
        }
    }

    pub fn extend(&mut self, amount: isize) {
        let current_size = ptr_diff(self.buffer_end, self.buffer_start);
        let new_size = mem::size_of::<u8>() * amount as usize + current_size as usize;
        
        let new_buffer = unsafe {
            libc::realloc(self.buffer_start as *mut libc::c_void, new_size) as *mut u8
        };

        assert!(!new_buffer.is_null(), "Failed to reallocate memory for GapBuffer");

        self.buffer_start = new_buffer;
    }

    fn buffer_length(&self) -> isize {
        let head_length = ptr_diff(self.gap_start, self.buffer_start);
        let tail_length = ptr_diff(self.buffer_end, self.gap_end);
        head_length + tail_length
    }

    fn gap_length(&self) -> isize {
        ptr_diff(self.gap_end, self.gap_start)
    }

    fn grow_gap(&mut self, size: isize) {
        let available = self.gap_length();
        let needed = size - available;

        let mut chunk = (needed as f32 / CHUNK_SIZE as f32).ceil() as isize;
        chunk *= CHUNK_SIZE;

        let head_length = ptr_diff(self.gap_start, self.buffer_start);
        let tail_length = ptr_diff(self.buffer_end, self.gap_end);
        let new_gap_size = self.gap_length() + chunk;
        let buffer_length = head_length + tail_length;

        self.extend(chunk);
        unsafe {
            libc::memmove(
                self.gap_start as *mut libc::c_void,
                self.gap_end as *mut libc::c_void,
                tail_length as usize,
            );

            self.gap_start = self.gap_start.offset(buffer_length);
            self.gap_end = self.gap_start.offset(new_gap_size);
            self.buffer_end = self.gap_end;
        }
    }

    fn move_gap(&mut self, offset: isize) {
        let gap_len = self.gap_length();
        let new_pos = unsafe {
            self.buffer_start.offset(offset)
        };

        let diff = ptr_diff(new_pos, self.gap_start);

        if diff == 0 {
            return;
        }

        if diff < 0 {
            unsafe {
                self.gap_start = new_pos;
                self.gap_end = self.gap_start.offset(gap_len);
                libc::memmove(
                    self.gap_end as *mut libc::c_void,
                    self.gap_start as *mut libc::c_void,
                    -diff as usize,
                );
            }
        }
        else {
            unsafe {
                self.gap_end = self.gap_end.offset(diff);
                self.gap_start = self.gap_start.offset(diff);
                libc::memmove(
                    new_pos as *mut libc::c_void,
                    self.gap_start as *mut libc::c_void,
                    diff as usize,
                );
            }
        }
    }

    fn clear(&mut self) {
        self.gap_start = self.buffer_start;
        self.gap_end = self.buffer_end;
    }

    fn head(&self) -> String {
        let head_len = ptr_diff(self.gap_start, self.buffer_start) as usize;
        string_from_slice(self.buffer_start, head_len)
    }
    fn tail(&self) -> String {
        let tail_len = ptr_diff(self.buffer_end, self.gap_end) as usize;
        string_from_slice(self.gap_end, tail_len)
    }

    pub fn insert(&mut self, offset: usize, s: &str) {
        let len = s.len() as isize;
        if len > self.gap_length() {
            self.grow_gap(len);
        }

        self.move_gap(offset as isize);

        let src_ptr = s.as_bytes().as_ptr();
        unsafe {
            libc::memcpy(
                self.gap_start as *mut libc::c_void, 
                src_ptr as *const libc::c_void, 
                len as usize
            );
            self.gap_start = self.gap_start.offset(len);
        }
    }

    pub fn remove(&mut self, range: Range<usize>) {
        let buffer_length = self.buffer_length() as usize;
        assert!(range.start <= buffer_length && range.end <= buffer_length, "Index out of bounds");
        assert!(range.start <= range.end, "Invalid range");

        let s = self.to_string();
        let head = &s[0..range.start];
        let tail = &s[range.end..];

        self.clear();
        self.insert(0, &head);
        self.insert(head.len(), &tail);
    }

    pub fn from_str(s: &str) -> Self {
        let mut buffer = GapBuffer::new(s.len());
        buffer.insert(0, s);
        buffer
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.buffer_length() as usize);
        let start = cmp::min(start, end);
        let s = self.to_string();
        s[start..end].to_string()
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.head(), self.tail())
    }
}

impl Drop for GapBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.buffer_start as *mut libc::c_void);
        }
    }
}

fn string_from_slice(start: *mut u8, len: usize) -> String {
    let mut s = String::with_capacity(len);
    let temp = unsafe{
        String::from_raw_parts(start as *mut u8, len, len)
    };
    s.push_str(&temp);
    mem::forget(temp);
    s
}

fn ptr_to_isize(p: *const u8) -> isize {
    unsafe { mem::transmute::<*const u8, isize>(p) }
}

fn ptr_diff(p: *const u8, q: *const u8) -> isize {
    ptr_to_isize(p) - ptr_to_isize(q)
}