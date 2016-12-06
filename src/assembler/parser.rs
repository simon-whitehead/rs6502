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
    tokens: Vec<Vec<LexerToken>>,
    line: u32,
}

/// Parser processes a list of 6502 Assembly tokens
impl Parser {
    pub fn new(tokens: Vec<Vec<LexerToken>>) -> Parser {
        Parser {
            tokens: tokens,
            line: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ParserToken>, ParserError> {
        let mut result = Vec::new();

        for line in &self.tokens {
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

                } else {
                    // Skip the ident and we'll check what is next
                    peeker.next();
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
                        if !Self::is_opcode(ident.clone()) {
                            return Err(ParserError::expected_instruction(self.line));
                        }
                    }

                    let next = *peeker.peek().unwrap();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, LexerToken, ParserToken};
    use ::opcodes::{AddressingMode, OpCode};

    #[test]
    fn can_detect_labels_via_lonely_label() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into())],
                          vec![LexerToken::Ident("START".into())]];

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into()), ParserToken::Label("START".into())],
                   &result[..]);
    }

    #[test]
    fn can_detect_labels_via_colon_terminator() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into())], vec![LexerToken::Colon]];

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(&[ParserToken::Label("MAIN".into())], &result[..]);
    }

    #[test]
    fn can_detect_double_labels_on_one_line() {
        let tokens = vec![vec![LexerToken::Ident("MAIN".into()),
                               LexerToken::Ident("START".into())]];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert_eq!(Err(ParserError::expected_instruction(1)), result);
    }
}