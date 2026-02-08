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
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.value.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
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
