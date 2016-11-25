use byteorder::{ByteOrder, LittleEndian};

use opcodes::{AddressingMode, OpCode};

pub fn disassemble(raw: &[u8]) -> String {
    let mut result = String::new();

    let mut i: usize = 0;
    while i < raw.len() {
        let opcode = OpCode::from_raw_byte(raw[i]);
        let val = match opcode.mode {
            AddressingMode::Implied |
            AddressingMode::Accumulator => String::from(""),
            AddressingMode::Immediate => format!(" #${:X}", raw[i + 1]),
            AddressingMode::Absolute => format!(" ${:X}", LittleEndian::read_u16(&raw[i + 1..])),
            _ => String::from(""),
        };
        let opcode_text = format!("{}{}\n", opcode.mnemonic, val);
        result.push_str(&opcode_text);
        i += opcode.length as usize;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_disassemble_basic_instructions() {
        let code: Vec<u8> = vec![0xA9, 0x20, 0x8D, 0x00, 0x44];
        let asm = disassemble(&code);

        assert_eq!(clean_asm("
        
            LDA #$20
            STA $4400

        "),
                   clean_asm(asm));
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
