use std::{
    fmt::{self, Display},
    sync::{Arc, Mutex},
    time,
};

use log::trace;
use phf::phf_ordered_map;
use rand::prelude::*;

use crate::{
    app::{FONT, SCREEN_HEIGHT, SCREEN_WIDTH},
    instruction::Instruction,
    register::Registers,
};
use Instruction::*;

#[derive(Debug)]
pub struct Chip8IO {
    pub keystate: [bool; 16],
    pub display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

pub const KEYPAD_TO_QWERTY: phf::OrderedMap<u8, char> = phf_ordered_map! {
  0x1u8 => '1',
  0x2u8 => '2',
  0x3u8 => '3',
  0xCu8 => '4',

  0x4u8 => 'Q',
  0x5u8 => 'W',
  0x6u8 => 'E',
  0xDu8 => 'R',

  0x7u8 => 'A',
  0x8u8 => 'S',
  0x9u8 => 'D',
  0xEu8 => 'F',

  0xAu8 => 'Z',
  0x0u8 => 'X',
  0xBu8 => 'C',
  0xFu8 => 'V',
};

impl Chip8IO {
    pub fn new() -> Chip8IO {
        Chip8IO {
            keystate: [false; 16],
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for Chip8IO {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Chip8 {
    pub stack: Vec<u16>,
    pub pc: u16,
    pub reg: Registers,
    pub delay: u8,
    tick: time::Instant,
    pub memory: [u8; 4096],
    pub io: Arc<Mutex<Chip8IO>>,

    pub paused: bool,
}

/// Outcome of one step of execution
#[derive(PartialEq, Eq)]
pub enum StepResult {
    /// Program continues. Bool specifies whether the display was updated
    Continue(bool),

    /// Endlessly looping
    Loop,

    /// Program ends.
    End,
}

fn wkey(f: &mut fmt::Formatter<'_>, keystate: [bool; 16], key: u8) -> fmt::Result {
    if keystate[key as usize] {
        write!(f, "{:X}", key)
    } else {
        write!(f, "█")
    }
}

impl Display for Chip8IO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wkey(f, self.keystate, 0x1)?;
        wkey(f, self.keystate, 0x2)?;
        wkey(f, self.keystate, 0x3)?;
        wkey(f, self.keystate, 0xC)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0x4)?;
        wkey(f, self.keystate, 0x5)?;
        wkey(f, self.keystate, 0x6)?;
        wkey(f, self.keystate, 0xD)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0x7)?;
        wkey(f, self.keystate, 0x8)?;
        wkey(f, self.keystate, 0x9)?;
        wkey(f, self.keystate, 0xE)?;
        writeln!(f)?;
        wkey(f, self.keystate, 0xA)?;
        wkey(f, self.keystate, 0x0)?;
        wkey(f, self.keystate, 0xB)?;
        wkey(f, self.keystate, 0xF)?;
        writeln!(f)?;

        writeln!(
            f,
            "\n┌────────────────────────────────────────────────────────────────┐"
        )?;
        for row in self.display {
            write!(f, "│")?;
            for pixel in row {
                if pixel {
                    write!(f, "█")?;
                } else {
                    write!(f, "·")?;
                }
            }
            writeln!(f, "│")?;
        }
        writeln!(
            f,
            "└────────────────────────────────────────────────────────────────┘"
        )?;
        Ok(())
    }
}

impl Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instr = match self.current_instruction() {
            Ok(i) => format!("{}", i),
            Err(e) => e,
        };

        write!(
            f,
            "CHIP8 | pc: {:#X} | {:<20} | idx: {:>3X} | reg: {:?} | stack: {}",
            self.pc,
            instr,
            self.reg.i,
            self.reg,
            self.stack.len()
        )?;
        Ok(())
    }
}

impl Chip8 {
    pub fn new(instruction_section: &[u8], io: Arc<Mutex<Chip8IO>>, paused: bool) -> Chip8 {
        let mut memory = [0; 4096];
        // Load the font
        memory[..FONT.len()].copy_from_slice(&FONT[..]);

        memory[0x200..0x200 + instruction_section.len()].copy_from_slice(instruction_section);

        // Return the Chip8
        Chip8 {
            reg: Registers::new(),
            pc: 0x200,
            stack: Vec::new(),
            delay: 0,
            tick: time::Instant::now(),
            memory,
            io,
            paused,
        }
    }

