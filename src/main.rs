use std::env;
use rand::Rng;

use chip8::Chip8;
use util::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub mod chip8;
pub mod cpu;
pub mod display;
pub mod keyboard;
pub mod memory;
pub mod rom;
pub mod util;

fn main() {
    // Setup emulator
    let args: Vec<String> = env::args().collect();
    let rom_file = &args[1];

    let mut chip8 = Chip8::new();
    chip8.load_rom(&rom_file);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rusty Chip", DISPLAY_WIDTH as u32 * 10, DISPLAY_HEIGHT as u32 * 10)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    canvas.set_scale(10f32, 10f32).unwrap();
    chip8.run(&sdl_context, &mut canvas);
}
