use std::iter::Peekable;

use ::opcodes::{AddressingMode, OpCode};
use assembler::token::{LexerToken, ParserToken};

#[derive(Debug, PartialEq)]
pub struct ParserError {
    pub message: String,
}

impl ParserError {
    fn expected_instruction(line: u32) -> ParserError {
        ParserError::from(format!("Instruction expected. Line {}", line))
    }

    fn invalid_opcode_addressing_mode_combination(line: u32) -> ParserError {
        ParserError::from(format!("Invalid addressing mode for opcode. Line {}", line))
    }

    fn unexpected_eol(line: u32) -> ParserError {
        ParserError::from(format!("Unexpected end of line. Line {}", line))
    }

    fn expected_eol(line: u32) -> ParserError {
        ParserError::from(format!("Expected end of line. Line {}", line))
    }

    fn cannot_parse_address(line: u32) -> ParserError {
        ParserError::from(format!("Unable to parse address. Line {}", line))
    }

    fn unexpected_token(line: u32) -> ParserError {
        ParserError::from(format!("Unexpected token. Line {}", line))
    }
}

impl From<String> for ParserError {
    fn from(error: String) -> ParserError {
        ParserError { message: error }
    }
}

impl<'a> From<&'a str> for ParserError {
    fn from(error: &str) -> ParserError {
        ParserError { message: error.into() }
    }
}

pub struct Parser {
    line: u32,
}

/// Parser processes a list of 6502 Assembly tokens
impl Parser {
    pub fn new() -> Parser {
        Parser { line: 0 }
    }

