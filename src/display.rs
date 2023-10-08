extern crate sdl2;

use crate::{util::DISPLAY_HEIGHT, util::DISPLAY_WIDTH};

#[derive(Debug)]
pub struct Display {
    // sdl_context: sdl2::Sdl,
    screen: [[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    // screen: Vec<u8>
}

impl Display {
    pub fn new() -> Self {
        Display {
            // sdl_context: sdl2::init().unwrap(),
            screen: [[0; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
            // screen: vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT]
        }
    }

    /**
     * Return true if the value was changed from set to unset in the display, false otherwise
     */
    pub fn set_pixel(&mut self, x: usize, y: usize, val: u8) -> bool {
        let current_val = self.screen[x][y];
        self.screen[x][y] ^= val;
        current_val == 1 && self.screen[x][y] == 0
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        // self.screen[x * y]
        self.screen[x][y]
    }

    pub fn get_screen_state(&self) -> &[[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH] {
        &self.screen
    }

    pub fn clear_screen(&mut self) {
        self.screen = [[0; DISPLAY_HEIGHT]; DISPLAY_WIDTH]
    }
}
