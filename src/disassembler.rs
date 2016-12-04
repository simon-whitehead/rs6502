use byteorder::{ByteOrder, LittleEndian};

use opcodes::{AddressingMode, OpCode};

pub struct Disassembler {
    /// Determines whether byte offsets are generated
    /// in the Assembly output
    disable_offsets: bool,

    /// Determines whether opcodes are generated
    /// in the Assembly output
    disable_opcodes: bool,
}

/// A 6502 instruction disassembler
impl Disassembler {
    /// Creates a new, default instance of the Disassembler
    ///
    /// # Example
    /// ```
    /// use rs6502::Disassembler;
    ///
    /// let dasm = Disassembler::new();
    ///
    /// let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
    /// let asm = dasm.disassemble(&code);
    ///
    /// assert_eq!(Disassembler::clean_asm("
    ///
    ///     0000 LDA #$20
    ///     0002 STA $4400
    ///
    /// "), Disassembler::clean_asm(asm));
    /// ```
    pub fn new() -> Disassembler {
        Disassembler {
            disable_offsets: false,
            disable_opcodes: true,
        }
    }

    /// Creates an instance of the Disassembler where no
    /// byte offsets are generated in the Assembly output
    ///
    /// # Example
    /// ```
    /// use rs6502::Disassembler;
    ///
    /// let dasm = Disassembler::with_code_only();
    ///
    /// let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
    /// let asm = dasm.disassemble(&code);
    ///
    /// assert_eq!(Disassembler::clean_asm("
    ///
    ///     LDA #$20
    ///     STA $4400
    ///
    /// "), Disassembler::clean_asm(asm));
    /// ```
    pub fn with_code_only() -> Disassembler {
        Disassembler {
            disable_offsets: true,
            disable_opcodes: true,
        }
    }

    /// Creates an instance of the Disassembler with all
    /// available information generated into the output
    ///
    /// # Example
    /// ```
    /// use rs6502::Disassembler;
    ///
    /// let dasm = Disassembler::with_verbose_output();
    ///
    /// let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
    /// let asm = dasm.disassemble(&code);
    ///
    /// assert_eq!(Disassembler::clean_asm("
    ///
    ///     0000 A9 20    LDA #$20
    ///     0002 8D 00 44 STA $4400
    ///
    /// "), Disassembler::clean_asm(asm));
    /// ```
    pub fn with_verbose_output() -> Disassembler {
        Disassembler {
            disable_offsets: false,
            disable_opcodes: false,
        }
    }

    /// Accepts a slice of 6502 bytecodes and translates them
    /// into an assembly String representation
    ///
    /// # Example
    /// ```
    /// use rs6502::Disassembler;
    ///
    /// let dasm = Disassembler::new();
    ///
    /// let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
    /// let asm = dasm.disassemble(&code);
    ///
    /// assert_eq!(Disassembler::clean_asm("
    ///
    ///     0000 LDA #$20
    ///     0002 STA $4400
    ///
    /// "), Disassembler::clean_asm(asm));
    /// ```
    pub fn disassemble(&self, raw: &[u8]) -> String {
        let mut result = String::new();

        let mut i: usize = 0;
        while i < raw.len() {
            let opcode = OpCode::from_raw_byte(raw[i]);

            // Each branch returns the opcode output and the
            // disassembled output
            let val = match opcode.mode {
                AddressingMode::Immediate => {
                    let imm = raw[i + 0x01];
                    (format!("{:02X} {:02X}", opcode.code, imm), format!(" #${:02X}", imm))
                }
                AddressingMode::Indirect => {
                    let b1 = raw[i + 0x01];
                    let b2 = raw[i + 0x02];

                    let addr = LittleEndian::read_u16(&[b1, b2]);

                    (format!("{:02X} {:02X} {:02X}", opcode.code, b1, b2),
                     format!(" (${:04X})", addr))
                }
                AddressingMode::Relative => {
                    let b1 = raw[i + 0x01];
                    let offset = b1 as i8;
                    let addr = if offset < 0 {
                        i - (-offset - 0x02) as usize
                    } else {
                        i + (offset as usize) + 0x02
                    };

                    (format!(" {:02X} {:02X}", opcode.code, b1), format!(" ${:04X}", addr))
                }
                AddressingMode::ZeroPage => {
                    let b1 = raw[i + 0x01];
                    (format!("{:02X} {:02X}", opcode.code, b1), format!(" ${:02X}", b1))
                }
                AddressingMode::ZeroPageX => {
                    let b1 = raw[i + 0x01];
                    (format!("{:02X} {:02X}", opcode.code, b1), format!(" ${:02X},X", b1))
                }
                AddressingMode::ZeroPageY => {
                    let b1 = raw[i + 0x01];
                    (format!("{:02X} {:02X}", opcode.code, b1), format!(" ${:02X},Y", b1))
                }
                AddressingMode::Absolute => {
                    let b1 = raw[i + 0x01];
                    let b2 = raw[i + 0x02];
                    let addr = LittleEndian::read_u16(&[b1, b2]);
                    (format!("{:02X} {:02X} {:02X}", opcode.code, b1, b2),
                     format!(" ${:04X}", addr))
                }
                AddressingMode::AbsoluteX => {
                    let b1 = raw[i + 0x01];
                    let b2 = raw[i + 0x02];
                    let addr = LittleEndian::read_u16(&[b1, b2]);
                    (format!("{:02X} {:02X} {:02X}", opcode.code, b1, b2),
                     format!(" ${:04X},X", addr))
                }
                AddressingMode::AbsoluteY => {
                    let b1 = raw[i + 0x01];
                    let b2 = raw[i + 0x02];
                    let addr = LittleEndian::read_u16(&[b1, b2]);
                    (format!("{:02X} {:02X} {:02X}", opcode.code, b1, b2),
                     format!(" ${:04X},Y", addr))
                }
                AddressingMode::IndirectX => {
                    let b1 = raw[i + 0x01];
                    (format!("{:02X} {:02X}", opcode.code, b1), format!(" (${:02X},X)", b1))
                }
                AddressingMode::IndirectY => {
                    let b1 = raw[i + 0x01];
                    (format!(" {:02X} {:02X}", opcode.code, b1), format!(" (${:02X}),Y", b1))
                }
                _ => ("".into(), "".into()),
            };

            let opcode_text = if self.disable_offsets {
                if self.disable_opcodes {
                    format!("{}{}\n", opcode.mnemonic, val.1)
                } else {
                    format!("{:<8} {}{}\n", val.0, opcode.mnemonic, val.1)
                }
            } else {
                if self.disable_opcodes {
                    format!("{:04X} {}{}\n", i, opcode.mnemonic, val.1)
                } else {
                    format!("{:04X} {:<8} {}{}\n", i, val.0, opcode.mnemonic, val.1)
                }
            };
            result.push_str(&opcode_text);
            i += opcode.length as usize;
        }

        result
    }

