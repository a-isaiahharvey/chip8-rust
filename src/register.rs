use std::ops::{Index, IndexMut};

/// The Register
#[derive(Debug)]
pub struct Registers {
    /// 16 8-bit data registers
    pub v: [u8; 16],
    /// Address register
    pub i: u16,
    /// Stack pointer
    pub sp: u8,
}

impl Registers {
    /// Creates a new [`Registers`].
    pub fn new() -> Self {
        Self {
            v: [0x00; 16],
            i: 0x000,
            sp: 0,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Registers {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.v[index]
    }
}
