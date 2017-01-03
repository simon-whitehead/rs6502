use std;

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use ::opcodes::{AddressingMode, OpCode};
use assembler::lexer::{Lexer, LexerError};
use assembler::parser::{Parser, ParserError};
use assembler::token::{LexerToken, ParserToken};

#[derive(Debug, PartialEq)]
pub struct Label(u16);

#[derive(Debug)]
pub struct AssemblerError {
    message: String,
}

impl AssemblerError {
    fn unknown_label<S>(label: S) -> AssemblerError
        where S: Into<String> + std::fmt::Display
    {
        AssemblerError::from(format!("Unknown label: '{}'", label))
    }

    fn relative_offset_too_large<S>(context: S) -> AssemblerError
        where S: Into<String> + Display
    {
        AssemblerError::from(format!("Branch too far: {}", context))
    }
}

impl From<String> for AssemblerError {
    fn from(error: String) -> AssemblerError {
        AssemblerError { message: error }
    }
}

impl From<LexerError> for AssemblerError {
    fn from(error: LexerError) -> AssemblerError {
        AssemblerError { message: error.message }
    }
}

impl From<ParserError> for AssemblerError {
    fn from(error: ParserError) -> AssemblerError {
        AssemblerError { message: error.message }
    }
}

#[derive(Debug)]
pub struct CodeSegment {
    pub address: u16,
    pub code: Vec<u8>,
}

