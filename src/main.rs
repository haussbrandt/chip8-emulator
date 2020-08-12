use std::fs;
use rand;

struct CPU {
    current_opcode: u16,
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,  // Index register
    pc: u16, // Program counter
    graphics: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16, // Stack pointer
    key: [u8; 16],
    draw_flag: bool,
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
        ];

        let mut memory = [0; 4096];

        for i in 0..80 {
            memory[i] = chip8_fontset[i];
        }

        CPU {
            current_opcode: 0,
            memory,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            graphics: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            draw_flag: false,
        }
    }

    fn load_game(&mut self, filename: &str) {
        let game = fs::read(filename).unwrap();
        for (i, v) in game.iter().enumerate() {
            self.memory[0x200 + i] = *v;
        }
    }

    fn emulate_cycle(&mut self) {
        let opcode =
            (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;

        let x = (opcode & 0x0F00) as usize;
        let y = (opcode & 0x00F0) as usize;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => {
                    self.graphics = [0; 64 * 32];
                    self.pc += 2;
                }, // Clear the screen
                0x000E => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }, // Return from subroutine
                x => println!("Unknown opcode: {}", x),
            },
            0x1000 => {
                self.pc = opcode & 0x0FFF;
            }, // Jump to address NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }, // Call subroutine at NNN
            0x3000 => {
                if self.v[x] == (opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }, // Skip if VX equal to NN
            0x4000 => {
                if self.v[x] != (opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }, // Skip if VX not equal to NN
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }, // Skip if VX equal to VY
            0x6000 => {
                self.v[x] = (opcode & 0x00FF) as u8;
                self.pc += 2;
            }, // Set VX to NN
            0x7000 => {
                if x != 0xF {
                    self.v[x] += (opcode & 0x00FF) as u8;
                }
                self.pc += 2;
            }, // Add NN to VX (carry flag not changed)
            0x8000 => match opcode & 0x000F {
                0x0 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                }, // Set VX to VY
                0x1 => {
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                }, // Set VX to VX or VY
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                }, // Set VX to VX and VY
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                }, // Set VX to VX xor VY
                0x4 => {
                    let result = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }
                    self.pc += 2;
                }, // Add VY to VX. Set VF to 1 if carry, 0 if not.
                0x5 => {
                    let result = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.pc += 2;
                }, // Subtract VY from VX. Set VF to 0 if borrow, 1 if not.
                0x6 => {
                    self.v[0xF] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                    self.pc += 2;
                }, // Store LSB of VX in VF. Shift VX to right by 1.
                0x7 => {
                    let result = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result.0;
                    if result.1 {
                        self.v[0xF] = 0;
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.pc += 2;
                }, // Set VX to VY - VX. Set VF to 0 if borrow, 1 if not.
                0xE => {
                    self.v[0xF] = self.v[x] & 0x80;
                    self.v[x] <<= 1;
                    self.pc += 2;
                }, // Store MSB of VX in VF. Shift VX to left by 1.
                x => println!("Unknown opcode: {}", x),
            },
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }, // Skip if VX not equal to VY
            0xA000 => {
                self.i = opcode & 0xFFF;
                self.pc += 2;
            }, // Set I to address NNN
            0xB000 => self.pc = (opcode & 0x0FFF) + self.v[0] as u16, // Jump to address NNN + V0
            0xC000 => {
                self.v[x] = rand::random::<u8>() & (opcode & 0x00FF) as u8;
                self.pc += 2;
            }, // Set VX to result of rand() & NN
            0xD000 => {}, // Draw
            0xE000 => {}, // Skip if key
            0xF000 =>  match opcode & 0x00FF {
                0x07 => {
                    self.v[x] = self.delay_timer;
                    self.pc += 2;
                }, // Set VX to delay timer
                0x0A => {

                }, // Wait for press key, store in VX
                0x15 => {
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                }, // Set delay timer to VX
                0x18 => {
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                }, // Set sound timer to VX
                0x1E => {
                    if x != 0xF {
                        self.i += self.v[x] as u16;
                    }
                    self.pc += 2;
                }, // Add VX to I if X is not F
                0x29 => {

                }, // Set I to the location of the sprite for the character in VX.
                0x33 => {

                }, // Store BCD representation of VX at the address in I
                0x55 => {

                }, // Store V0 to VX (inclusive) in memory starting at address I
                0x65 => {

                }, // Fill V0 to VX (inclusive) with values from memory starting at address I
                x => println!("Unknown opcode: {}", x),
            },
            x => println!("Unknown opcode: {}", x),
        }
    }
}

fn main() {
    let mut chip = CPU::new();

    setup_graphics();
    setup_input();

    chip.load_game("rockto.ch8");

    // for i in 45..66 {
    //     println!("{:#02x?}", &chip.memory[i * 32..i * 32 + 32]);
    // }

    // loop {
    chip.emulate_cycle();

    //     if chip.draw_flag {
    //         draw_graphics();
    //     }

    //     chip.set_keys();
    // }
}

fn setup_graphics() {}

fn setup_input() {}
