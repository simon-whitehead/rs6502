
pub enum MemoryBusErrorKind {
    WriteError,
    ReadError,
}

pub struct MemoryBusError {
    kind: MemoryBusErrorKind,
}

pub trait MemoryBus {
    fn read_byte(&self, addr: u16) -> Result<u8, MemoryBusError>;
    fn write_byte(&mut self, addr: u16, byte: u8) -> Result<(), MemoryBusError>;
}

/// Default, 64kb memory bus
pub struct DefaultMemoryBus {
    ram: [u8; 1024 * 64],
}

impl DefaultMemoryBus {
    pub fn new() -> DefaultMemoryBus {
        DefaultMemoryBus { ram: [0; 1024 * 64] }
    }
}

impl MemoryBus for DefaultMemoryBus {
    fn read_byte(&self, addr: u16) -> Result<u8, MemoryBusError> {
        Ok(self.ram[addr as usize])
    }

    fn write_byte(&mut self, addr: u16, byte: u8) -> Result<(), MemoryBusError> {
        self.ram[addr as usize] = byte;
        Ok(())
    }
}