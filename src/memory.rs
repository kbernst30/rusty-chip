use crate::{rom::Rom, util::FONTS};

#[derive(Debug)]
pub struct Memory {
    data: [u8; 0x1000], // 4096 memory locations (i.e. 0x1000)
}

impl Memory {
    pub fn new() -> Self {
        let mut data: [u8; 0x1000] = [0; 0x1000];

        // Load Font data starting at memory 0 - there is no actual specification for where font data should be...
        // So let's just put it somewhere safe.
        for i in 0..FONTS.len() {
            data[i] = FONTS[i];
        }

        Self { data }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data;
    }

    pub fn load_program(&mut self, rom: Rom) {
        // Programs load into memory at address 0x200
        for i in 0..rom.get_size() {
            self.data[0x200 + i] = rom.get_byte(i);
        }
    }
}