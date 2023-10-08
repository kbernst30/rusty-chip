use std::fs;

#[derive(Debug)]
pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn new(file: &str) -> Rom {
        let contents = fs::read(file)
            .expect("Something went wrong reading the file");

        println!("Loading ROM {}", file);

        Rom {
            data: contents
        }
    }

    pub fn get_byte(&self, addr: usize) -> u8 {
        self.data[addr]
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }
}