pub struct Assembler {
    symbol_table: HashMap<String, Label>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler { symbol_table: HashMap::new() }
    }

    pub fn assemble_string<S, O>(&mut self,
                                 code: S,
                                 offset: O)
                                 -> Result<Vec<CodeSegment>, AssemblerError>
        where S: Into<String>,
              O: Into<Option<u16>>
    {
        let code = code.into();
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string(code)?;
        let mut parser = Parser::new();
        let tokens = parser.parse(tokens)?;

        Ok(self.assemble(tokens, offset)?)
    }

    pub fn assemble_file<P, O>(&mut self,
                               path: P,
                               offset: O)
                               -> Result<Vec<CodeSegment>, AssemblerError>
        where P: AsRef<Path>,
              O: Into<Option<u16>>
    {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_file(path)?;
        let mut parser = Parser::new();
        let tokens = parser.parse(tokens)?;

        Ok(self.assemble(tokens, offset)?)
    }

    fn assemble<O>(&mut self,
                   tokens: Vec<ParserToken>,
                   offset: O)
                   -> Result<Vec<CodeSegment>, AssemblerError>
        where O: Into<Option<u16>>
    {
        let mut addr: u16 = offset.into().unwrap_or(0);

        // First, index the labels so we have addresses for them
        self.index_labels(&tokens, addr);

        // Now assemble the code
        let mut result = Vec::new();
        let mut last_addressing_mode = AddressingMode::Absolute;
        let mut current_segment = CodeSegment {
            address: addr,
            code: Vec::new(),
        };

        for token in tokens {
            // Push an opcode into the output and increment our address
            // offset
            if let ParserToken::OpCode(opcode) = token {
                current_segment.code.push(opcode.code);
                addr += opcode.length as u16;
                last_addressing_mode = opcode.mode;
            } else if let ParserToken::OrgDirective(org_addr) = token {
                if current_segment.code.len() > 0 {
                    result.push(current_segment);
                }
                current_segment = CodeSegment {
                    address: org_addr,
                    code: Vec::new(),
                };
                addr = org_addr;
            } else if let ParserToken::RawByte(byte) = token {
                // Push raw bytes directly into the output
                current_segment.code.push(byte);
            } else if let ParserToken::RawBytes(bytes) = token {
                // Push raw bytes directly into output
                for b in &bytes {
                    current_segment.code.push(*b);
                }
            } else if let ParserToken::LabelArg(ref label) = token {
                // Labels as arguments should be in the symbol table, look
                // it up and calculate the address direction/location
                if let Some(&Label(label_addr)) = self.symbol_table.get(label) {
                    if last_addressing_mode == AddressingMode::Absolute {
                        let low_byte = (label_addr & 0xFF) as u8;
                        let high_byte = ((label_addr >> 8) & 0xFF) as u8;

                        current_segment.code.push(low_byte);
                        current_segment.code.push(high_byte);
                    } else {
                        // Its relative.. lets generate a relative branch
                        if addr > label_addr {
                            let distance = (label_addr as i16 - addr as i16) as i8;
                            if distance < -128 || distance > 127 {
                                return Err(AssemblerError::relative_offset_too_large(format!("Attempted jump to {} at {:04X}", label, addr)));
                            }
                            current_segment.code.push(distance as u8);
                        } else {
                            let distance = label_addr - addr;
                            if distance > 127 {
                                return Err(AssemblerError::relative_offset_too_large(format!("Attempted jump to {} at {:04X}", label, addr)));
                            }
                            current_segment.code.push(distance as u8);
                        }
                    }
                } else {
                    return Err(AssemblerError::unknown_label(label.clone()));
                }
            }
        }

        result.push(current_segment);

        Ok(result)
    }

    /// Stores all labels in the code in a Symbol table for lookup later
    fn index_labels(&mut self, tokens: &[ParserToken], offset: u16) {
        let mut addr: u16 = offset;
        let mut last_addressing_mode = AddressingMode::Absolute;

        for token in tokens {
            if let &ParserToken::Label(ref label) = token {
                // Insert a label with the specified memory address
                // as its offset
                self.symbol_table.insert(label.clone(), Label(addr));
            } else if let &ParserToken::OpCode(opcode) = token {
                // Add the length of this opcode to our
                // address offset
                addr += opcode.length as u16;
                last_addressing_mode = opcode.mode;
            } else if let &ParserToken::OrgDirective(new_addr) = token {
                addr = new_addr
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_assemble_basic_code() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            LDA $4400
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xAD, 0x00, 0x44], &segments[0].code[..]);
    }

    #[test]
    fn can_jump_to_label_behind() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            MAIN LDA $4400
            PHA
            JMP MAIN
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xAD, 0x00, 0x44, 0x48, 0x4C, 0x00, 0x00],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_jump_to_label_with_colon_behind() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            MAIN:
                LDA $4400
                PHA
                JMP MAIN
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xAD, 0x00, 0x44, 0x48, 0x4C, 0x00, 0x00],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_jump_to_label_ahead() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            JMP MAIN
            PHA
            LDX #15
            MAIN LDA $4400
            RTS
        ",
                             None)
            .unwrap();

        assert_eq!(&[0x4C, 0x06, 0x00, 0x48, 0xA2, 0x0F, 0xAD, 0x00, 0x44, 0x60],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_use_variables() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            MAIN_ADDRESS = $0000
            MAIN:
            LDX #15
            JMP MAIN_ADDRESS
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xA2, 0x0F, 0x4C, 0x00, 0x00], &segments[0].code[..]);
    }

    #[test]
    fn can_use_variables_assigned_to_variables() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            MAIN_ADDRESS = $0000
            MAIN_ADDRESS_INDIRECT_ONE = MAIN_ADDRESS
            MAIN_ADDRESS_INDIRECT_TWO = MAIN_ADDRESS_INDIRECT_ONE
            MAIN:
            LDX #15
            JMP MAIN_ADDRESS_INDIRECT_TWO
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xA2, 0x0F, 0x4C, 0x00, 0x00], &segments[0].code[..]);
    }

    #[test]
    fn can_assemble_clearmem_implementation() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            CLRMEM  LDA #$00
                    TAY             
            CLRM1   STA ($FF),Y
                    INY             
                    DEX             
                    BNE CLRM1       
                    RTS             
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xA9, 0x00, 0xA8, 0x91, 0xFF, 0xC8, 0xCA, 0xD0, 0xFA, 0x60],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_assemble_clearmem_implementation_that_jumps_forward_and_is_lowercase() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            jmp     clrmem
            lda     #$00
            beq     clrm1
            nop
            nop
            clrm1   sta ($ff),y
                    iny             
                    dex             
                    bne clrm1       
                    rts             
            clrmem  lda #$00
                    tay             
            jmp     clrm1
        ",
                             None)
            .unwrap();

        assert_eq!(&[0x4C, 0x10, 0x00, 0xA9, 0x00, 0xF0, 0x02, 0xEA, 0xEA, 0x91, 0xFF, 0xC8,
                     0xCA, 0xD0, 0xFA, 0x60, 0xA9, 0x00, 0xA8, 0x4C, 0x09, 0x00],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_assemble_clearmem_implementation_that_jumps_forward() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            JMP     CLRMEM
            LDA     #$00
            BEQ     CLRM1
            NOP
            NOP
            BRK
            CLRM1   STA ($FF),Y
                    INY             
                    DEX             
                    BNE CLRM1       
                    RTS             
            CLRMEM  LDA #$00
                    TAY             
            JMP     CLRM1
        ",
                             None)
            .unwrap();

        assert_eq!(&[0x4C, 0x11, 0x00, 0xA9, 0x00, 0xF0, 0x03, 0xEA, 0xEA, 0x00, 0x91, 0xFF,
                     0xC8, 0xCA, 0xD0, 0xFA, 0x60, 0xA9, 0x00, 0xA8, 0x4C, 0x0A, 0x00],
                   &segments[0].code[..]);
    }

    #[test]
    fn can_use_variables_for_indirect_addressing() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            MAIN_ADDRESS = $0000
            MAIN:
            LDX #15
            LDA (MAIN_ADDRESS),Y
        ",
                             None)
            .unwrap();

        assert_eq!(&[0xA2, 0x0F, 0xB1, 0x00, 0x00], &segments[0].code[..]);
    }

    #[test]
    fn can_assign_code_segments_to_different_memory_addresses() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            .ORG $C000
            LDA #$FF
            STA $2000

            .ORG $100
            LDA #$AA
            STA $2001
        ",
                             None)
            .unwrap();

        assert_eq!(0xC000, segments[0].address);
        assert_eq!(0x0100, segments[1].address);
    }

    #[test]
    fn can_jump_between_code_segments() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            .ORG $C000
            JMP CALLBACK

            .ORG $2000
            LDA #$AA
            STA $2001

            CALLBACK
            LDX #$0A
        ",
                             None)
            .unwrap();

        assert_eq!(0xC000, segments[0].address);
        assert_eq!(0x2000, segments[1].address);

        assert_eq!(0x05, segments[0].code[0x01]);
        assert_eq!(0x20, segments[0].code[0x02]);
    }

    #[test]
    fn can_dump_raw_bytes() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            .ORG $C000

            .BYTE #$40, #10, #$0A
        ",
                             None)
            .unwrap();

        assert_eq!(&[64, 10, 10], &segments[0].code[..]);
    }

    #[test]
    fn can_dump_single_raw_byte() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            .ORG $C000

            .BYTE #$FF
        ",
                             None)
            .unwrap();

        assert_eq!(&[255], &segments[0].code[..]);
    }

    #[test]
    fn can_dump_bytes_with_other_code() {
        let mut assembler = Assembler::new();
        let segments = assembler.assemble_string("
            .ORG $C000
            JMP CALLBACK
            .BYTE #$0A

            .ORG $2000
            LDA #$AA
            STA $2001
            .BYTE #$FE, #$CB

            CALLBACK
            LDX #$0A
        ",
                             None)
            .unwrap();

        assert_eq!(0x0A, segments[0].code[3]);
        assert_eq!(&[0xFE, 0xCB], &segments[1].code[5..7]);

        assert_eq!(0xC000, segments[0].address);
        assert_eq!(0x2000, segments[1].address);

        assert_eq!(0x05, segments[0].code[0x01]);
        assert_eq!(0x20, segments[0].code[0x02]);
    }
}