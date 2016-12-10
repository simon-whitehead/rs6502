use ::opcodes::OpCode;

use cpu::cpu_error::CpuError;
use cpu::memory_bus::MemoryBus;
use cpu::registers::Registers;

const DEFAULT_CODE_SEGMENT_START_ADDRESS: u16 = 0xC000;  // Default to a 16KB ROM, leaving 32KB of main memory

/// A representation of a 6502 microprocessor
pub struct Cpu {
    memory: MemoryBus,
    pub registers: Registers,
}

pub type CpuLoadResult = Result<(), CpuError>;
pub type CpuStepResult = Result<(), CpuError>;

impl Cpu {
    /// Returns a default instance of a Cpu
    pub fn new() -> Cpu {
        Cpu {
            memory: MemoryBus::new(),
            registers: Registers::new(),
        }
    }

    /// Loads code into the Cpu main memory at an optional offset. If no
    /// offset is provided, the Cpu will, by default, load the code into
    /// main memory at 0xC000
    pub fn load<T>(&mut self, code: &[u8], addr: T) -> CpuLoadResult
        where T: Into<Option<u16>>
    {
        let addr = addr.into();
        let addr: u16 = if addr.is_some() {
            let addr = addr.unwrap();
            if addr as u32 + code.len() as u32 > u16::max_value() as u32 {
                return Err(CpuError::code_segment_out_of_range(addr));
            } else {
                addr
            }
        } else {
            DEFAULT_CODE_SEGMENT_START_ADDRESS
        };

        for x in 0..code.len() {
            self.memory.write_byte(addr + x as u16, code[x]);
        }

        Ok(())
    }

    /// Runs a single instruction of code through the Cpu
    pub fn step<T>(&mut self) -> CpuStepResult {
        let byte = self.memory.read_byte(self.registers.PC);

        if let Some(opcode) = OpCode::from_raw_byte(byte) {
            Ok(())
        } else {
            Err(CpuError::unknown_opcode(self.registers.PC, byte))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpu::cpu_error::CpuError;

    #[test]
    fn can_instantiate_cpu() {
        let cpu = Cpu::new();

        assert!(0 == 0);
    }

    #[test]
    fn can_load_code_segment_into_memory() {
        let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cpu = Cpu::new();
        cpu.load(&fake_code[..], None);

        let memory_sum: u32 = cpu.memory.iter().map(|n| *n as u32).sum();
        assert_eq!(46, memory_sum);
    }

    #[test]
    fn can_load_code_segment_at_default_address() {
        let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cpu = Cpu::new();
        cpu.load(&fake_code[..], None);

        assert_eq!(0x0D, cpu.memory.read_byte(0xC003));
        assert_eq!(0x0C, cpu.memory.read_byte(0xC002));
        assert_eq!(0x0B, cpu.memory.read_byte(0xC001));
        assert_eq!(0x0A, cpu.memory.read_byte(0xC000));
    }

    #[test]
    fn can_load_code_segment_at_specific_address() {
        let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cpu = Cpu::new();
        cpu.load(&fake_code[..], 0xF000);

        assert_eq!(0x0D, cpu.memory.read_byte(0xF003));
        assert_eq!(0x0C, cpu.memory.read_byte(0xF002));
        assert_eq!(0x0B, cpu.memory.read_byte(0xF001));
        assert_eq!(0x0A, cpu.memory.read_byte(0xF000));
    }

    #[test]
    fn errors_when_code_segment_extends_past_memory_bounds() {
        let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cpu = Cpu::new();
        let load_result = cpu.load(&fake_code[..], 0xFFFD);

        assert_eq!(Err(CpuError::code_segment_out_of_range(0xFFFD)),
                   load_result);
    }
}
