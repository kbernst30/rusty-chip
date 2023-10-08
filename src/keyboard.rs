#[derive(Debug)]
pub struct Keyboard {
    keys: [bool; 16]
}

impl Keyboard {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn press_key(&mut self, key: u8) {
        self.keys[key as usize] = true;
    }

    pub fn release_key(&mut self, key: u8) {
        self.keys[key as usize] = false;
    }

    pub fn get_pressed(&mut self) -> Option<u8> {
        for i in 0..self.keys.len() {
            if self.is_pressed(i as u8) {
                self.release_key(i as u8);
                return Some(i as u8);
            }
        }

        None
    }
}