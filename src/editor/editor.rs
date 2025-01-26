use std::collections::VecDeque;

use crate::editor::{document, terminal, util, highlighting};

pub static BLACK: &str = "\x1B[0;30m";
pub static RED: &str = "\x1B[0;31m";
pub static GREEN: &str = "\x1B[0;32m";
pub static YELLOW: &str = "\x1B[0;33m";
pub static BLUE: &str = "\x1B[0;34m";
pub static MAGENTA: &str = "\x1B[0;35m";
pub static CYAN: &str = "\x1B[0;36m";
pub static WHITE: &str = "\x1B[0;37m";
pub static STATUS_BAR: &str = "\x1B[48;5;237m";
pub static SIDEBAR: &str = "\x1B[48;5;234m";
static ANSI_END: &str = "\x1B[0m";

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(PartialEq, Eq)]
enum EditorState {
    EDIT,
    COMMAND,
}

pub struct Editor {
    terminal: terminal::Terminal,
    document: document::Document,
    cursor_x: usize,
    cursor_y: usize,
    exit: bool,
    display_height: usize,
    display_width: usize,
    display_x: usize,
    display_y: usize,
    file_cols: usize,
    file_rows: usize,
    previous_positions: VecDeque<(usize, usize)>,
    state: EditorState,
    key_pressed: Option<termion::event::Key>,
    status_text: util::GapBuffer,
    sidebar: document::Document,
}
//TODO: Implement the sidebar
//TODO: Implement row numbers
impl Editor {
    pub fn new() -> Result<Self, std::io::Error> {
        let terminal = terminal::Terminal::new()?;
        let document = document::Document::open("src/editor/editor.rs");
        let doc_rows = document.rows();
        let doc_cols = document.cols();
        let display_height = terminal.height - 4; // I have no idea why its -4
        let display_width = (terminal.width as f64 * 0.75) as usize -10;
        Ok(Editor {
            terminal,
            document,
            cursor_x: 0,
            cursor_y: display_height/2,
            exit: false,
            display_height,
            display_width,
            display_x: 0,
            display_y: 0,
            file_cols: doc_cols,
            file_rows: doc_rows,
            previous_positions: VecDeque::new(),
            state: EditorState::EDIT,
            key_pressed: Some(termion::event::Key::Null),
            status_text: util::GapBuffer::from_str(""),
            sidebar: document::Document::open("test.txt"),
        })
    }

    pub fn run(&mut self) {
        self.terminal.clear_screen();
        loop {
            if let Err(err) = self.refresh_screen() {
                self.kill(err);
            }
            if let Err(err) = self.process_keys() {
                self.kill(err);
            }
            if self.exit {
                break;
            }
        }
    }
    // TODO: FIXME: THE TEXT IS RENDERING ONE LINE TOO LATE
    pub fn render_new(&self) {
        // Calculate line number width dynamically
        let line_number_width = if self.file_rows == 0 {
            1
        } else {
            (self.file_rows.checked_ilog10().unwrap_or(0) + 1) as usize
        }.max(3); // Minimum 3 digits for alignment

        // Calculate scroll proportions
        let (scroll_height, scroll_start) = self.calculate_scrollbar();
        let total_sidebar_width = self.terminal.width - self.display_width - 2;

        for line in 0..self.display_height {
            self.terminal.clear_line();
            let main_row = self.display_y + line;
            let is_scroll_thumb = line >= scroll_start && line < scroll_start + scroll_height;

            // Main content
            let main_content = if main_row < self.document.rows() {
                self.document.row(main_row).unwrap()
                    .render(self.display_x, self.display_x + self.display_width)
            } else {
                "~".to_string()
            };

            // Line numbers (right-aligned)
            let line_num = if main_row < self.file_rows {
                format!("{:>width$}", main_row + 1, width = line_number_width)
            } else {
                "~".repeat(line_number_width)
            };

            // Sidebar content
            let sidebar_content = if line < self.sidebar.rows() {
                self.sidebar.row(line).unwrap()
                    .render(0, total_sidebar_width - line_number_width - 3)
            } else {
                String::new()
            };

            // Construct line with proper formatting
            let mut rendered = String::new();
            
            // Main content area
            rendered.push_str(&main_content);
            rendered.push_str(&" ".repeat(self.display_width.saturating_sub(main_content.chars().count())));
            
            // Separator and line numbers
            rendered.push_str(&format!(
                "{SIDEBAR}┆{}{} {}{}{}┃{ANSI_END}\r",
                line_num,
                if is_scroll_thumb { "█" } else { "│" },
                sidebar_content,
                " ".repeat(total_sidebar_width.saturating_sub(sidebar_content.chars().count() + line_number_width + 2)),
                STATUS_BAR
            ));

            println!("{}", rendered);
        }

        // Status bars
        self.status_bar(0, &termion::event::Key::Null);
        self.status_bar(1, &termion::event::Key::Null);
        self.status_bar(2, self.key_pressed.as_ref().unwrap());
    }

