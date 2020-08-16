use ggez;
use ggez::event::{self, KeyCode};
use ggez::graphics;
use ggez::input;
use rand;
use std::{self, env, fs};

struct CPU {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,  // Index register
    pc: u16, // Program counter
    graphics: [u8; 64 * 128],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16, // Stack pointer
    key: [bool; 16],
    draw_flag: bool,
    rpl_user_flags: [u8; 8],
    is_extended: bool,
}

impl CPU {
    fn new() -> CPU {
        let chip8_fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
            0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, // 0
            0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // 1
            0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
            0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
            0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
            0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
            0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
            0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
            0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
            0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
        ];

        let mut memory = [0; 4096];

        for i in 0..180 {
            memory[i] = chip8_fontset[i];
        }

        CPU {
            memory,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            graphics: [0; 64 * 128],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [false; 16],
            draw_flag: false,
            rpl_user_flags: [0; 8],
            is_extended: false,
        }
    }

    fn set_keys(&mut self, ctx: &mut ggez::Context) {
        self.key = [false; 16];
        if input::keyboard::is_key_pressed(ctx, KeyCode::Key1) {
            self.key[0x1] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Key2) {
            self.key[0x2] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Key3) {
            self.key[0x3] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Key4) {
            self.key[0xC] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Q) {
            self.key[0x4] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.key[0x5] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::E) {
            self.key[0x6] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::R) {
            self.key[0xD] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            self.key[0x7] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
            self.key[0x8] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            self.key[0x9] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::F) {
            self.key[0xE] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Z) {
            self.key[0xA] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::X) {
            self.key[0x0] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::C) {
            self.key[0xB] = true;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::V) {
            self.key[0xF] = true;
        }
    }

    fn load_game(&mut self, filename: &str) {
        let game = fs::read(filename).unwrap();
        for (i, v) in game.iter().enumerate() {
            self.memory[0x200 + i] = *v;
        }
    }

    fn emulate_cycle(&mut self, ctx: &mut ggez::Context) {
        let opcode =
            (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        // println!("opcode: {:#04x}, pc: {:#04x}", opcode, self.pc);
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => {
                    self.graphics = [0; 64 * 128];
                    self.pc += 2;
                } // Clear the screen
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                } // Return from subroutine
                0x00FB => {
                    for y_line in 0..64 {
                        for x_line in (0..124).rev() {
                            self.graphics[x_line + y_line * 128] = self.graphics[x_line - 4 + y_line * 128];
                        }
                        for x_line in 0..4 {
                            self.graphics[x_line + y_line * 128] = 0;
                        }
                    }
                    self.pc += 2;
                } // Scroll right
                0x00FC => {
                    for y_line in 0..64 {
                        for x_line in 0..124 {
                            self.graphics[x_line + y_line * 128] = self.graphics[x_line + 4 + y_line * 128];
                        }
                        for x_line in 124..128 {
                            self.graphics[x_line + y_line * 128] = 0;
                        }
                    }
                    self.pc += 2;
                } // Scroll left
                0x00FD => {
                    ggez::event::quit(ctx);
                    self.pc += 2;
                } // Exit interpreter
                0x00FE => {
                    self.is_extended = false;
                    self.pc += 2;
                } // Disable extended mode
                0x00FF => {
                    self.is_extended = true;
                    self.pc += 2;
                } // Enable extended mode
                _ => match opcode & 0x00F0 {
                    0x00C0 => {
                        let n = opcode & 0x000F;
                        for x_line in 0..128 {
                            for y_line in (n..64).rev() {
                                self.graphics[(x_line + y_line * 128) as usize] = self.graphics[(x_line + (y_line - n) * 128) as usize];
                            }
                            for y_line in 0..n {
                                self.graphics[(x_line + y_line * 128) as usize] = 0;
                            }
                        }
                        self.pc += 2;
                    } // Scroll display N lines down
                    _ => println!("Unknown opcode: {:#04x}", opcode),
                }
            },
            0x1000 => {
                self.pc = opcode & 0x0FFF;
            } // Jump to address NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            } // Call subroutine at NNN
            0x3000 => {
                if self.v[x] == (opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            } // Skip if VX equal to NN
            0x4000 => {
                if self.v[x] != (opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            } // Skip if VX not equal to NN
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            } // Skip if VX equal to VY
            0x6000 => {
                self.v[x] = (opcode & 0x00FF) as u8;
                self.pc += 2;
            } // Set VX to NN
            0x7000 => {
                if x != 0xF {
                    let result = self.v[x].overflowing_add((opcode & 0x00FF) as u8);
                    self.v[x] = result.0;
                }
                self.pc += 2;
            } // Add NN to VX (carry flag not changed)
            0x8000 => match opcode & 0x000F {
                0x0 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                } // Set VX to VY
                0x1 => {
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                } // Set VX to VX or VY
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                } // Set VX to VX and VY
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                } // Set VX to VX xor VY
                0x4 => {
                    let result = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }
                    self.pc += 2;
                } // Add VY to VX. Set VF to 1 if carry, 0 if not.
                0x5 => {
                    let result = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.pc += 2;
                } // Subtract VY from VX. Set VF to 0 if borrow, 1 if not.
                0x6 => {
                    self.v[0xF] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                    self.pc += 2;
                } // Store LSB of VX in VF. Shift VX to right by 1.
                0x7 => {
                    let result = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.pc += 2;
                } // Set VX to VY - VX. Set VF to 0 if borrow, 1 if not.
                0xE => {
                    self.v[0xF] = self.v[x] & 0x80;
                    self.v[x] <<= 1;
                    self.pc += 2;
                } // Store MSB of VX in VF. Shift VX to left by 1.
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            } // Skip if VX not equal to VY
            0xA000 => {
                self.i = opcode & 0xFFF;
                self.pc += 2;
            } // Set I to address NNN
            0xB000 => self.pc = (opcode & 0x0FFF) + self.v[0] as u16, // Jump to address NNN + V0
            0xC000 => {
                self.v[x] = rand::random::<u8>() & (opcode & 0x00FF) as u8;
                self.pc += 2;
            } // Set VX to result of rand() & NN
            0xD000 => {
                let pos_x = self.v[x];
                let pos_y = self.v[y];
                let height = opcode & 0x000F;

                if (height == 0) & self.is_extended {
                    for y_line in 0..16 {
                        let pixels: u16 = ((self.memory[(self.i + y_line) as usize] as u16) << 8)
                            | (self.memory[(self.i + y_line) as usize] as u16);
                        for x_line in 0..16 {
                            if (pixels & (0x8000 >> x_line)) != 0 {
                                if ((pos_x as u16 + x_line as u16) < 128)
                                    & ((pos_y as u16 + y_line as u16) < 64)
                                {
                                    if self.graphics[((pos_x + x_line) as usize
                                        + ((pos_y as usize + y_line as usize) * 128))
                                        as usize]
                                        == 1
                                    {
                                        self.v[0xF] = 1;
                                    }
                                    self.graphics[((pos_x + x_line) as usize
                                        + ((pos_y as usize + y_line as usize) * 128))
                                        as usize] ^= 1
                                }
                            }
                        }
                    }
                } else {
                    for y_line in 0..height {
                        let pixels = self.memory[(self.i + y_line) as usize];
                        for x_line in 0..8 {
                            if (pixels & (0x80 >> x_line)) != 0 {
                                if ((pos_x as u16 + x_line as u16)
                                    < (64 * (1 + self.is_extended as u16)))
                                    & ((pos_y as u16 + y_line)
                                        < (128 * (1 + self.is_extended as u16)))
                                {
                                    if self.graphics[((pos_x + x_line) as usize
                                        + ((pos_y as usize + y_line as usize) * 128))
                                        as usize]
                                        == 1
                                    {
                                        self.v[0xF] = 1;
                                    }
                                    self.graphics[((pos_x + x_line) as usize
                                        + ((pos_y as usize + y_line as usize) * 128))
                                        as usize] ^= 1
                                }
                            }
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            } // Draw
            0xE000 => match opcode & 0x00FF {
                0x9E => {
                    if self.key[self.v[x] as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                } // Skip if key in VX is pressed
                0xA1 => {
                    if !self.key[self.v[x] as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                } // Skip if key in VX is not pressed
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => {
                    self.v[x] = self.delay_timer;
                    self.pc += 2;
                } // Set VX to delay timer
                0x0A => {
                    for key in self.key.iter().enumerate() {
                        if *key.1 {
                            self.v[x] = key.0 as u8;
                            self.pc += 2;
                            break;
                        }
                    }
                } // Wait for press key, store in VX
                0x15 => {
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                } // Set delay timer to VX
                0x18 => {
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                } // Set sound timer to VX
                0x1E => {
                    if x != 0xF {
                        self.i += self.v[x] as u16;
                    }
                    self.pc += 2;
                } // Add VX to I if X is not F
                0x29 => {
                    self.i = self.v[x] as u16 * 5;
                    self.pc += 2;
                } // Set I to the location of the sprite for the 5-byte character in VX.
                0x30 => {
                    self.i = self.v[x] as u16 * 10 + 80;
                    self.pc += 2;
                } // Set I to the location of the sprite for the 10-byte character in VX.
                0x33 => {
                    self.memory[self.i as usize] = self.v[x] / 100;
                    self.memory[self.i as usize + 1] = (self.v[x] / 10) % 10;
                    self.memory[self.i as usize + 2] = (self.v[x] % 100) % 10;
                    self.pc += 2;
                } // Store BCD representation of VX at the address in I
                0x55 => {
                    for j in 0..=x {
                        self.memory[self.i as usize + j] = self.v[j];
                    }
                    self.pc += 2;
                } // Store V0 to VX (inclusive) in memory starting at address I
                0x65 => {
                    for j in 0..=x {
                        self.v[j] = self.memory[self.i as usize + j];
                    }
                    self.pc += 2;
                } // Fill V0 to VX (inclusive) with values from memory starting at address I
                0x75 => {
                    for j in 0..=x {
                        self.rpl_user_flags[j] = self.v[j];
                    }
                    self.pc += 2;
                } // Store V0 to VX (inclusive) in RPL user flags
                0x85 => {
                    for j in 0..=x {
                        self.v[j] = self.rpl_user_flags[j];
                    }
                    self.pc += 2;
                } // Read V0 to VX (inclusive) from RPL user flags
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            _ => println!("Unknown opcode: {:#04x}", opcode),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        };
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("Make sound");
            }
            self.sound_timer -= 1;
        }
    }
}

impl event::EventHandler for CPU {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.emulate_cycle(ctx);
        self.set_keys(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        if self.draw_flag {
            graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
            let size = graphics::drawable_size(ctx);
            let pixel_width = size.0 / 128.0;
            let pixel_height = size.1 / 64.0;
            let mut mesh = graphics::MeshBuilder::new();

            for (idx, &pixel) in self.graphics.iter().enumerate() {
                if pixel != 0 {
                    let r = graphics::Rect::new(
                        (idx as f32 % 128.0) * pixel_width,
                        (idx / 128) as f32 * pixel_height,
                        pixel_width,
                        pixel_height,
                    );
                    mesh.rectangle(graphics::DrawMode::fill(), r, [0.9, 0.9, 0.9, 1.0].into());
                }
            }

            self.draw_flag = false;
            let mesh = mesh.build(ctx)?;
            graphics::draw(ctx, &mesh, graphics::DrawParam::new())?;
            graphics::present(ctx)?;
        }
        std::thread::sleep(std::time::Duration::from_micros(300));
        Ok(())
    }
}

fn main() -> ggez::GameResult {
    let wm = ggez::conf::WindowMode {
        width: 640.0,
        height: 320.0,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
        resizable: true,
    };

    let cb = ggez::ContextBuilder::new("chip8", "haussbrandt").window_mode(wm);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut CPU::new();
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    state.load_game(filename);
    event::run(ctx, event_loop, state)
}
