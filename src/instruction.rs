use std::fmt;

pub type Addr = u16;
// type MemVal = u16;
pub type Reg = u8;
pub type RegVal = u8;
pub type ShortVal = u8;

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// Opcode: 00E0
    Clr,
    /// Opcode: 00EE
    Rts,

    /// Opcode: Dxyn
    Draw(ShortVal, Reg, Reg),

    /// Opcode: 0nnn
    Sys(u16),
    /// Opcode: 1nnn
    Jump(Addr),
    /// Opcode: 2nnn
    Call(Addr),
    /// Opcode: Annn
    LoadI(Addr),
    /// Opcode: Bnnn
    JumpI(Addr),

    /// Opcode: 3xnn
    Ske(Reg, RegVal),
    /// Opcode: 4xnn
    Skne(Reg, RegVal),
    /// Opcode: 6xnn
    Load(Reg, RegVal),
    /// Opcode: 7xnn
    Add(Reg, RegVal),
    /// Opcode: Cxnn
    Rand(Reg, RegVal),

    /// Opcode: 5xy0
    Skre(Reg, Reg),
    /// Opcode: 9xy0
    Skrne(Reg, Reg),
    /// Opcode: 8xy0
    Move(Reg, Reg),
    /// Opcode: 8xy1
    Or(Reg, Reg),
    /// Opcode: 8xy2
    And(Reg, Reg),
    /// Opcode: 8xy3
    Xor(Reg, Reg),
    /// Opcode: 8xy4
    Addr(Reg, Reg),
    /// Opcode: 8xy5
    Sub(Reg, Reg),
    /// Opcode: 8xy6
    Shr(Reg, Reg),
    /// Opcode: 8xyE
    Shl(Reg, Reg),

    /// Opcode: Ex9E
    Skpr(Reg),
    /// Opcode: ExA1
    Skup(Reg),
    /// Opcode: Fx07
    Moved(Reg),
    /// Opcode: Fx0A
    Keyd(Reg),
    /// Opcode: Fx15
    LoadD(Reg),
    /// Opcode: Fx18
    LoadS(Reg),
    /// Opcode: Fx1E
    AddI(Reg),
    /// Opcode: Fx29
    Ldspr(Reg),
    /// Opcode: Fx33
    Bcd(Reg),
    /// Opcode: Fx55
    Stor(Reg),
    /// Opcode: Fx65
    Read(Reg),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            Clr => write!(f, "CLR"),
            Rts => write!(f, "RTS"),

            Draw(x, y, n) => write!(f, "DRAW  v{:X}, v{:X}, {:#x}", x, y, n),

            Sys(addr) => write!(f, "SYS   {:#x}", addr),
            Jump(addr) => write!(f, "JUMP  {:#x}", addr),
            Call(addr) => write!(f, "CALL  {:#x}", addr),
            LoadI(addr) => write!(f, "LOADI {:#x}", addr),
            JumpI(addr) => write!(f, "JUMPI {:#x}", addr),

            Ske(x, n) => write!(f, "SKE   v{:X}, {:#x}", x, n),
            Skne(x, n) => write!(f, "SKNE  v{:X}, {:#x}", x, n),
            Load(x, n) => write!(f, "LOAD  v{:X}, {:#x}", x, n),
            Add(x, n) => write!(f, "ADD   v{:X}, {:#x}", x, n),
            Rand(x, n) => write!(f, "RAND  v{:X}, {:#x}", x, n),

            Skre(x, y) => write!(f, "SKRE  v{:X}, v{:X}", x, y),
            Skrne(x, y) => write!(f, "SKRNE v{:X}, v{:X}", x, y),
            Move(x, y) => write!(f, "MOVE  v{:X}, v{:X}", x, y),
            Or(x, y) => write!(f, "OR    v{:X}, v{:X}", x, y),
            And(x, y) => write!(f, "AND   v{:X}, v{:X}", x, y),
            Xor(x, y) => write!(f, "XOR   v{:X}, v{:X}", x, y),
            Addr(x, y) => write!(f, "ADDR  v{:X}, v{:X}", x, y),
            Sub(x, y) => write!(f, "SUB   v{:X}, v{:X}", x, y),
            Shr(x, y) => write!(f, "SHR   v{:X}, v{:X}", x, y),
            Shl(x, y) => write!(f, "SHL   v{:X}, v{:X}", x, y),

            Skpr(x) => write!(f, "SKPR  v{:X}", x),
            Skup(x) => write!(f, "SKUP  v{:X}", x),
            Moved(x) => write!(f, "MOVED v{:X}", x),
            Keyd(x) => write!(f, "KEYD  v{:X}", x),
            LoadD(x) => write!(f, "LOADD v{:X}", x),
            LoadS(x) => write!(f, "LOADS v{:X}", x),
            AddI(x) => write!(f, "ADDI  v{:X}", x),
            Ldspr(x) => write!(f, "LDSPR v{:X}", x),
            Bcd(x) => write!(f, "BCD   v{:X}", x),
            Stor(x) => write!(f, "STOR  v{:X}", x),
            Read(x) => write!(f, "READ  v{:X}", x),
        }
    }
}

fn addr(x: u16) -> Addr {
    x & 0x0FFF
}

fn imm(x: u16) -> RegVal {
    (x & 0x00FF) as RegVal
}

fn r1(x: u16) -> Reg {
    ((x & 0x0F00) >> 8) as Reg
}

fn r2(x: u16) -> Reg {
    ((x & 0x00F0) >> 4) as Reg
}

