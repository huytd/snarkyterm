use crate::constants::{TERMINAL_COLS, TERMINAL_ROWS};

pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
    NextLine,
    BOL,
    EOL,
    BOF
}

pub struct Cursor {
    pub row: usize,
    pub col: usize
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
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
                self.row += 1;
            }
            CursorDirection::Left => {
                if self.col > 0 {
                    self.col -= 1;
                    // TODO: Move to previous line
                }
            },
            CursorDirection::Right => {
                if self.col < TERMINAL_COLS as usize {
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
                self.col = TERMINAL_COLS as usize;
            },
            CursorDirection::BOF => {
                self.row = 0;
                self.col = 0;
            }
        }
    }
}