    /// Returns a Vector of Strings where each entry
    /// is a non-empty line of assembly instructions, with
    /// all leading and trailing whitespace removed.
    ///
    /// # Example
    ///
    /// ```
    /// use rs6502::Disassembler;
    ///
    /// assert_eq!(Disassembler::clean_asm("
    ///
    ///     0000 LDA #$20
    ///     0002 STA $4400
    ///
    /// "), &["0000 LDA #$20", "0002 STA $4400"]);
    /// ```
    pub fn clean_asm<I>(input: I) -> Vec<String>
        where I: Into<String>
    {
        input.into()
            .lines()
            .map(|line| line.trim())
            .map(String::from)
            .filter(|line| line.len() > 0)
            .collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_disassemble_basic_instructions() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 LDA #$20
            0002 STA $4400

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_indirect_jmp() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0x6C, 0x00, 0x44];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 JMP ($4400)

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_relative_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0xA9, 0x20, 0x69, 0x10, 0xD0, 0xFA];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 LDA #$20
            0002 ADC #$10
            0004 BNE $0000

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_zero_page_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0xA5, 0x35];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 LDA $35

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_zero_page_indexed_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0x95, 0x44, 0x96, 0xFE];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 STA $44,X
            0002 STX $FE,Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_absolute_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0x8D, 0x00, 0x44];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 STA $4400

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_absolute_indexed_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0x9D, 0x00, 0x44, 0x99, 0xFE, 0xFF];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 STA $4400,X
            0003 STA $FFFE,Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_indirect_indexed_addressing() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0x81, 0x44, 0x91, 0xFE];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            0000 STA ($44,X)
            0002 STA ($FE),Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_without_byte_offsets() {
        let dasm = Disassembler::with_code_only();
        let code: Vec<u8> = vec![0x81, 0x35, 0x91, 0xFE];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            STA ($35,X)
            STA ($FE),Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn move_memory_down_test() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0xA0, 0x00, 0xAE, 0x00, 0x00, 0xF0, 0x10, 0xB1, 0x02, 0x91, 0x03,
                                 0xC8, 0xD0, 0xF9, 0xEE, 0x02, 0x00, 0xEE, 0x03, 0x00, 0xCA, 0xD0,
                                 0xF0, 0xAE, 0x01, 0x00, 0xF0, 0x08, 0xB1, 0x02, 0x91, 0x03, 0xC8,
                                 0xCA, 0xD0, 0xF8, 0x60];

        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("

            0000 LDY #$00
            0002 LDX $0000
            0005 BEQ $0017
            0007 LDA ($02),Y
            0009 STA ($03),Y
            000B INY
            000C BNE $0007
            000E INC $0002
            0011 INC $0003
            0014 DEX
            0015 BNE $0007
            0017 LDX $0001
            001A BEQ $0024
            001C LDA ($02),Y
            001E STA ($03),Y
            0020 INY
            0021 DEX
            0022 BNE $001C
            0024 RTS

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn test_memset_implementation() {
        let dasm = Disassembler::new();
        let code: Vec<u8> = vec![0xA9, 0x00, 0xA8, 0x91, 0xFF, 0xC8, 0xCA, 0xD0, 0xFA, 0x60];
        let asm = dasm.disassemble(&code);

        assert_eq!(Disassembler::clean_asm("

            0000 LDA #$00
            0002 TAY
            0003 STA ($FF),Y
            0005 INY
            0006 DEX
            0007 BNE $0003
            0009 RTS

        "),
                   Disassembler::clean_asm(asm));
    }
}
