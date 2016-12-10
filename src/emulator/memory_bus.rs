use std::ops::Deref;

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
}

// Used in tests to verify specific memory states
impl Deref for MemoryBus {
    type Target = [u8; 1024 * 64];

    fn deref(&self) -> &Self::Target {
        &self.ram
    }
}