    pub fn parse(&mut self, tokens: Vec<Vec<LexerToken>>) -> Result<Vec<ParserToken>, ParserError> {
        let mut result = Vec::new();

        for line in &tokens {
            self.line += 1;

            let mut peeker = line.iter().peekable();

            // Skip blank lines
            if let None = peeker.peek() {
                continue;
            }

            let next = *peeker.peek().unwrap();

            if let &LexerToken::Ident(ref ident) = next {
                // Check if this is an opcode
                if Self::is_opcode(ident.clone()) {
                    // Yep its an opcode, lets figure out its addressing mode
                    peeker.next();
                    let mut opcode = self.consume_opcode(&mut peeker, ident.clone())?;
                    result.append(&mut opcode);
                } else {
                    // Skip the ident and we'll check what is next
                    let original_ident = peeker.next().unwrap();
                    // if there is nothing else - lets mark this as a Label and move on
                    if let None = peeker.peek() {
                        result.push(ParserToken::Label(ident.clone()));
                        continue;
                    }

                    // A colon after the ident also indicates a label
                    let next = *peeker.peek().unwrap();
                    if let &LexerToken::Colon = next {
                        result.push(ParserToken::Label(ident.clone()));
                        continue;
                    }

                    // Is the next one a label as well? Thats an error:
                    if let &LexerToken::Ident(ref ident) = next {
                        // Lets add the original as a label
                        if let &LexerToken::Ident(ref original_ident) = original_ident {
                            result.push(ParserToken::Label(original_ident.clone()));
                        }

                        if !Self::is_opcode(ident.clone()) {
                            return Err(ParserError::expected_instruction(self.line));
                        } else {
                            // Oh it is an opcode after the label - consume it
                            let mut opcode = self.consume_opcode(&mut peeker, ident.clone())?;
                            result.append(&mut opcode);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn is_opcode<S>(mnemonic: S) -> bool
        where S: Into<String>
    {
        if let Some(opcode) = OpCode::from_mnemonic(mnemonic) {
            true
        } else {
            false
        }
    }

    fn consume_opcode<'a, I, S>(&mut self,
                                mut peeker: &mut Peekable<I>,
                                ident: S)
                                -> Result<Vec<ParserToken>, ParserError>
        where I: Iterator<Item = &'a LexerToken>,
              S: Into<String>
    {
        // If there is nothing else after this opcode.. lets check if there is
        // a matching opcode with an implied addressing mode
        if let None = peeker.peek() {
            if let Some(opcode) =
                   OpCode::from_mnemonic_and_addressing_mode(ident, AddressingMode::Implied) {
                return Ok(vec![ParserToken::OpCode(opcode)]);
            } else {
                return Err(ParserError::invalid_opcode_addressing_mode_combination(self.line));
            }
        } else {
            // TODO: Complete this
            // Jump over the opcode
            peeker.next();

            // Check the next token, is it an address?
            let next = *peeker.peek().unwrap();
            if let &LexerToken::Address(ref address) = next {
                // Its an address. What sort of address?
                if address.len() == 2 || address.len() == 4 {
                    // Its zero-page or absolute.. lets try and convert it to a raw byte
                    let (bytes, addressing_mode) = if address.len() == 2 {
                        // Its a 1 byte address
                        if let Ok(raw_byte) = u8::from_str_radix(&address[..], 16) {
                            (vec![raw_byte], AddressingMode::ZeroPage)
                        } else {
                            return Err(ParserError::cannot_parse_address(self.line));
                        }
                    } else {
                        // Its a 2 byte address
                        if let Ok(low_byte) = u8::from_str_radix(&address[2..], 16) {
                            if let Ok(high_byte) = u8::from_str_radix(&address[0..2], 16) {
                                (vec![low_byte, high_byte], AddressingMode::Absolute)
                            } else {
                                return Err(ParserError::cannot_parse_address(self.line));
                            }
                        } else {
                            return Err(ParserError::cannot_parse_address(self.line));
                        }
                    };
                    // consume the address and peek what is next:
                    peeker.next();
                    if let None = peeker.peek() {
                        // Nothing else.. find an opcode with this ident and addressing mode
                        if let Some(opcode) =
                               OpCode::from_mnemonic_and_addressing_mode(ident, addressing_mode) {
                            // We found one..
                            let mut final_vec = vec![ParserToken::OpCode(opcode)];
                            // Push the address bytes into the result
                            for b in bytes {
                                final_vec.push(ParserToken::RawByte(b));
                            }
                            return Ok(final_vec);
                        } else {
                            return Err(ParserError::invalid_opcode_addressing_mode_combination(self.line));
                        }
                    }

                    // There is something after this address - if its
                    // a comma, then we're peachy. If its something else.. Thats
                    // an error.
                    let next = *peeker.peek().unwrap();
                    if let &LexerToken::Comma = next {
                        // Yes, its a comma. Consume it and check what is next
                        peeker.next();
                        // If theres nothing after the comma thats an error
                        if let None = peeker.peek() {
                            return Err(ParserError::unexpected_eol(self.line));
                        }

                        let next = *peeker.peek().unwrap();
                        if let &LexerToken::Ident(ref register) = next {
                            let register = register.to_uppercase();
                            if register != "X" && register != "Y" {
                                return Err(ParserError::unexpected_token(self.line));
                            }
                            let addressing_mode = if register == "X" {
                                if addressing_mode == AddressingMode::ZeroPage {
                                    AddressingMode::ZeroPageX
                                } else {
                                    AddressingMode::AbsoluteX
                                }
                            } else {
                                if addressing_mode == AddressingMode::ZeroPage {
                                    AddressingMode::ZeroPageY
                                } else {
                                    AddressingMode::AbsoluteY
                                }
                            };
                            if let Some(opcode) =
                                   OpCode::from_mnemonic_and_addressing_mode(ident, addressing_mode) {
                                // We found one..
                                let mut final_vec = vec![ParserToken::OpCode(opcode)];
                                // Push the address bytes into the result
                                for b in bytes {
                                    final_vec.push(ParserToken::RawByte(b));
                                }
                                return Ok(final_vec);
                            } else {
                                return Err(ParserError::invalid_opcode_addressing_mode_combination(self.line));
                            }
                        } else {
                            return Err(ParserError::unexpected_token(self.line));
                        }
                    } else {
                        return Err(ParserError::unexpected_token(self.line));
                    }
                    let next = *peeker.peek().unwrap();

                } else {
                    return Err(ParserError::cannot_parse_address(self.line));
                }
            }
            Ok(vec![ParserToken::Label("BLAH".into())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, LexerToken, ParserToken};
    use ::opcodes::{AddressingMode, OpCode};

    #[test]
    fn can_parse_labels_via_lonely_label() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into())],
                          vec![LexerToken::Ident("START".into())]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens).unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into()), ParserToken::Label("START".into())],
                   &result[..]);
    }

    #[test]
    fn can_parse_labels_via_colon_terminator() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into())], vec![LexerToken::Colon]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens).unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into())], &result[..]);
    }

    #[test]
    fn can_parse_opcodes_after_labels_on_one_line() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into()),
                               LexerToken::Ident("LDA".into()),
                               LexerToken::Address("4400".into())]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens).unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into()),
                     ParserToken::OpCode(OpCode::from_mnemonic_and_addressing_mode("LDA", AddressingMode::Absolute).unwrap()),
                     ParserToken::RawByte(0),
                     ParserToken::RawByte(68)],
                   &result[..]);
    }

    #[test]
    fn can_detect_double_labels_on_one_line() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into()),
                               LexerToken::Ident("START".into())]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens);

        assert_eq!(Err(ParserError::expected_instruction(1)), result);
    }

    #[test]
    fn can_detect_opcode_with_implied_addressing_mode() {
        let tokens = vec![vec![LexerToken::Ident("CLC".into())]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens).unwrap();

        assert_eq!(&[ParserToken::OpCode(OpCode::from_mnemonic_and_addressing_mode("CLC", AddressingMode::Implied).unwrap())], &result[..]);
    }

    #[test]
    fn can_detect_opcode_with_correct_absolute_x_addressing_mode() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into()),
                               LexerToken::Ident("LDA".into()),
                               LexerToken::Address("4400".into()),
                               LexerToken::Comma,
                               LexerToken::Ident("X".into())]];

        let mut parser = Parser::new();
        let result = parser.parse(tokens).unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into()),
                     ParserToken::OpCode(OpCode::from_mnemonic_and_addressing_mode("LDA", AddressingMode::AbsoluteX).unwrap()),
                     ParserToken::RawByte(0),
                     ParserToken::RawByte(68)],
                   &result[..]);
    }
}