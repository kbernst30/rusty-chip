use std::collections::HashMap;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use crate::{display::Display, memory::Memory, rom::Rom, cpu::Cpu, keyboard::Keyboard};
use crate::util::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

#[derive(Debug)]
pub struct Chip8 {
    cpu: Cpu
}

impl Chip8 {

    pub fn new() -> Self {
        let memory = Memory::new();
        let display = Display::new();
        let keyboard = Keyboard::new();
        Self { cpu: Cpu::new(memory, display, keyboard) }
    }

    pub fn run(&mut self, sdl: &Sdl, canvas: &mut WindowCanvas) {
        println!("RUNNING CHIP8 PROGRAM...");
        let mut event_pump = sdl.event_pump().unwrap();
        let key_map = self.get_key_map();

        'running: loop {
            self.cpu.decrement_timer();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running; },
                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            self.cpu.press_key(*key)
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            self.cpu.release_key(*key);
                        }
                    },
                    _ => {}
                }
            }

            // Tick CPU 10 times per frame
            for _ in 0..10 {
                self.cpu.execute();
            }

            self.draw_screen(canvas);
        }
    }

    pub fn load_rom(&mut self, file: &str) {
        let rom = Rom::new(file);
        self.cpu.load_program(rom);
    }

    fn get_key_map(&self) -> HashMap<Keycode, u8> {
        let mut key_map = HashMap::new();
        key_map.insert(Keycode::X, 0x0);
        key_map.insert(Keycode::Num1, 0x1);
        key_map.insert(Keycode::Num2, 0x2);
        key_map.insert(Keycode::Num3, 0x3);
        key_map.insert(Keycode::Q, 0x4);
        key_map.insert(Keycode::W, 0x5);
        key_map.insert(Keycode::E, 0x6);
        key_map.insert(Keycode::A, 0x7);
        key_map.insert(Keycode::S, 0x8);
        key_map.insert(Keycode::D, 0x9);
        key_map.insert(Keycode::Z, 0xA);
        key_map.insert(Keycode::C, 0xB);
        key_map.insert(Keycode::Num4, 0xC);
        key_map.insert(Keycode::R, 0xD);
        key_map.insert(Keycode::F, 0xE);
        key_map.insert(Keycode::V, 0xF);
        key_map
    }

    fn draw_screen(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.cpu.get_display_pixel(x, y) > 0 {
                    let rect = Rect::new(x as i32, y as i32, 1, 1);
                    canvas.fill_rect(rect).unwrap();
                }
            }
        }

        canvas.present();
    }
}