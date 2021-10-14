pub struct ScreenBuffer {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height],
            width,
            height,
        }
    }

    fn expand_buffer(&mut self) {
        self.data.append(&mut vec![0; self.width * self.height]);
    }

    pub fn lines_count(&self) -> usize {
        self.data.len() / self.width + 1
    }

    pub fn clear(&mut self) {
        self.data = vec![0; self.width * self.height];
    }

    pub fn set_char_at(&mut self, c: u8, row: usize, col: usize) {
        let index = row * self.width + col;
        if index >= self.data.len() {
            self.expand_buffer();
        }
        self.data[index] = c;
    }

    pub fn get_char_at(&self, row: usize, col: usize) -> u8 {
        let index = row * self.width + col;
        if index < self.data.len() {
            return self.data[index];
        } else {
            return 0;
        }
    }
}