    fn calculate_scrollbar(&self) -> (usize, usize) {
        let total = self.file_rows.max(1);
        let visible = self.display_height;
        let ratio = visible as f32 / total as f32;
        
        let thumb_height = (visible as f32 * ratio).ceil() as usize;
        let thumb_height = thumb_height.clamp(1, visible);
        
        let max_scroll = total.saturating_sub(visible);
        let thumb_pos = if max_scroll > 0 {
            (self.display_y * (visible - thumb_height)) / max_scroll
        } else {
            0
        };

        (thumb_height, thumb_pos)
    }

    pub fn render_old(&self) {
        for line in 0..self.display_height {
            self.terminal.clear_line();
            let main_row_num = self.display_y + line;
            let sidebar_row_num = line;
    
            // Render main document content
            let main_content = if main_row_num < self.document.rows() {
                let row = self.document.row(main_row_num).unwrap();
                row.render(self.display_x, self.display_x + self.display_width)
            } else {
                "~".to_string()
            };
    
            // Render sidebar content
            let sidebar_content = if sidebar_row_num < self.sidebar.rows() {
                let row = self.sidebar.row(sidebar_row_num).unwrap();
                row.render(0, self.terminal.width - self.display_width - 3)
            } else {
                "~".to_string()
            };
    
            // Construct the full line with padding
            let mut rendered = String::new();
            rendered.push_str(&main_content);
            // Add padding to main content
            for _ in main_content.chars().count()..self.display_width {
                rendered.push(' ');
            }
            rendered.push_str(STATUS_BAR);
            rendered.push('┊');
            rendered.push_str(&sidebar_content);
            // Add padding to sidebar content
            for _ in sidebar_content.chars().count()..(self.terminal.width - self.display_width - 3) {
                rendered.push(' ');
            }
            rendered.push('┃');
            rendered.push_str(ANSI_END);
    
            println!("{}\r", rendered);
        }
    
        // Status bars
        self.status_bar(0, &termion::event::Key::Null);
        self.status_bar(1, &termion::event::Key::Null);
        self.status_bar(2, self.key_pressed.as_ref().unwrap());
    }