    fn advance(&mut self, amount: u16) -> Result<StepResult, String> {
        self.pc += amount;
        Ok(StepResult::Continue(false))
    }

    pub fn reset(&mut self) {
        self.reg = Registers::default();
        self.reg.i = 0;
        self.pc = 0x200;
        self.stack = Vec::new();
        self.delay = 0;
        self.tick = time::Instant::now();
        self.memory = {
            let mut memory = [0; 4096];
            // Load the font
            memory[..FONT.len()].copy_from_slice(&FONT[..]);
            memory
        };
        self.io.lock().unwrap().reset();
    }

    /// Load ROM for `Chip8` from file path
    pub fn load_rom(&mut self, rom: &[u8]) {
        let filesize = rom.len();

        trace!("ROM file size is {} bytes", filesize);

        let start = 0x200;
        let end = start + filesize;

        self.memory[start..end].copy_from_slice(rom);
    }

    pub fn current_instruction(&self) -> Result<Instruction, String> {
        Instruction::try_from(u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[self.pc as usize + 1],
        ]))
    }

    pub fn step(&mut self) -> Result<StepResult, String> {
        if self.paused {
            return Ok(StepResult::Continue(false));
        }

        if time::Instant::now() - self.tick > time::Duration::from_millis(16) {
            self.delay = self.delay.saturating_sub(1);
            self.tick = time::Instant::now();
        }

        match self.current_instruction()? {
            Move(x, y) => {
                self.reg[x as usize] = self.reg[y as usize];
                self.advance(2)
            }
            Or(x, y) => {
                self.reg[x as usize] |= self.reg[y as usize];
                self.advance(2)
            }
            And(x, y) => {
                self.reg[x as usize] &= self.reg[y as usize];
                self.advance(2)
            }
            Xor(x, y) => {
                self.reg[x as usize] ^= self.reg[y as usize];
                self.advance(2)
            }
            Addr(x, y) => {
                match self.reg[x as usize].checked_add(self.reg[y as usize]) {
                    Some(val) => {
                        self.reg[x as usize] = val;
                        self.reg[0xf] = 0;
                    }
                    None => {
                        self.reg[x as usize] =
                            self.reg[x as usize].wrapping_add(self.reg[y as usize]);
                        self.reg[0xf] = 1;
                    }
                }
                self.advance(2)
            }
            Sub(x, y) => {
                self.reg[x as usize] = self.reg[x as usize].wrapping_sub(self.reg[y as usize]);
                self.advance(2)
            }
            Shr(x, y) => {
                self.reg[0x0F] = self.reg[y as usize] & 1;
                self.reg[y as usize] = self.reg[x as usize] >> 1;
                self.advance(2)
            }
            Shl(x, y) => {
                self.reg[0x0F] = self.reg[y as usize] & 0xE0;
                self.reg[y as usize] = self.reg[x as usize] << 1;
                self.advance(2)
            }
            Load(x, n) => {
                self.reg[x as usize] = n;
                self.advance(2)
            }
            Add(x, n) => {
                self.reg[x as usize] = self.reg[x as usize].wrapping_add(n);
                self.advance(2)
            }
            // Subroutines
            Call(addr) => {
                if addr == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.stack.push(self.pc);
                    self.pc = addr;
                    Ok(StepResult::Continue(false))
                }
            }
            Rts => {
                if let Some(pc) = self.stack.pop() {
                    self.pc = pc;
                    self.advance(2)
                } else {
                    Err("Return from empty stack".to_string())
                }
            }
            // Jumps
            Jump(ofs) => {
                let next_pc = (self.pc & 0xF000) | (ofs & 0x0FFF);
                if next_pc == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.pc = next_pc;
                    Ok(StepResult::Continue(false))
                }
            }
            JumpI(addr) => {
                let next_pc = addr + self.reg[0] as u16;
                if next_pc == self.pc {
                    Ok(StepResult::Loop)
                } else {
                    self.pc = next_pc;
                    Ok(StepResult::Continue(false))
                }
            }
            // Skip
            Ske(x, n) => {
                if self.reg[x as usize] == n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            Skne(x, n) => {
                if self.reg[x as usize] != n {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            Skre(x, y) => {
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            Skrne(x, y) => {
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            // Memory
            Stor(x) => {
                for r in 0..=x {
                    self.memory[self.reg.i as usize] = self.reg[r as usize];
                    self.reg.i += 1;
                }

                self.advance(2)
            }
            Read(x) => {
                for r in 0..=x {
                    self.reg[r as usize] = self.memory[self.reg.i as usize];
                    self.reg.i += 1;
                }

                self.advance(2)
            }
            // Input
            Skpr(x) => {
                let keyidx: usize = self.reg[x as usize] as usize;
                let pressed = *self
                    .io
                    .lock()
                    .unwrap()
                    .keystate
                    .get(keyidx)
                    .unwrap_or(&false);
                if pressed {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            Skup(x) => {
                let keyidx: usize = self.reg[x as usize] as usize;
                let pressed = *self
                    .io
                    .lock()
                    .unwrap()
                    .keystate
                    .get(keyidx)
                    .unwrap_or(&false);
                if !pressed {
                    self.advance(4)
                } else {
                    self.advance(2)
                }
            }
            Keyd(x) => {
                let keystate = self.io.lock().unwrap().keystate;
                for (key, &pressed) in keystate.iter().enumerate() {
                    if pressed {
                        self.reg[x as usize] = key as u8;
                        let _ = self.advance(2);
                        break;
                    }
                }
                Ok(StepResult::Continue(false))
            }

            // Sound
            // TODO: Implement sound
            LoadS(_) => self.advance(2),

            // Delays
            Moved(x) => {
                self.reg[x as usize] = self.delay;
                self.advance(2)
            }
            LoadD(x) => {
                self.delay = self.reg[x as usize];
                self.advance(2)
            }

            // Index register
            AddI(x) => {
                self.reg.i += self.reg[x as usize] as u16;
                self.advance(2)
            }
            LoadI(addr) => {
                self.reg.i = addr;
                self.advance(2)
            }
            // Screen
            Draw(x, y, n) => {
                let mut row = self.reg[y as usize] as usize;
                let memidx = self.reg.i as usize;

                {
                    // Lock IO here
                    let display = &mut self.io.lock().unwrap().display;
                    self.reg[0x0F] = 0;
                    for byte in &self.memory[memidx..memidx + n as usize] {
                        let mut col = self.reg[x as usize] as usize;
                        for bitidx in 0..8 {
                            let bit = (byte & (1 << (7 - bitidx))) != 0;
                            if display[row % SCREEN_HEIGHT][col % SCREEN_WIDTH] & bit {
                                self.reg[0x0F] = 1;
                            }

                            display[row % SCREEN_HEIGHT][col % SCREEN_WIDTH] ^= bit;
                            col += 1;
                        }

                        row += 1;
                    }
                }

                let _ = self.advance(2);
                Ok(StepResult::Continue(true))
            }
            Clr => {
                self.io.lock().unwrap().display = [[false; 64]; 32];
                self.advance(2)
            }
            // Other
            Ldspr(x) => {
                let val = self.reg[x as usize];
                if val > 15 {
                    Err(format!("LDSPR for {} > 15", val))
                } else {
                    self.reg.i = val as u16 * 5;
                    self.advance(2)
                }
            }
            Bcd(x) => {
                let hundreds = self.reg[x as usize] / 100;
                let tens = (self.reg[x as usize] % 100) / 10;
                let ones = self.reg[x as usize] % 10;

                self.memory[self.reg.i as usize] = hundreds;
                self.memory[self.reg.i as usize + 1] = tens;
                self.memory[self.reg.i as usize + 2] = ones;

                self.advance(2)
            }
            Rand(x, n) => {
                let mut rng = rand::thread_rng();
                self.reg[x as usize] = rng.gen_range(0..n);
                self.advance(2)
            }
            Sys(0) => Ok(StepResult::End),
            Sys(_) => Err("SYS".to_string()),
        }
    }
}
