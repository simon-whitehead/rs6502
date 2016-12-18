use ::opcodes::{AddressingMode, OpCode};

use cpu::cpu_error::CpuError;
use cpu::flags::StatusFlags;
use cpu::memory_bus::MemoryBus;
use cpu::registers::Registers;
use cpu::stack::Stack;

const DEFAULT_CODE_SEGMENT_START_ADDRESS: u16 = 0xC000;  // Default to a 16KB ROM, leaving 32KB of main memory

const STACK_START: usize = 0x100;
const STACK_END: usize = 0x1FF;

#[derive(Debug)]
pub enum Operand {
    Immediate(u8),
    Memory(u16),
    Implied,
}

/// A representation of a 6502 microprocessor
pub struct Cpu {
    pub memory: MemoryBus,
    pub registers: Registers,
    pub flags: StatusFlags,
    pub stack: Stack,
    code_start: usize,
    code_size: usize,
}

pub type CpuLoadResult = Result<(), CpuError>;
pub type CpuStepResult = Result<u8, CpuError>;
pub type CpuMultiStepResult = Result<u64, CpuError>;

impl Cpu {
    /// Returns a default instance of a Cpu
    pub fn new() -> Cpu {
        Cpu {
            memory: MemoryBus::new(),
            registers: Registers::new(),
            flags: Default::default(),
            stack: Stack::new(),
            code_start: DEFAULT_CODE_SEGMENT_START_ADDRESS as usize,
            code_size: 0,
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

        // Set the Program Counter to point at the
        // start address of the code segment
        self.registers.PC = addr;

        self.code_start = addr as usize;
        self.code_size = code.len();

        Ok(())
    }

    pub fn get_code(&self) -> &[u8] {
        &self.memory[self.code_start..self.code_start + self.code_size]
    }

    /// Runs N instructions of code through the Cpu
    pub fn step_n(&mut self, n: u32) -> CpuMultiStepResult {
        let mut v = 0;
        for _ in 0..n {
            if self.finished() {
                break;
            }
            v += self.step()? as u64;
        }

        Ok(v)
    }

    pub fn finished(&self) -> bool {
        self.registers.PC > self.code_start as u16 + self.code_size as u16 - 1
    }

    pub fn reset(&mut self) {
        self.registers.PC = self.code_start as u16;
    }

    /// Runs a single instruction of code through the Cpu
    pub fn step(&mut self) -> CpuStepResult {
        let byte = self.memory.read_byte(self.registers.PC);

        if let Some(opcode) = OpCode::from_raw_byte(byte) {
            let operand = self.get_operand_from_opcode(&opcode);

            self.registers.PC += opcode.length as u16;

            match opcode.mnemonic {
                "ADC" => self.adc(&operand),
                "AND" => self.and(&operand),
                "ASL" => self.asl(&operand),
                "BCC" => self.bcc(&operand),
                "BCS" => self.bcs(&operand),
                "BEQ" => self.beq(&operand),
                "BIT" => self.bit(&operand),
                "BMI" => self.bmi(&operand),
                "BNE" => self.bne(&operand),
                "BPL" => self.bpl(&operand),
                "BRK" => self.brk(),
                "BVC" => self.bvc(&operand),
                "BVS" => self.bvs(&operand),
                "CLC" => self.set_carry_flag(false),
                "CLD" => self.set_decimal_flag(false),
                "CLI" => self.set_interrupt_flag(false),
                "CLV" => self.set_overflow_flag(false),
                "CMP" => {
                    let a = self.registers.A;
                    self.compare(&operand, a)
                }
                "CPX" => {
                    let x = self.registers.X;
                    self.compare(&operand, x)
                }
                "CPY" => {
                    let y = self.registers.Y;
                    self.compare(&operand, y)
                }
                "DEC" => self.dec(&operand),
                "DEX" => self.dex(),
                "DEY" => self.dey(),
                "EOR" => self.eor(&operand),
                "INC" => self.inc(&operand),
                "INX" => self.inx(),
                "INY" => self.iny(),
                "JMP" => self.jmp(&operand),
                "JSR" => self.jsr(&operand),
                "LDA" => self.lda(&operand),
                "LDX" => self.ldx(&operand),
                "LDY" => self.ldy(&operand),
                "LSR" => self.lsr(&operand),
                "NOP" => self.nop(),
                "ORA" => self.ora(&operand),
                "PHA" => self.pha(),
                "PHP" => self.php(),
                "PLA" => self.pla(),
                "PLP" => self.plp(),
                "ROL" => self.rol(&operand),
                "ROR" => self.ror(&operand),
                "RTI" => self.rti(),
                "RTS" => self.rts(),
                "SBC" => self.sbc(&operand),
                "SEC" => self.set_carry_flag(true),
                "SED" => self.set_decimal_flag(true),
                "SEI" => self.set_interrupt_flag(true),
                "STA" => self.sta(&operand),
                "STX" => self.stx(&operand),
                "STY" => self.sty(&operand),
                "TAX" => self.tax(),
                "TAY" => self.tay(),
                "TSX" => self.tsx(),
                "TXA" => self.txa(),
                "TXS" => self.txs(),
                "TYA" => self.tya(),
                _ => return Err(CpuError::unknown_opcode(self.registers.PC, opcode.code)),
            }

            Ok(opcode.time)
        } else {
            Err(CpuError::unknown_opcode(self.registers.PC, byte))
        }
    }

    fn get_operand_from_opcode(&self, opcode: &OpCode) -> Operand {
        use ::opcodes::AddressingMode::*;

        let operand_start = self.registers.PC + 1;

        match opcode.mode {
            Unknown => unreachable!(),
            Implied => Operand::Implied,
            Immediate => Operand::Immediate(self.read_byte(operand_start)),
            Relative => Operand::Immediate(self.read_byte(operand_start)),
            Accumulator => Operand::Implied,
            ZeroPage => Operand::Memory((self.read_byte(operand_start) as u16) & 0xFF),
            ZeroPageX => {
                Operand::Memory((self.registers.X as u16 + self.read_byte(operand_start) as u16) &
                                0xFF)
            }
            ZeroPageY => {
                Operand::Memory((self.registers.Y as u16 + self.read_byte(operand_start) as u16) &
                                0xFF)
            }
            Absolute => Operand::Memory(self.read_u16(operand_start)),
            AbsoluteX => Operand::Memory(self.registers.X as u16 + self.read_u16(operand_start)),
            AbsoluteY => Operand::Memory(self.registers.Y as u16 + self.read_u16(operand_start)),
            Indirect => Operand::Memory(self.read_u16(self.read_u16(operand_start))),
            IndirectX => {
                Operand::Memory(self.read_u16((self.registers.X as u16 +
                                               self.read_byte(self.registers.PC + 1) as u16) &
                                              0xFF))
            }
            IndirectY => {
                Operand::Memory(self.registers.Y as u16 +
                                self.read_u16(self.read_byte(self.registers.PC + 1) as u16))
            }
        }
    }

    fn unwrap_immediate(&self, operand: &Operand) -> u8 {
        match *operand {
            Operand::Immediate(byte) => byte,
            Operand::Memory(addr) => self.read_byte(addr),
            Operand::Implied => 0,
        }
    }

    fn unwrap_address(&self, operand: &Operand) -> u16 {
        match *operand {
            Operand::Immediate(byte) => byte as u16,
            Operand::Memory(addr) => addr,
            Operand::Implied => 0,
        }
    }

    // ## OpCode handlers ##

    fn adc(&mut self, operand: &Operand) {
        // This is implemented on the information provided here:
        // http://www.electrical4u.com/bcd-or-binary-coded-decimal-bcd-conversion-addition-subtraction/
        // and here:
        // http://www.6502.org/tutorials/decimal_mode.html,
        // and here:
        // http://www.atariarchives.org/2bml/chapter_10.php,
        // and also here:
        // http://stackoverflow.com/questions/29193303/6502-emulation-proper-way-to-implement-adc-and-sbc

        let carry = if self.flags.carry { 1 } else { 0 };

        let value = self.unwrap_immediate(&operand) as u16;
        let value_signs = self.registers.A & 0x80 == 0x80 && value & 0x80 == 0x80;

        // Do normal binary arithmetic first
        let mut result = self.registers.A as u16 + value as u16 + carry as u16;

        // Handle packed binary coded decimal
        if self.flags.decimal {
            if (self.registers.A as u16 & 0x0F) + (value & 0x0F) + carry > 0x09 {
                result += 0x06;
            }

            if result > 0x99 {
                result += 0x60;
            }
        }

        self.flags.carry = (result & 0x100) == 0x100;
        self.flags.zero = result as u8 & 0xFF == 0x00;
        self.flags.sign = result & 0x80 == 0x80;

        if self.flags.sign != value_signs {
            self.flags.overflow = true;
        }

        self.registers.A = result as u8 & 0xFF;
    }

    fn and(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);
        let result = self.registers.A & value;

        self.registers.A = result;

        self.flags.zero = result as u8 & 0xFF == 0;
        self.flags.sign = result & 0x80 == 0x80;
    }