    pub fn process_keys(&mut self) -> Result<(), std::io::Error> {
        let key = self.terminal.read_key()?;
        self.key_pressed = Some(key);
        match key {
            termion::event::Key::Esc => {
                self.escape();
            }, 
            termion::event::Key::Ctrl('q') => {
                self.exit = true;
            },
            termion::event::Key::Ctrl('s') => {
                self.save()?;
            },
            termion::event::Key::Up | termion::event::Key::Down | termion::event::Key::Left | termion::event::Key::Right | termion::event::Key::Home | termion::event::Key::End => {
                self.move_cursor(key);
            },
            termion::event::Key::Backspace => {
                self.backspace();
            },
            termion::event::Key::Delete => {
                self.delete();
            },
            termion::event::Key::Char('\n') => {
                self.enter();
            },
            termion::event::Key::Char(_) => {
                self.insert_text(key);
            },
            _ => (),
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.document.save()
    }

    pub fn insert_text(&mut self, key: termion::event::Key) {
        match self.state {
            EditorState::EDIT => {
                self.insert_text_edit(key);
            },
            EditorState::COMMAND => {
                self.insert_text_command(key);
            },
        }
    }

    fn insert_text_edit(&mut self, key: termion::event::Key) {
        match key {
            termion::event::Key::Char(c) => {
                let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
                row.insert_char(self.cursor_x, c);
                self.cursor_x = self.cursor_x.saturating_add(1);
            },
            _ => (),
        }
    }

    fn insert_text_command(&mut self, key: termion::event::Key) {
        match key {
            termion::event::Key::Char(c) => {
                self.status_text.insert_char(self.cursor_x, c);
                self.cursor_x = self.cursor_x.saturating_add(1);
            },
            _ => (),
        }
    }

    pub fn escape(&mut self) {
        match self.state {
            EditorState::EDIT => {
                self.state = EditorState::COMMAND;
                self.cursor_x = 0;
            }
            EditorState::COMMAND => {
                self.state = EditorState::EDIT;
            }
        }
    }

    pub fn backspace(&mut self) {
        match self.state {
            EditorState::EDIT => {
                self.backspace_edit();
            },
            EditorState::COMMAND => {
                self.backspace_command();
            },
        }
    }


    // FIXME: Backspace command is not working, when backspace is pressed a big chunk of trailing whitespace is deleted.
    fn backspace_command(&mut self) {
        if self.cursor_x == 0 { return }
        self.status_text.remove_char(self.cursor_x);
        self.cursor_x = self.cursor_x.saturating_sub(1);
    }

    //TODO: Pressing backspace and insertion in rapid succession causes the next character to be deleted also.
    //TODO: does not handle backspace at the beginning of the line
    fn backspace_edit(&mut self) {
        let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
        if self.cursor_x > 0 {
            row.remove_char(self.display_x + self.cursor_x - 1);
            if self.display_x == 0 {
                self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            else {
                self.display_x = self.display_x.saturating_sub(1);
            }
        }
        else if self.cursor_x == 0 {
            // line merging with previous line
            let current_row = self.display_y + self.cursor_y;
            if current_row > 0 {
                // get current line content
                let current_content = self.document.rows[current_row].to_string();
                
                // Remove current line
                self.document.rows.remove(current_row);
                
                // Get previous line references
                let prev_row = current_row - 1;
                if let Some(prev_line) = self.document.rows.get_mut(prev_row) {
                    // Get previous line's character count and byte length
                    let prev_char_len = prev_line.str_len();
                    let prev_byte_len = prev_line.buffer_length();
                    
                    // Insert current content at end of previous line
                    prev_line.insert(prev_byte_len, current_content.as_bytes());
                    
                    // Update cursor position
                    if self.display_y == 0 {
                        self.cursor_y = self.cursor_y.saturating_sub(1);
                    }
                    else {
                        self.display_y = self.display_y.saturating_sub(1);
                    }
                    self.cursor_x = prev_char_len;
                    
                    // Update document dimensions
                    self.file_rows = self.document.rows();
                    self.file_cols = self.document.cols();
                }
            }
        }
    }

    pub fn delete(&mut self) {
        match self.state {
            EditorState::EDIT => {
                self.delete_edit();
            },
            EditorState::COMMAND => {
                self.delete_command();
            },
        }
    }

    // TODO: Implement delete command because its not working
    fn delete_command(&mut self) {
        self.status_text.remove_char(self.cursor_x);
    }

    fn delete_edit(&mut self) {
        let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
        if self.cursor_x < row.str_len() {
            row.remove_char(self.display_x + self.cursor_x);
        }
        else if self.cursor_x == row.str_len() {
            // line merging with next line
            let current_row = self.display_y + self.cursor_y;
            if current_row < self.file_rows - 1 {
                // get current line content
                let current_content = self.document.rows[current_row].to_string();
                
                // Remove current line
                self.document.rows.remove(current_row);
                
                // Get next line references
                let next_row = current_row;
                if let Some(next_line) = self.document.rows.get_mut(next_row) {
                    // Get next line's character count and byte length
                    let next_char_len = next_line.str_len();
                    let next_byte_len = next_line.buffer_length();
                    
                    // Insert current content at beginning of next line
                    next_line.insert(0, current_content.as_bytes());
                    
                    // Update document dimensions
                    self.file_rows = self.document.rows();
                    self.file_cols = self.document.cols();
                }
            }
        }
    }

    pub fn enter(&mut self) {
        match self.state {
            EditorState::EDIT => {
                self.enter_edit();
            },
            EditorState::COMMAND => {
                self.enter_command();
            },
        }
    }

    fn enter_command(&mut self) {
        let text = format!("Executed {}", self.status_text.to_string());
        self.status_text = util::GapBuffer::from_str(&text);
        self.cursor_x = 0;
    }

    fn enter_edit(&mut self) {
        let actual_row = self.display_y + self.cursor_y;
        let current_col = self.cursor_x;
        
        self.document.new_line(actual_row, current_col);

        self.file_rows = self.document.rows();
        self.file_cols = self.document.cols();
        
        self.cursor_x = 0;
        self.display_y = self.display_y.saturating_add(1);
    }

    pub fn move_cursor(&mut self, key: termion::event::Key) {
        match self.state {
            EditorState::EDIT => {
                self.move_cursor_edit(key);
            },
            EditorState::COMMAND => {
                self.move_cursor_command(key);
            },
        }
    }

    pub fn move_cursor_command(&mut self, key: termion::event::Key) {
    }

    // TODO: Fix the cursor state save where it should return to previous position
    // Error occurs when moving up and down not corresponding to the previous position.
    pub fn move_cursor_edit(&mut self, key: termion::event::Key) {
        let min_y = self.display_height/2;
        let min_x = 0;
        let max_y = self.file_rows - self.display_height/2;
        let max_x = self.file_cols;

        match key {
            termion::event::Key::Up => {
                    self.display_y = self.display_y.saturating_sub(1);
                    if self.display_y == 0 {
                        self.cursor_y = self.cursor_y.saturating_sub(1);
                    }
                    let curr_row_len = self.document.row(self.display_y+self.cursor_y).unwrap().str_len();
                    if self.cursor_x <= curr_row_len {
                        match self.previous_positions.pop_back() {
                            Some((x, _)) => {
                                self.cursor_x = x;
                            },
                            None => (),
                        }
                    }
                    if self.cursor_x > curr_row_len {
                        self.previous_positions.push_back((self.cursor_x, self.cursor_y + 1));
                        self.cursor_x = curr_row_len;
                    }
            },
            termion::event::Key::Down => {
                if self.display_y == max_y - 1 { // -1 because of 0 indexing
                    return;
                }

                if self.display_y < max_y && self.cursor_y == min_y {
                    self.display_y = self.display_y.saturating_add(1);
                }
                if self.display_y == 0 {
                    self.cursor_y = self.cursor_y.saturating_add(1);
                }
                let curr_row_len = self.document.row(self.display_y + self.cursor_y).unwrap().str_len();
                if self.cursor_x <= curr_row_len {
                    match self.previous_positions.pop_back() {
                        Some((x, _)) => {
                            self.cursor_x = x;
                        },
                        None => (),
                    }
                }
                if self.cursor_x > curr_row_len {
                    self.previous_positions.push_back((self.cursor_x, self.cursor_y - 1));
                    self.cursor_x = curr_row_len;
                }
            },
            termion::event::Key::Left => {
                if self.cursor_x > min_x {
                    self.cursor_x = self.cursor_x.saturating_sub(1);
                }
                if self.cursor_x == min_x && self.display_x > 0 {
                    self.display_x = self.display_x.saturating_sub(1);
                }
                self.previous_positions = VecDeque::new();
            },
            termion::event::Key::Right => {
                if self.cursor_x == self.document.row(self.display_y + self.cursor_y).unwrap().str_len() {
                    return;
                }
                if self.cursor_x < self.display_width {
                    self.cursor_x = self.cursor_x.saturating_add(1);
                }
                if self.cursor_x == self.display_width && self.display_x < max_x {
                    self.display_x = self.display_x.saturating_add(1);
                }
                self.previous_positions = VecDeque::new();
            },
            termion::event::Key::Home => {
                self.cursor_x = 0;
                self.display_x = 0;
                self.previous_positions = VecDeque::new();
            },
            termion::event::Key::End => {
                self.cursor_x = self.document.row(self.display_y + self.cursor_y).unwrap().str_len();
                self.display_x = self.cursor_x.saturating_sub(self.display_width);
                self.previous_positions = VecDeque::new();
            },
            _ => (),
        }
    }

    pub fn status_bar(&self, row: usize, key: &termion::event::Key) {
        let mut status = String::new();

        match row {
            0 => {
                status.push_str(format!("File: {} - {} lines", self.document.file_path, self.file_rows).as_str());
            },
            1 => {
                status.push_str(format!("{}", self.status_text).as_str());
            },
            2 => {
                match self.state {
                    EditorState::EDIT => {
                        status.push_str("EDIT MODE");
                    },
                    EditorState::COMMAND => {
                        status.push_str("COMMAND MODE");
                    },
                }
                match key {
                    termion::event::Key::Ctrl('s') => {
                        status.push_str(format!("File saved to {}.", self.document.file_path).as_str());
                    },
                    _ => {},
                }
            },
            _ => {},
        }
        
        for _ in 0..self.terminal.width - status.to_string().chars().count() {
            status.push(' ');
        }
        print!("{}{}{}", STATUS_BAR, status, ANSI_END);
    }

    pub fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.hide_cursor();
        self.terminal.cursor_position(0, self.display_height/2);

        if self.exit {
            self.terminal.clear_screen();
            println!("Goodbye.\r");
        } else {
            self.render_old();
            match self.state {
                EditorState::EDIT => {
                    self.terminal.cursor_position(self.cursor_x, self.cursor_y);
                },
                EditorState::COMMAND => {
                    self.terminal.cursor_position(self.cursor_x, self.terminal.height - 2);
                },
            }
        }
        self.terminal.show_cursor();
        self.terminal.flush()
    }

    fn kill(&self, err: std::io::Error) {
        self.terminal.clear_screen();
        panic!("Error: {}", err);
    }
}