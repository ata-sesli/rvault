#[derive(Clone, Default, Debug)]
pub struct InputState {
    pub value: String,
    pub cursor_position: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(s: String) -> Self {
        let len = s.len();
        Self {
            value: s,
            cursor_position: len,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            let mut new_pos = self.cursor_position - 1;
            while !self.value.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            let mut new_pos = self.cursor_position + 1;
            while new_pos < self.value.len() && !self.value.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let mut new_pos = self.cursor_position - 1;
            while !self.value.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.value.remove(new_pos);
            self.cursor_position = new_pos;
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    // Helper to ensure cursor stays within bounds if value is modified externally
    pub fn clamp_cursor(&mut self) {
        if self.cursor_position > self.value.len() {
            self.cursor_position = self.value.len();
        }
    }
}
