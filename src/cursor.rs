use crate::constants::{TERMINAL_COLS, TERMINAL_ROWS};

pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
    NextLine,
    BOL,
    EOL
}

pub struct Cursor {
    pub row: i32,
    pub col: i32
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    pub fn is_cursor_position(&self, row: f32, col: f32) -> bool {
        let row = row as i32;
        let col = col as i32;
        row == self.row && col == self.col
    }

    pub fn move_to(&mut self, direction: CursorDirection) {
        match direction {
           CursorDirection::Up => {
                if self.row > 0 {
                    self.row -= 1;
                } else {
                    self.move_to(CursorDirection::BOL);
                }
            },
            CursorDirection::Down => {
                if self.row < TERMINAL_ROWS {
                    self.row += 1;
                } else {
                    self.move_to(CursorDirection::EOL);
                }
            }
            CursorDirection::Left => {
                if self.col > 0 {
                    self.col -= 1;
                    // TODO: Move to previous line
                }
            },
            CursorDirection::Right => {
                if self.col < TERMINAL_COLS {
                    self.col += 1;
                } else {
                    self.move_to(CursorDirection::NextLine);
                }
            },
            CursorDirection::NextLine => {
                self.move_to(CursorDirection::Down);
                self.move_to(CursorDirection::BOL);
            },
            CursorDirection::BOL => {
                self.col = 0;
            },
            CursorDirection::EOL => {
                self.col = TERMINAL_COLS;
            },
        }
    }
}
