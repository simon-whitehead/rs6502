use std::cell::RefCell;

use emulator::memory_bus::MemoryBus;
use emulator::registers::Registers;

pub struct Cpu<T>
    where T: MemoryBus
{
    memory: RefCell<T>,
    registers: Registers,
}

impl<T> Cpu<T>
    where T: MemoryBus
{
    pub fn new(memory_bus: RefCell<T>) -> Cpu<T> {
        Cpu {
            memory: memory_bus,
            registers: Registers::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use emulator::DefaultMemoryBus;

    #[test]
    fn can_instantiate_cpu() {
        let memory = RefCell::new(DefaultMemoryBus::new());
        let cpu = Cpu::new(memory);

        assert!(cpu.memory.borrow().iter().all(|x| *x == 0));
    }
}
