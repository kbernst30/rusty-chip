use rand::random;
use crate::{display::Display, memory::Memory, rom::Rom, util::{get_bit_val, DISPLAY_WIDTH, DISPLAY_HEIGHT}, keyboard::Keyboard};

#[derive(Debug)]
pub struct Cpu {
    program_counter: u16,
    v_registers: [u8; 16], // There are 16 general purpose V registers, each capable of storing 8 bits
    i_register: u16, // Address register can hold up to two bytes, but in reality can't be more than 12 bits
    delay_timer: u8,
    sound_timer: u8,
    memory: Memory,
    display: Display,
    stack: Vec<u16>,
    keyboard: Keyboard,
    halted: bool,
}

impl Cpu {
    pub fn new(memory: Memory, display: Display, keyboard: Keyboard) -> Self {
        Self {
            program_counter: 0x200,
            v_registers: [0; 16],
            i_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            memory,
            display,
            stack: Vec::new(),
            keyboard,
            halted: false
        }
    }

    pub fn load_program(&mut self, rom: Rom) {
        self.memory.load_program(rom);
    }

    pub fn get_display_state(&self) -> &[[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH] {
        &self.display.get_screen_state()
    }

    pub fn get_display_pixel(&self, x: usize, y: usize) -> u8 {
        self.display.get_pixel(x, y)
    }

    pub fn press_key(&mut self, key: u8) {
        self.keyboard.press_key(key);
    }

    pub fn release_key(&mut self, key: u8) {
        self.keyboard.release_key(key);
    }

    pub fn execute(&mut self) {
        if self.halted {
            self.program_counter -= 2;
        }

        let pc = self.program_counter.clone();
        let instr = self.get_instruction();
        let prefix = (instr >> 12) & 0xF;

        // println!("V - {:?}, I - 0x{:04X}", self.v_registers, self.i_register);
        // println!("Executing PC 0x{:04X} - 0x{:04X}", pc, instr);
        // for row in self.display.get_screen_state() {
        //     println!("{:?}", row);
        // }

        match prefix {
            0x0 => self.do_execute_zero_instr(instr),
            0x1 => self.do_jump_to_address(instr),
            0x2 => self.do_execute_subroutine(instr),
            0x3 => self.do_skip_instruction(instr, (instr & 0xFF) as u8, true),
            0x4 => self.do_skip_instruction(instr, (instr & 0xFF) as u8, false),
            0x5 => self.do_skip_instruction(instr, self.v_registers[((instr >> 4) & 0xF) as usize], true),
            0x6 => self.do_store_number_in_v(instr),
            0x7 => self.do_add_value_to_vx(instr),
            0x8 => self.do_execute_8_instr(instr),
            0x9 => self.do_skip_instruction(instr, self.v_registers[((instr >> 4) & 0xF) as usize], false),
            0xA => self.do_store_memory_address_in_i(instr),
            0xB => self.program_counter = (instr & 0xFFF) + (self.v_registers[0x0] as u16),
            0xC => self.do_set_vx_to_random_with_mask(instr),
            0xD => self.do_draw_sprite(instr),
            0xE => self.do_execute_e_instr(instr),
            0xF => self.do_execute_f_instr(instr),
            _   => panic!("Operation not found - 0x{:04X}", instr)
        };
    }

    pub fn decrement_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn get_instruction(&mut self) -> u16 {
        let hi_byte = self.memory.read(self.program_counter) as u16;
        let lo_byte = self.memory.read(self.program_counter + 1) as u16;

        // Increment Program Counter - 2 spots as each instruction is 2 bytes in memory
        self.program_counter += 2;

        // Chip 8 is Big endian
        (hi_byte << 8) | lo_byte
    }

    fn do_execute_zero_instr(&mut self, instr: u16) {
        let lower = instr & 0xFF;

        match lower {
            0xE0 => self.display.clear_screen(),
            0xEE => self.do_return_from_subroutine(),
            _    => panic!("Operation not found - 0x{:04X}", instr)
        }
    }

    fn do_execute_8_instr(&mut self, instr: u16) {
        let x = (instr >> 8) & 0xF;
        let y = (instr >> 4) & 0xF;

        match instr & 0xF {
            0x0 => self.v_registers[x as usize] = self.v_registers[y as usize],
            0x1 => self.v_registers[x as usize] |= self.v_registers[y as usize],
            0x2 => self.v_registers[x as usize] &= self.v_registers[y as usize],
            0x3 => self.v_registers[x as usize] ^= self.v_registers[y as usize],
            0x4 => self.do_add_vy_to_vx(instr),
            0x5 => self.do_subtract_vy_from_vx(instr),
            0x6 => self.do_shift_bit_right(instr),
            0x7 => self.do_subtract_vx_from_vy(instr),
            0xE => self.do_shift_bit_left(instr),
            _   => panic!("Operation not found - 0x{:04X}", instr)
        }
    }

    fn do_execute_e_instr(&mut self, instr: u16) {
        let lower = instr & 0xFF;

        match lower {
            0x9E => self.do_key_pressed_skip(instr),
            0xA1 => self.do_key_not_pressed_skip(instr),
            _    => panic!("Operation not found - 0x{:04X}", instr)
        }
    }

    fn do_execute_f_instr(&mut self, instr: u16) {
        let lower = instr & 0xFF;

        match lower {
            0x07 => self.do_store_delay_timer(instr),
            0x0A => self.do_await_key_press(instr),
            0x15 => self.do_set_delay_timer(instr),
            0x18 => self.do_set_sound_timer(instr),
            0x1E => self.do_add_to_i_register(instr),
            0x29 => self.do_set_i_to_sprite_location(instr),
            0x33 => self.do_store_binary_coded_decimal(instr),
            0x55 => self.do_store_v_registers(instr),
            0x65 => self.do_fill_v_registers(instr),
            _    => panic!("Operation not found - 0x{:04X}", instr)
        }
    }

    fn do_jump_to_address(&mut self, instr: u16) {
        self.program_counter = instr & 0xFFF;
    }

    fn do_return_from_subroutine(&mut self) {
        // self.program_counter = self.stack.pop();
        self.program_counter = match self.stack.pop() {
            Some(val) => val,
            None => panic!("Stack was empty!")
        }
    }

    fn do_execute_subroutine(&mut self, instr: u16) {
        let addr = instr & 0xFFF;
        self.stack.push(self.program_counter);
        self.program_counter = addr;
    }

    fn do_skip_instruction(&mut self, instr: u16, value: u8, should_equal_val: bool) {
        let register_x = (instr >> 8) & 0xF;

        let condition = match should_equal_val {
            true  => self.v_registers[register_x as usize] == value,
            false => self.v_registers[register_x as usize] != value
        };

        // Skip next instruction IFF value in register_x = value
        self.program_counter = match condition {
            true  => self.program_counter + 2,
            false => self.program_counter
        }
    }

    fn do_store_number_in_v(&mut self, instr: u16) {
        let v = ((instr >> 8) & 0xF) as usize;
        let value = (instr & 0xFF) as u8;
        self.v_registers[v] = value;
    }

    fn do_add_value_to_vx(&mut self, instr: u16) {
        let register_x = ((instr >> 8) & 0xF) as usize;
        let value = (instr & 0xFF) as u8;
        self.v_registers[register_x] = self.v_registers[register_x].wrapping_add(value);
    }

    fn do_add_vy_to_vx(&mut self, instr: u16) {
        let register_x = ((instr >> 8) & 0xF) as usize;
        let register_y = ((instr >> 4) & 0xF) as usize;

        // Set if carry
        let carry = match self.v_registers[register_x].checked_add(self.v_registers[register_y]) {
            Some(_) => 0,
            None    => 1
        };

        self.v_registers[register_x] = self.v_registers[register_x].wrapping_add(self.v_registers[register_y]);
        self.v_registers[0xF] = carry;
    }

    fn do_subtract_vy_from_vx(&mut self, instr: u16) {
        let register_x = ((instr >> 8) & 0xF) as usize;
        let register_y = ((instr >> 4) & 0xF) as usize;

        // Set if not borrow
        let borrow = match self.v_registers[register_x].checked_sub(self.v_registers[register_y]) {
            Some(_) => 1,
            None    => 0
        };

        self.v_registers[register_x] = self.v_registers[register_x].wrapping_sub(self.v_registers[register_y]);
        self.v_registers[0xF] = borrow;
    }

    fn do_subtract_vx_from_vy(&mut self, instr: u16) {
        let register_x = ((instr >> 8) & 0xF) as usize;
        let register_y = ((instr >> 4) & 0xF) as usize;

        // Set if not borrow
        let borrow = match self.v_registers[register_y].checked_sub(self.v_registers[register_x]) {
            Some(_) => 1,
            None    => 0
        };

        self.v_registers[register_x] = self.v_registers[register_y].wrapping_sub(self.v_registers[register_x]);
        self.v_registers[0xF] = borrow;
    }

    fn do_shift_bit_right(&mut self, instr: u16) {
        // Shift VY right one bit and store in VX. VF is set to least significant bit before the change
        // VY is unchanged
        let register_x = ((instr >> 8) & 0xF) as usize;
        let register_y = ((instr >> 4) & 0xF) as usize;
        let val = self.v_registers[register_y];

        self.v_registers[register_x] = val >> 1;
        self.v_registers[0xF] = val & 0x1;
    }

    fn do_shift_bit_left(&mut self, instr: u16) {
        // Shift VY left one bit and store in VX. VF is set to most significant bit before the change
        // VY is unchanged
        let register_x = ((instr >> 8) & 0xF) as usize;
        let register_y = ((instr >> 4) & 0xF) as usize;
        let val = self.v_registers[register_y];

        self.v_registers[register_x] = val << 1;
        self.v_registers[0xF] = val >> 7;
    }

    fn do_store_memory_address_in_i(&mut self, instr: u16) {
        let addr = instr & 0xFFF;
        self.i_register = addr;
    }

    fn do_set_vx_to_random_with_mask(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        let value = (instr & 0xFF) as u8;
        let random_val = random::<u8>();
        self.v_registers[register_x as usize] = random_val & value;
    }

    fn do_draw_sprite(&mut self, instr: u16) {
        // DXYN - Draw sprite at position VX, VY with N bytes of sprite data
        // Sprite data starts at the address stored in register I
        let register_x = (instr >> 8) & 0xF;
        let register_y = (instr >> 4) & 0xF;
        let end_addr = self.i_register + (instr & 0xF);

        let x_pos: u8 = self.v_registers[register_x as usize];
        let mut y_pos: u8 = self.v_registers[register_y as usize];

        // let mut pixels_unset = false;
        let mut any_flipped = false;
        for i in self.i_register..end_addr {
            // Sprite data will be one byte. Draw 8 bits starting from the x_pos
            // Each "sprite data" we read will be at the next y_pos
            let sprite_data = self.memory.read(i);
            for x in (0..8).rev() {
                let val = get_bit_val(sprite_data, x);
                // let x_idx = ((x_pos + 7 - x) as usize) % DISPLAY_WIDTH;
                let x_idx = ((x_pos as usize) + 7 - (x as usize)) % DISPLAY_WIDTH;
                let y_idx = (y_pos as usize) % DISPLAY_HEIGHT;
                any_flipped = self.display.set_pixel(x_idx, y_idx, val) || any_flipped;
            }

            y_pos = y_pos.wrapping_add(1);
        }

        // Set VF to 1 if any pixel was flipped in the display, 0 otherwise
        self.v_registers[0xF] = any_flipped as u8;
    }

    fn do_key_pressed_skip(&mut self, instr: u16) {
        // Skip next instruction if key corresponding to hex value in X is pressed
        let register_x = (instr >> 8) & 0xF;
        self.program_counter = match self.keyboard.is_pressed(self.v_registers[register_x as usize]) {
            true  => self.program_counter + 2,
            false => self.program_counter
        }
    }

    fn do_key_not_pressed_skip(&mut self, instr: u16) {
        // Skip next instruction if key corresponding to hex value in X is not pressed
        let register_x = (instr >> 8) & 0xF;
        self.program_counter = match self.keyboard.is_pressed(self.v_registers[register_x as usize]) {
            true  => self.program_counter,
            false => self.program_counter + 2
        }
    }

    fn do_store_delay_timer(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        self.v_registers[register_x as usize] = self.delay_timer;
    }

    fn do_await_key_press(&mut self, instr: u16) {
        // Await a key press and store the result in register X
        let register_x = (instr >> 8) & 0xF;
        self.halted = true;
        match self.keyboard.get_pressed() {
            Some(key) => {
                self.halted = false;
                self.v_registers[register_x as usize] = key;
            },
            None => ()
        }
    }

    fn do_set_delay_timer(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        self.delay_timer = self.v_registers[register_x as usize];
    }

    fn do_set_sound_timer(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        self.sound_timer = self.v_registers[register_x as usize];
    }

    fn do_add_to_i_register(&mut self, instr: u16) {
        // Add the value of VX to I
        let register_x = (instr >> 8) & 0xF;
        self.i_register = self.i_register.wrapping_add(self.v_registers[register_x as usize] as u16);
    }

    fn do_set_i_to_sprite_location(&mut self, instr: u16) {
        // Sets I to location of the sprite for character in VX - characters 0-F represented by 4x5 font
        let register_x = (instr >> 8) & 0xF;
        let value = self.v_registers[register_x as usize];
        self.i_register = (value as u16) * 5;
    }

    fn do_store_binary_coded_decimal(&mut self, instr: u16) {
        // Store binary coded decimal equivalent of value in register X at I, I + 1, and I + 2
        let register_x = (instr >> 8) & 0xF;
        let value = self.v_registers[register_x as usize];

        // BCD can be up to 3 digits (max value of 255).
        let most_significant_digit = value / 100;
        let middle_digit = (value / 10) % 10;
        let least_significant_digit = value % 10;

        self.memory.write(self.i_register, most_significant_digit);
        self.memory.write(self.i_register + 1, middle_digit);
        self.memory.write(self.i_register + 2, least_significant_digit);
    }

    fn do_store_v_registers(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        for i in 0..=register_x {
            self.memory.write(self.i_register + i, self.v_registers[i as usize]);
        }

        // Set I Register to I + X + 1
        self.i_register = self.i_register + register_x + 1;
    }

    fn do_fill_v_registers(&mut self, instr: u16) {
        let register_x = (instr >> 8) & 0xF;
        for i in 0..=register_x {
            self.v_registers[i as usize] = self.memory.read(self.i_register + i);
        }

        // Set I Register to I + X + 1
        self.i_register = self.i_register + register_x + 1;
    }
}