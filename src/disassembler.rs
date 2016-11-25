use byteorder::{ByteOrder, LittleEndian};

use opcodes::{AddressingMode, OpCode};

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(raw: &[u8]) -> String {
        let mut result = String::new();

        let mut i: usize = 0;
        while i < raw.len() {
            let opcode = OpCode::from_raw_byte(raw[i]);
            let val = match opcode.mode {
                AddressingMode::Implied |
                AddressingMode::Accumulator => String::from(""),
                AddressingMode::Immediate => format!(" #${:02X}", raw[i + 0x01]),
                AddressingMode::Indirect => {
                    format!(" (${:04X})", LittleEndian::read_u16(&raw[i + 0x01..]))
                }
                AddressingMode::Relative => format!(" #${:02X}", raw[i + 0x01]),
                AddressingMode::ZeroPage => format!(" ${:02X}", raw[i + 0x01]),
                AddressingMode::ZeroPageX => format!(" ${:02X},X", raw[i + 0x01]),
                AddressingMode::ZeroPageY => format!(" ${:02X},Y", raw[i + 0x01]),
                AddressingMode::Absolute => {
                    format!(" ${:04X}", LittleEndian::read_u16(&raw[i + 0x01..]))
                }
                AddressingMode::AbsoluteX => {
                    format!(" ${:04X},X", LittleEndian::read_u16(&raw[i + 0x01..]))
                }
                AddressingMode::AbsoluteY => {
                    format!(" ${:04X},Y", LittleEndian::read_u16(&raw[i + 0x01..]))
                }
                AddressingMode::IndirectX => format!(" (${:02X},X)", raw[i + 0x01]),
                AddressingMode::IndirectY => format!(" (${:02X}),Y", raw[i + 0x01]),
            };
            let opcode_text = format!("{}{}\n", opcode.mnemonic, val);
            result.push_str(&opcode_text);
            i += opcode.length as usize;
        }

        result
    }

    /// Returns a Vector of Strings where each entry
    /// is a non-empty line from some text input, with
    /// all leading and trailing whitespace removed.
    fn clean_asm<I>(input: I) -> Vec<String>
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
        let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            LDA #$20
            STA $4400

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_indirect_jmp() {
        let code: Vec<u8> = vec![0x6C, 0x00, 0x44];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            JMP ($4400)

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_relative_addressing() {
        let code: Vec<u8> = vec![0xD0, 0xFF];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            BNE #$FF

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_zero_page_addressing() {
        let code: Vec<u8> = vec![0xA5, 0x35];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            LDA $35

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_zero_page_indexed_addressing() {
        let code: Vec<u8> = vec![0x95, 0x44, 0x96, 0xFE];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            STA $44,X
            STX $FE,Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_absolute_addressing() {
        let code: Vec<u8> = vec![0x8D, 0x00, 0x44];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            STA $4400

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_absolute_indexed_addressing() {
        let code: Vec<u8> = vec![0x9D, 0x00, 0x44, 0x99, 0xFE, 0xFF];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            STA $4400,X
            STA $FFFE,Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn can_disassemble_indirect_indexed_addressing() {
        let code: Vec<u8> = vec![0x81, 0x44, 0x91, 0xFE];
        let asm = Disassembler::disassemble(&code);

        assert_eq!(Disassembler::clean_asm("
        
            STA ($44,X)
            STA ($FE),Y

        "),
                   Disassembler::clean_asm(asm));
    }

    #[test]
    fn move_memory_down_test() {
        let code: Vec<u8> = vec![0xA0, 0x00, 0xAE, 0x00, 0x00, 0xF0, 0x10, 0xB1, 0x02, 0x91, 0x03,
                                 0xC8, 0xD0, 0xF9, 0xEE, 0x02, 0x00, 0xEE, 0x03, 0x00, 0xCA, 0xD0,
                                 0xF0, 0xAE, 0x01, 0x00, 0xF0, 0x08, 0xB1, 0x02, 0x91, 0x03, 0xC8,
                                 0xCA, 0xD0, 0xF8, 0x60];

        let asm = Disassembler::disassemble(&code);

        print!("{}", asm);
    }
}
