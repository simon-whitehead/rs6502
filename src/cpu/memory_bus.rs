use byteorder::{ByteOrder, LittleEndian};

use std::ops::{Deref, DerefMut};

/// Default, 64kb memory bus
pub struct MemoryBus {
    ram: [u8; 1024 * 64],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus { ram: [0; 1024 * 64] }
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        let addr = addr as usize;
        self.ram[addr] = byte;
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        self.ram[addr]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        LittleEndian::read_u16(&self.ram[addr..])
    }
}

// Used in tests to verify specific memory states
impl Deref for MemoryBus {
    type Target = [u8; 1024 * 64];

    fn deref(&self) -> &Self::Target {
        &self.ram
    }
}

impl DerefMut for MemoryBus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ram
    }
}