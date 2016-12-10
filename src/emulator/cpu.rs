use ::opcodes::OpCode;

use emulator::memory_bus::MemoryBus;
use emulator::registers::Registers;

const DEFAULT_CODE_SEGMENT_START_ADDRESS: u16 = 0xC000;  // Default to a 16KB ROM, leaving 32KB of main memory

pub struct Cpu {
    memory: MemoryBus,
    pub registers: Registers,
}

#[derive(Debug, PartialEq)]
pub enum CpuErrorKind {
    SegFault,
}

#[derive(Debug, PartialEq)]
pub struct CpuError {
    message: String,
    addr: u16,
    kind: CpuErrorKind,
}

impl CpuError {
    fn code_segment_out_of_range(addr: u16) -> CpuError {
        CpuError {
            message: format!("Attempted to layout code segment outside memory bounds"),
            addr: addr,
            kind: CpuErrorKind::SegFault,
        }
    }
}

pub type CpuLoadResult = Result<(), CpuError>;
pub type CpuStepResult = Result<(), CpuError>;

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            memory: MemoryBus::new(),
            registers: Registers::new(),
        }
    }

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

    pub fn step<T>(&mut self) -> CpuStepResult {
        let byte = self.memory.read_byte(self.registers.PC);

        let opcode = OpCode::from_raw_byte(byte);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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