use std::iter::Peekable;

use ::opcodes::{AddressingMode, OpCode};
use assembler::token::Token;

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
    tokens: Vec<Vec<Token>>,
    line: u32,
}

/// Parser processes a list of 6502 Assembly tokens
impl Parser {
    pub fn new(tokens: Vec<Vec<Token>>) -> Parser {
        Parser {
            tokens: tokens,
            line: 0,
        }
    }

    /// Processes its tokens and either returns them to the caller
    /// or produces an error
    pub fn parse(&mut self) -> Result<Vec<Vec<Token>>, ParserError> {
        let successful_result = self.tokens.iter().map(|v| v.clone()).collect();

        for line in &self.tokens {
            self.line += 1;
            let mut peeker = line.iter().peekable();
            // Check what starts the line
            let token = peeker.peek().unwrap().clone();

            match *token {
                Token::Label(_) => {
                    // if its a label, consume it and move on
                    peeker.next();
                    if let None = peeker.peek() {
                        // If its just a label, thats fine
                        return Ok(successful_result);
                    }
                    let next = *peeker.peek().unwrap();
                    if let &Token::OpCode(ref mnemonic) = next {
                        peeker.next();
                        Self::validate_opcode(&mut peeker, mnemonic, self.line)?;
                    } else {
                        return Err(ParserError::expected_instruction(self.line));
                    }
                }
                Token::OpCode(ref mnemonic) => {
                    peeker.next();
                    Self::validate_opcode(&mut peeker, mnemonic, self.line)?;
                }
                _ => (),
            }
        }

        Ok(successful_result)
    }

    fn validate_opcode<'a, I>(mut peeker: &mut Peekable<I>,
                              mnemonic: &str,
                              line: u32)
                              -> Result<(), ParserError>
        where I: Iterator<Item = &'a Token>
    {
        // Determine an addressing mode that was attempted,
        let addressing_mode = if let None = peeker.peek() {
            // if there was no token, then its implied
            AddressingMode::Implied
        } else {
            // Otherwise, lets try and convert
            // the next token to a mode
            let next = *peeker.peek().unwrap();
            // Skip the argument
            peeker.next();
            next.to_addressing_mode()
        };

        if let Some(opcode) = OpCode::from_mnemonic_and_addressing_mode(mnemonic.clone(),
                                                                        addressing_mode) {
            // There should be nothing else now:
            if let None = peeker.peek() {
                Ok(())
            } else {
                Err(ParserError::expected_eol(line))
            }
        } else {
            Err(ParserError::invalid_opcode_addressing_mode_combination(line))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, Token};

    #[test]
    fn errors_on_multiple_labels() {
        let mut parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                               Token::Label("METHOD".into()),
                                               Token::OpCode("LDA".into()),
                                               Token::Immediate("10".into(),
                                                                ImmediateBase::Base16)]]);

        assert_eq!(Err(ParserError::expected_instruction(1)), parser.parse());
    }

    #[test]
    fn does_not_error_on_single_label() {
        let mut parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                               Token::OpCode("LDA".into()),
                                               Token::Immediate("10".into(),
                                                                ImmediateBase::Base16)]]);

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("10".into(), ImmediateBase::Base16)],
                   &parser.parse().unwrap()[0][..]);
    }

    #[test]
    fn can_detect_invalid_addressing_modes() {
        let mut parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                               Token::OpCode("LDX".into()),
                                               Token::IndirectY("10".into())]]);

        assert_eq!(Err(ParserError::invalid_opcode_addressing_mode_combination(1)),
                   parser.parse());
    }

    #[test]
    fn does_not_error_on_valid_addressing_modes() {
        let mut parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                               Token::OpCode("LDA".into()),
                                               Token::IndirectY("10".into())]]);

        let result = parser.parse().unwrap();
        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::IndirectY("10".into())],
                   &result[0][..]);
    }

    #[test]
    fn does_not_error_on_label_only_line() {
        let mut parser = Parser::new(vec![vec![Token::Label("MAIN".into())],
                                          vec![Token::OpCode("LDA".into()),
                                               Token::Absolute("4400".into())]]);

        let result = parser.parse().unwrap();

        assert_eq!(&[Token::Label("MAIN".into())], &result[0][..]);
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("4400".into())],
                   &result[1][..]);
    }

    #[test]
    fn does_not_error_on_implied_addressing_mode() {
        let mut parser = Parser::new(vec![vec![Token::OpCode("NOP".into())],
                                          vec![Token::OpCode("BRK".into())],
                                          vec![Token::OpCode("CLC".into())]]);

        let result = parser.parse().unwrap();

        assert_eq!(&[Token::OpCode("NOP".into())], &result[0][..]);
        assert_eq!(&[Token::OpCode("BRK".into())], &result[1][..]);
        assert_eq!(&[Token::OpCode("CLC".into())], &result[2][..]);
    }

    #[test]
    fn does_error_on_implied_addressing_mode_opcodes_that_have_arguments() {
        let mut parser = Parser::new(vec![vec![Token::OpCode("NOP".into()),
                                               Token::Absolute("4400".into())]]);

        assert_eq!(Err(ParserError::invalid_opcode_addressing_mode_combination(1)),
                   parser.parse());
    }

    #[test]
    fn does_error_when_multiple_instructions_are_not_split_across_lines() {
        let mut parser = Parser::new(vec![vec![Token::OpCode("LDA".into()),
                                               Token::Absolute("4400".into()),
                                               Token::OpCode("LDA".into()),
                                               Token::Absolute("4400".into()),
                                            ]]);

        assert_eq!(Err(ParserError::expected_eol(1)), parser.parse());
    }
}