    fn asl(&mut self, operand: &Operand) {
        let mut value = if let &Operand::Implied = operand {
            // Implied ASL uses the A register
            self.registers.A
        } else {
            self.unwrap_immediate(&operand)
        };

        // Test the seventh bit - if its set, shift it
        // into the carry flag
        self.flags.carry = (value & 0x80) == 0x80;

        // Shift the value left
        value = value << 0x01;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value as u8 & 0xFF == 0;

        if let &Operand::Implied = operand {
            self.registers.A = value;
        } else {
            let addr = self.unwrap_address(&operand);
            self.write_byte(addr, value);
        }
    }

    fn bcc(&mut self, operand: &Operand) {
        // Branch if the carry flag is not set
        if !self.flags.carry {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn bcs(&mut self, operand: &Operand) {
        // Branch if the carry flag is set
        if self.flags.carry {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn beq(&mut self, operand: &Operand) {
        // Branch if the zero flag is set
        if self.flags.zero {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn bit(&mut self, operand: &Operand) {
        let a = self.registers.A;
        let value = self.unwrap_immediate(&operand);
        let result = value & a;

        self.flags.zero = result == 0x00;
        self.flags.overflow = value & 0x40 == 0x40; // "The V flag and the N flag receive copies of the sixth and seventh bits of the tested number"
        self.flags.sign = value & 0x80 == 0x80;
    }

    fn bmi(&mut self, operand: &Operand) {
        // Branch if the sign flag is set
        if self.flags.sign {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn bne(&mut self, operand: &Operand) {
        // Branch if the zero flag is not set
        if !self.flags.zero {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn bpl(&mut self, operand: &Operand) {
        // Branch if the sign flag is not set
        if !self.flags.sign {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn brk(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        self.stack.push_u16(mem, self.registers.PC);
        self.stack.push(mem, self.flags.to_u8());

        self.flags.interrupt_disabled = true;
    }

    fn bvc(&mut self, operand: &Operand) {
        // Branch if the overflow flag is not set
        if !self.flags.overflow {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn bvs(&mut self, operand: &Operand) {
        // Branch if the overflow flag is set
        if self.flags.overflow {
            let offset = self.unwrap_immediate(&operand);
            self.relative_jump(offset);
        }
    }

    fn set_carry_flag(&mut self, value: bool) {
        self.flags.carry = value;
    }

    fn set_decimal_flag(&mut self, value: bool) {
        self.flags.decimal = value;
    }

    fn set_interrupt_flag(&mut self, value: bool) {
        self.flags.interrupt_disabled = value;
    }

    fn set_overflow_flag(&mut self, value: bool) {
        self.flags.overflow = value;
    }

    fn compare(&mut self, operand: &Operand, byte: u8) {
        let value = self.unwrap_immediate(&operand);
        let result: i16 = byte as i16 - value as i16;

        self.flags.carry = (result as u16) < 0x100;
        self.flags.zero = result & 0xFF == 0x00;
        self.flags.sign = result & 0x80 == 0x80;
    }

    fn dec(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);
        let addr = self.unwrap_address(&operand);
        let result = value - 1;

        self.write_byte(addr, result);

        self.flags.sign = result & 0x80 == 0x80;
        self.flags.zero = result & 0xFF == 0x00;
    }

    fn dex(&mut self) {
        self.registers.X -= 0x01;

        self.flags.sign = self.registers.X & 0x80 == 0x80;
        self.flags.zero = self.registers.X & 0xFF == 0x00;
    }

    fn dey(&mut self) {
        self.registers.Y -= 0x01;

        self.flags.sign = self.registers.Y & 0x80 == 0x80;
        self.flags.zero = self.registers.Y & 0xFF == 0x00;
    }

    fn eor(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);
        let result = self.registers.A ^ value;

        self.registers.A = result;

        self.flags.sign = result & 0x80 == 0x80;
        self.flags.zero = result & 0xFF == 0x00;
    }

    fn inc(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);
        let addr = self.unwrap_address(&operand);
        let result = value + 1;

        self.write_byte(addr, result);

        self.flags.sign = result & 0x80 == 0x80;
        self.flags.zero = result & 0xFF == 0x00;
    }

    fn inx(&mut self) {
        self.registers.X += 0x01;

        self.flags.sign = self.registers.X & 0x80 == 0x80;
        self.flags.zero = self.registers.X & 0xFF == 0x00;
    }

    fn iny(&mut self) {
        self.registers.Y += 0x01;

        self.flags.sign = self.registers.Y & 0x80 == 0x80;
        self.flags.zero = self.registers.Y & 0xFF == 0x00;
    }

    fn jmp(&mut self, operand: &Operand) {
        let value = self.unwrap_address(&operand);
        self.registers.PC = value;
    }

    fn jsr(&mut self, operand: &Operand) {
        let addr = self.unwrap_address(&operand);
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        self.stack.push_u16(mem, self.registers.PC);
        self.registers.PC = addr;
    }

    fn lda(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);

        self.registers.A = value;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;
    }

    fn ldx(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);

        self.registers.X = value;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;
    }

    fn ldy(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);

        self.registers.Y = value;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;
    }

    fn lsr(&mut self, operand: &Operand) {
        // Accumulator is the implied register here
        let value = if let &Operand::Implied = operand {
            self.registers.A
        } else {
            self.unwrap_immediate(&operand)
        };

        self.flags.carry = value & 0x01 == 0x01;

        let value = value >> 0x01;

        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;

        if let &Operand::Implied = operand {
            self.registers.A = value;
        } else {
            let addr = self.unwrap_address(&operand);
            self.memory.write_byte(addr, value);
        }
    }

    fn nop(&self) {
        // Nothing. No Operation.
    }

    fn ora(&mut self, operand: &Operand) {
        let value = self.unwrap_immediate(&operand);
        let result = self.registers.A | value;

        self.flags.sign = result & 0x80 == 0x80;
        self.flags.zero = result & 0xFF == 0x00;

        self.registers.A = result;
    }

    fn pha(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        self.stack.push(mem, self.registers.A);
    }

    fn php(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        self.stack.push(mem, self.flags.to_u8());
    }

    fn pla(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        let value = self.stack.pop(mem).unwrap();

        self.registers.A = value;
    }

    fn plp(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        let value = self.stack.pop(mem).unwrap();

        self.flags = value.into();
    }

    fn rts(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];
        let addr = self.stack.pop_u16(mem).unwrap();

        self.registers.PC = addr;
    }

    fn rol(&mut self, operand: &Operand) {
        let value = if let &Operand::Implied = operand {
            self.registers.A
        } else {
            self.unwrap_immediate(&operand)
        };

        let carry = value & 0x80 == 0x80;

        let value = if self.flags.carry {
            (value << 0x01) | 0x01
        } else {
            value << 0x01
        };

        self.flags.carry = carry;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;

        if let &Operand::Implied = operand {
            self.registers.A = value;
        } else {
            let addr = self.unwrap_address(&operand);
            self.memory.write_byte(addr, value);
        }
    }
    fn ror(&mut self, operand: &Operand) {
        let value = if let &Operand::Implied = operand {
            self.registers.A
        } else {
            self.unwrap_immediate(&operand)
        };

        let carry = value & 0x01 == 0x01;   // Carry flag is the low bit in a ROR

        let value = if self.flags.carry {
            (value >> 0x01) | 0x80
        } else {
            value >> 0x01
        };

        self.flags.carry = carry;
        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;

        if let &Operand::Implied = operand {
            self.registers.A = value;
        } else {
            let addr = self.unwrap_address(&operand);
            self.memory.write_byte(addr, value);
        }
    }

    fn rti(&mut self) {
        let mut mem = &mut self.memory[STACK_START..STACK_END + 0x01];

        let value = self.stack.pop(mem).unwrap();
        self.flags = value.into();
    }

    fn sbc(&mut self, operand: &Operand) {
        let carry = if self.flags.carry { 0 } else { 1 };

        let value = self.unwrap_immediate(&operand) as i16;
        let value_signs = self.registers.A & 0x80 == 0x80 && value & 0x80 == 0x80;

        // Do normal binary arithmetic first
        let mut result = self.registers.A as i16 - value as i16 - carry as i16;

        self.flags.zero = result as u8 & 0xFF == 0x00;
        self.flags.sign = result & 0x80 == 0x80;

        if self.flags.sign != value_signs {
            self.flags.overflow = true;
        }

        if self.flags.decimal {
            if (((self.registers.A as i16) & 0x0F) - carry as i16) < ((value as i16) & 0x0F) {
                result -= 0x06;
            }
            if (result as u16) > 0x99 {
                result -= 0x60;
            }
        }

        self.flags.carry = (result as u16) < 0x100;
        self.registers.A = result as u8;
    }

    fn sta(&mut self, operand: &Operand) {
        let addr = self.unwrap_address(&operand);
        let value = self.registers.A;

        self.write_byte(addr, value);
    }

    fn stx(&mut self, operand: &Operand) {
        let addr = self.unwrap_address(&operand);
        let value = self.registers.X;

        self.write_byte(addr, value);
    }

    fn sty(&mut self, operand: &Operand) {
        let addr = self.unwrap_address(&operand);
        let value = self.registers.Y;

        self.write_byte(addr, value);
    }

    fn tax(&mut self) {
        self.registers.X = self.registers.A;

        self.flags.sign = self.registers.A & 0x80 == 0x80;
        self.flags.zero = self.registers.A & 0xFF == 0x00;
    }

    fn tay(&mut self) {
        self.registers.Y = self.registers.A;

        self.flags.sign = self.registers.A & 0x80 == 0x80;
        self.flags.zero = self.registers.A & 0xFF == 0x00;
    }

    fn tsx(&mut self) {
        let value = self.stack.pointer as u8;
        self.registers.X = value;

        self.flags.sign = value & 0x80 == 0x80;
        self.flags.zero = value & 0xFF == 0x00;
    }

    fn txa(&mut self) {
        self.registers.A = self.registers.X;

        self.flags.sign = self.registers.X & 0x80 == 0x80;
        self.flags.zero = self.registers.X & 0xFF == 0x00;
    }

    fn txs(&mut self) {
        self.stack.pointer = self.registers.X as usize;
    }

    fn tya(&mut self) {
        self.registers.A = self.registers.Y;

        self.flags.sign = self.registers.Y & 0x80 == 0x80;
        self.flags.zero = self.registers.Y & 0xFF == 0x00;
    }

    fn relative_jump(&mut self, offset: u8) {
        // If the sign bit is there, negate the PC by the difference
        // between 256 and the offset
        if offset & 0x80 == 0x80 {
            self.registers.PC -= 0x100 - offset as u16;
        } else {
            self.registers.PC += offset as u16;
        }
    }

    /// Convenience wrapper for accessing a byte
    /// in memory
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory.read_byte(addr)
    }

    /// Convenience wrapper for writing a byte
    /// to memory
    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory.write_byte(addr, byte);
    }

    /// Convenience wrapper for accessing a word
    /// in memory
    fn read_u16(&self, addr: u16) -> u16 {
        self.memory.read_u16(addr)
    }
}
