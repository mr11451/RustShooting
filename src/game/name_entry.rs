pub const MATRIX_COLS: usize = 7;
pub const MATRIX_ROWS: usize = 6;
pub const MAX_NAME_LEN: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridKey {
    Char(u8),
    Left,
    Right,
    Delete,
    Space,
}

pub const CHAR_MATRIX: [[GridKey; MATRIX_COLS]; MATRIX_ROWS] = [
    [
        GridKey::Char(b'A'),
        GridKey::Char(b'B'),
        GridKey::Char(b'C'),
        GridKey::Char(b'D'),
        GridKey::Char(b'F'),
        GridKey::Char(b'E'),
        GridKey::Char(b'G'),
    ],
    [
        GridKey::Char(b'H'),
        GridKey::Char(b'I'),
        GridKey::Char(b'J'),
        GridKey::Char(b'K'),
        GridKey::Char(b'L'),
        GridKey::Char(b'M'),
        GridKey::Char(b'N'),
    ],
    [
        GridKey::Char(b'O'),
        GridKey::Char(b'P'),
        GridKey::Char(b'Q'),
        GridKey::Char(b'R'),
        GridKey::Char(b'S'),
        GridKey::Char(b'T'),
        GridKey::Char(b'U'),
    ],
    [
        GridKey::Char(b'V'),
        GridKey::Char(b'W'),
        GridKey::Char(b'X'),
        GridKey::Char(b'Y'),
        GridKey::Char(b'Z'),
        GridKey::Char(b'.'),
        GridKey::Char(b'-'),
    ],
    [
        GridKey::Char(b'0'),
        GridKey::Char(b'1'),
        GridKey::Char(b'2'),
        GridKey::Char(b'3'),
        GridKey::Char(b'4'),
        GridKey::Char(b'5'),
        GridKey::Char(b'6'),
    ],
    [
        GridKey::Char(b'7'),
        GridKey::Char(b'8'),
        GridKey::Char(b'9'),
        GridKey::Left,
        GridKey::Right,
        GridKey::Delete,
        GridKey::Space,
    ],
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameEntryState {
    pub cursor_col: usize,
    pub cursor_row: usize,
    pub name: [u8; MAX_NAME_LEN],
    pub char_index: usize,
    pub completed: bool,
    pub prev_move_x: i8,
    pub prev_move_y: i8,
    pub prev_action: bool,
}

impl Default for NameEntryState {
    fn default() -> Self {
        Self {
            cursor_col: 0,
            cursor_row: 0,
            name: [b' '; MAX_NAME_LEN],
            char_index: 0,
            completed: false,
            prev_move_x: 0,
            prev_move_y: 0,
            prev_action: false,
        }
    }
}

impl NameEntryState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn current_key(&self) -> GridKey {
        CHAR_MATRIX[self.cursor_row][self.cursor_col]
    }

    pub fn move_cursor(&mut self, dx: i8, dy: i8) {
        if dx > 0 {
            self.cursor_col = (self.cursor_col + 1) % MATRIX_COLS;
        } else if dx < 0 {
            self.cursor_col = (self.cursor_col + MATRIX_COLS - 1) % MATRIX_COLS;
        }

        if dy > 0 {
            self.cursor_row = (self.cursor_row + 1) % MATRIX_ROWS;
        } else if dy < 0 {
            self.cursor_row = (self.cursor_row + MATRIX_ROWS - 1) % MATRIX_ROWS;
        }
    }

    pub fn select_current(&mut self) -> bool {
        match self.current_key() {
            GridKey::Char(ch) => {
                if self.char_index < MAX_NAME_LEN {
                    self.name[self.char_index] = ch;
                    self.char_index += 1;
                    if self.char_index >= MAX_NAME_LEN {
                        self.completed = true;
                        return true;
                    }
                }
            }
            GridKey::Space => {
                if self.char_index < MAX_NAME_LEN {
                    self.name[self.char_index] = b' ';
                    self.char_index += 1;
                    if self.char_index >= MAX_NAME_LEN {
                        self.completed = true;
                        return true;
                    }
                }
            }
            GridKey::Delete => {
                if self.char_index > 0 {
                    self.char_index -= 1;
                    self.name[self.char_index] = b' ';
                }
            }
            GridKey::Left => {
                if self.char_index > 0 {
                    self.char_index -= 1;
                }
            }
            GridKey::Right => {
                if self.char_index < MAX_NAME_LEN {
                    self.char_index += 1;
                    if self.char_index >= MAX_NAME_LEN {
                        self.completed = true;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn update_input(&mut self, move_x: i8, move_y: i8, action: bool) -> bool {
        let step_x = if move_x != 0 && self.prev_move_x == 0 {
            move_x
        } else {
            0
        };
        let step_y = if move_y != 0 && self.prev_move_y == 0 {
            move_y
        } else {
            0
        };
        self.prev_move_x = move_x;
        self.prev_move_y = move_y;

        if step_x != 0 || step_y != 0 {
            self.move_cursor(step_x, step_y);
        }

        let pressed_action = action && !self.prev_action;
        self.prev_action = action;

        if pressed_action {
            self.select_current()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_wraps_around_matrix() {
        let mut state = NameEntryState::default();
        assert_eq!(state.cursor_col, 0);
        assert_eq!(state.cursor_row, 0);

        state.move_cursor(-1, 0);
        assert_eq!(state.cursor_col, MATRIX_COLS - 1);

        state.move_cursor(0, -1);
        assert_eq!(state.cursor_row, MATRIX_ROWS - 1);

        state.move_cursor(1, 1);
        assert_eq!(state.cursor_col, 0);
        assert_eq!(state.cursor_row, 0);
    }

    #[test]
    fn selecting_characters_fills_name_up_to_three() {
        let mut state = NameEntryState::default();
        // At (0, 0) is 'A'
        assert_eq!(state.current_key(), GridKey::Char(b'A'));
        assert!(!state.select_current());
        assert_eq!(state.name[0], b'A');
        assert_eq!(state.char_index, 1);

        // Move to (1, 0) which is 'B'
        state.move_cursor(1, 0);
        assert_eq!(state.current_key(), GridKey::Char(b'B'));
        assert!(!state.select_current());
        assert_eq!(state.name[1], b'B');
        assert_eq!(state.char_index, 2);

        // Move to (2, 0) which is 'C'
        state.move_cursor(1, 0);
        assert_eq!(state.current_key(), GridKey::Char(b'C'));
        let finished = state.select_current();
        assert!(finished);
        assert_eq!(state.name, *b"ABC");
        assert!(state.completed);
    }

    #[test]
    fn delete_and_space_operations() {
        let mut state = NameEntryState::default();
        state.select_current(); // 'A'
        assert_eq!(state.name[0], b'A');

        // Move to delete at row 5, col 5
        state.cursor_row = 5;
        state.cursor_col = 5;
        assert_eq!(state.current_key(), GridKey::Delete);
        state.select_current();
        assert_eq!(state.char_index, 0);
        assert_eq!(state.name[0], b' ');
    }
}