impl TryFrom<u16> for Instruction {
    type Error = String;

    fn try_from(x: u16) -> Result<Self, Self::Error> {
        use Instruction::*;
        match x & 0xF000 {
            0x0000 => match x {
                0x00E0 => Ok(Clr),
                0x00EE => Ok(Rts),
                _ => Ok(Sys(addr(x))),
            },
            0x1000 => Ok(Jump(addr(x))),
            0x2000 => Ok(Call(addr(x))),
            0x3000 => Ok(Ske(r1(x), imm(x))),
            0x4000 => Ok(Skne(r1(x), imm(x))),
            0x5000 => match x & 0x000F {
                0x0 => Ok(Skre(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0x6000 => Ok(Load(r1(x), imm(x))),
            0x7000 => Ok(Add(r1(x), imm(x))),
            0x8000 => match x & 0x000F {
                0x0 => Ok(Move(r1(x), r2(x))),
                0x1 => Ok(Or(r1(x), r2(x))),
                0x2 => Ok(And(r1(x), r2(x))),
                0x3 => Ok(Xor(r1(x), r2(x))),
                0x4 => Ok(Addr(r1(x), r2(x))),
                0x5 => Ok(Sub(r1(x), r2(x))),
                0x6 => Ok(Shr(r1(x), r2(x))),
                0xE => Ok(Shl(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0x9000 => match x & 0x000F {
                0x0 => Ok(Skrne(r1(x), r2(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0xA000 => Ok(LoadI(addr(x))),
            0xB000 => Ok(JumpI(addr(x))),
            0xC000 => Ok(Rand(r1(x), imm(x))),
            0xD000 => Ok(Draw(r1(x), r2(x), (x & 0x000F) as ShortVal)),
            0xE000 => match x & 0x00FF {
                0x9E => Ok(Skpr(r1(x))),
                0xA1 => Ok(Skup(r1(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            0xF000 => match x & 0x00FF {
                0x07 => Ok(Moved(r1(x))),
                0x0A => Ok(Keyd(r1(x))),
                0x15 => Ok(LoadD(r1(x))),
                0x18 => Ok(LoadS(r1(x))),
                0x1E => Ok(AddI(r1(x))),
                0x29 => Ok(Ldspr(r1(x))),
                0x33 => Ok(Bcd(r1(x))),
                0x55 => Ok(Stor(r1(x))),
                0x65 => Ok(Read(r1(x))),
                _ => Err(format!("Invalid Instruction: {:#x}", x)),
            },
            _ => Err(format!("Invalid Instruction: {:#x}", x)),
        }
    }
}

impl From<Instruction> for u16 {
    fn from(instr: Instruction) -> Self {
        use Instruction::*;
        match instr {
            Clr => 0x00E0,
            Rts => 0x00EE,

            Draw(x, y, n) => {
                0xD000
                    | (((x as u16) << 8) & 0x0F00)
                    | (((y as u16) << 4) & 0x00F0)
                    | ((n as u16) & 0x000F)
            }

            Sys(addr) => (addr & 0x0FFF),
            Jump(addr) => 0x1000 | (addr & 0x0FFF),
            Call(addr) => 0x2000 | (addr & 0x0FFF),
            LoadI(addr) => 0xA000 | (addr & 0x0FFF),
            JumpI(addr) => 0xB000 | (addr & 0x0FFF),

            Ske(r, v) => 0x3000 | 0x0F00 & ((r as u16) << 8) | (0x00FF & v as u16),
            Skne(r, v) => 0x4000 | 0x0F00 & ((r as u16) << 8) | (0x00FF & v as u16),
            Load(r, v) => 0x6000 | 0x0F00 & ((r as u16) << 8) | (0x00FF & v as u16),
            Add(r, v) => 0x7000 | 0x0F00 & ((r as u16) << 8) | (0x00FF & v as u16),
            Rand(r, v) => 0xC000 | 0x0F00 & ((r as u16) << 8) | (0x00FF & v as u16),

            Skre(r1, r2) => 0x5000 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Skrne(r1, r2) => 0x9000 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Move(r1, r2) => 0x8000 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Or(r1, r2) => 0x8001 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            And(r1, r2) => 0x8002 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Xor(r1, r2) => 0x8003 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Addr(r1, r2) => 0x8004 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Sub(r1, r2) => 0x8005 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Shr(r1, r2) => 0x8006 | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),
            Shl(r1, r2) => 0x800E | 0x0F00 & ((r1 as u16) << 8) | (0x00F0 & r2 as u16),

            Skpr(r) => 0xE09E | 0x0F00 & ((r as u16) << 8),
            Skup(r) => 0xE0A1 | 0x0F00 & ((r as u16) << 8),
            Moved(r) => 0xF007 | 0x0F00 & ((r as u16) << 8),
            Keyd(r) => 0xF00A | 0x0F00 & ((r as u16) << 8),
            LoadD(r) => 0xF015 | 0x0F00 & ((r as u16) << 8),
            LoadS(r) => 0xF018 | 0x0F00 & ((r as u16) << 8),
            AddI(r) => 0xF01E | 0x0F00 & ((r as u16) << 8),
            Ldspr(r) => 0xF029 | 0x0F00 & ((r as u16) << 8),
            Bcd(r) => 0xF033 | 0x0F00 & ((r as u16) << 8),
            Stor(r) => 0xF055 | 0x0F00 & ((r as u16) << 8),
            Read(r) => 0xF065 | 0x0F00 & ((r as u16) << 8),
        }
    }
}
