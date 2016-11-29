// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It looks like it matches the various 6502 assembly examples I have seen online.

use std;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::str;

use assembler::token::{ImmediateBase, Token};
use ::opcodes::OpCode;

#[derive(Debug, PartialEq)]
pub struct LexerError {
    pub message: String,
}

impl LexerError {
    fn unexpected_ident<A, B>(expected: A, found: B) -> LexerError
        where A: std::fmt::Display,
              B: std::fmt::Display
    {
        LexerError {
            message: format!("ERR: Unexpected identifier. Found '{}', expected '{}'",
                             found,
                             expected),
        }
    }
}

impl From<std::io::Error> for LexerError {
    fn from(error: std::io::Error) -> LexerError {
        LexerError { message: error.description().into() }
    }
}

impl From<String> for LexerError {
    fn from(error: String) -> LexerError {
        LexerError { message: error }
    }
}

impl<'a> From<&'a str> for LexerError {
    fn from(error: &str) -> LexerError {
        LexerError { message: error.into() }
    }
}

/// Lexer accepts the program code as a string
/// and converts it to a list of Tokens
pub struct Lexer;

impl Lexer {
    /// Returns a vector of Tokens given an input of
    /// 6502 assembly code
    pub fn lex_string<S>(input: S) -> Result<Vec<Vec<Token>>, LexerError>
        where S: Into<String>
    {
        Ok(Self::lex(input.into())?)
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(path: P) -> Result<Vec<Vec<Token>>, LexerError>
        where P: AsRef<std::path::Path>
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(Self::lex(contents)?)
    }

    /// Performs the bulk of the lexing logic
    fn lex(source: String) -> Result<Vec<Vec<Token>>, LexerError> {

        let mut result = Vec::new();

        for line in source.lines() {
            if line.trim().len() == 0 {
                continue;
            }
            let mut tokens = Vec::new();
            let mut idx = 0;
            let line = &line[..].as_bytes();
            while idx < line.len() {
                let c = line[idx] as char;
                if c.is_alphanumeric() {
                    let token = Self::consume_alphanumeric(&mut idx, line)?;
                    tokens.push(token);
                } else if c == ';' {
                    // Skip the rest of this line
                    break;
                } else if c == '(' {
                    // Indirect addressing
                    let token = Self::consume_indirect(&mut idx, line)?;
                    tokens.push(token);
                } else if c == '$' {
                    let token = Self::consume_address(&mut idx, line)?;
                    tokens.push(token);
                } else if c == '#' {
                    let token = Self::consume_number(&mut idx, line)?;
                    tokens.push(token);
                } else if c == '.' {
                    idx += 1;
                    let token = Self::consume_alphanumeric(&mut idx, line)?;
                    if let Token::Label(label) = token {
                        tokens.push(Token::Directive(label));
                    }
                } else {
                    idx += 1;
                }
            }

            result.push(tokens);
        }

        Ok(result)
    }

    fn consume_alphanumeric(idx: &mut usize, line: &[u8]) -> Result<Token, LexerError> {
        let mut tok = String::new();

        while *idx < line.len() {
            let c = line[*idx] as char;
            if c.is_whitespace() || c == ';' || c == ':' || c == '\n' {
                return Ok(Self::classify(&tok));
            } else {
                tok.push(c);
            }

            *idx += 1;
        }

        Err("ERR: An error occurred while consuming an alphanumeric identifier".into())
    }

    fn consume_number(idx: &mut usize, line: &[u8]) -> Result<Token, LexerError> {
        let mut tok = String::new();
        *idx += 1;
        let mut c = line[*idx] as char;
        let mut base = ImmediateBase::Base16;

        // Very next character should be a dollar sign
        if c != '$' {
            if c.is_digit(10) {
                base = ImmediateBase::Base10;
                // Consume every number
                loop {
                    if c.is_digit(10) {
                        tok.push(c);
                        *idx += 1;
                        if *idx < line.len() {
                            c = line[*idx] as char;
                        } else {
                            break;
                        }
                    } else {
                        if c == ',' || c == ')' {
                            break;
                        } else {
                            return Err(LexerError::unexpected_ident("{digit}", c));
                        }
                    }
                }

                return Ok(Token::Immediate(tok.clone(), base));
            } else {
                return Err(LexerError::unexpected_ident("{digit}", c));
            }
        }

        *idx += 1;
        let c = line[*idx] as char;
        if c.is_digit(16) {
            tok.push(c as char);
        } else {
            return Err(LexerError::unexpected_ident("{hex_digit}", c));
        }

        *idx += 1;
        let c = line[*idx] as char;
        if c.is_digit(16) {
            tok.push(c);
            *idx += 0x02;
            return Ok(Token::Immediate(str::from_utf8(&tok[..].as_bytes()).unwrap().into(), base));
        } else {
            return Err(LexerError::unexpected_ident("{hex_digit}", c));
        }

        Err("An error occurred while consuming an immediate identifier".into())
    }

    fn consume_address(mut idx: &mut usize, line: &[u8]) -> Result<Token, LexerError> {
        let mut tok = String::new();
        if let Token::Immediate(val, _) = Self::consume_number(&mut idx, &line)? {
            // If the length is greater than 2.. its not a Zero Page address
            if val.len() > 2 {
                let mut token_type = Token::Absolute(val.clone());
                // Check for AbsoluteX
                if *idx + 0x01 < line.len() && line[*idx] as char == ',' {
                    *idx += 1;  // Jump over the comma
                    Self::consume_whitespace(&mut idx, line);
                    let c = line[*idx] as char;
                    if c == 'X' {
                        token_type = Token::AbsoluteX(val.clone());
                    } else if c == 'Y' {
                        token_type = Token::AbsoluteY(val.clone());
                    }
                    *idx += 0x02;
                }

                Ok(token_type)
            } else {
                let mut token_type = Token::ZeroPage(val.clone());

                // Check for ZeroPageX:
                if *idx + 0x01 < line.len() && line[*idx] as char == ',' &&
                   line[*idx + 0x01] as char == 'X' {
                    token_type = Token::ZeroPageX(val.clone());
                    *idx += 0x02;
                }

                Ok(token_type)
            }
        } else {
            Err(LexerError::from("Error consuming address"))
        }
    }

    fn consume_indirect(mut idx: &mut usize, line: &[u8]) -> Result<Token, LexerError> {
        let mut tok = String::new();
        *idx += 1; // jump the opening parenthesis
        let addr = Self::consume_address(&mut idx, &line)?;

        if let Token::ZeroPageX(val) = addr {
            // Its IndirectX
            return Ok(Token::IndirectX(val.clone()));
        } else {
            let c = line[*idx] as char;
            if c == ')' {
                // High chance its IndirectY - lets check:
                *idx += 1;
                let c = line[*idx] as char;
                if c == ',' {
                    *idx += 1; // Skip the comma
                    Self::consume_whitespace(&mut idx, &line);
                    let c = line[*idx] as char;
                    if c == 'Y' {
                        if let Token::ZeroPage(val) = addr {
                            *idx += 1;
                            return Ok(Token::IndirectY(val.clone()));
                        }
                    }
                }
            }
        }
        Err(LexerError::from("ERR: Error while parsing Indirect address"))
    }

    fn consume_whitespace(mut idx: &mut usize, line: &[u8]) {
        loop {
            if *idx < line.len() {
                if (line[*idx] as char).is_whitespace() {
                    *idx += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn classify(input: &str) -> Token {
        let mut tok = String::from(input);
        if let Some(opcode) = OpCode::from_mnemonic(tok.clone()) {
            Token::OpCode(tok.clone())
        } else {
            Token::Label(input.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, Token};

    #[test]
    fn can_classify_simple_labels_and_opcodes() {
        let tokens = Lexer::lex_string("
            LDA #$20 
        ")
            .unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn does_not_classify_unknown_labels_and_opcodes() {
        let tokens = Lexer::lex_string("
            .LOL LDA #$20 
        ")
            .unwrap();

        assert_eq!(&[Token::Directive("LOL".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_labels() {
        let tokens = Lexer::lex_string("MAIN: LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_labels_with_no_colon() {
        let tokens = Lexer::lex_string("MAIN LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid() {
        assert_eq!(Lexer::lex_string("
            LDA #$INVALID20
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "I")));
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid_hex_code() {
        assert_eq!(Lexer::lex_string("
            LDA #$2Z
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "Z")));
    }

    #[test]
    fn immediate_values_throw_errors_when_not_even_a_hex_code() {
        assert_eq!(Lexer::lex_string("
            LDA #@hotmail.com
        "),
                   Err(LexerError::unexpected_ident("{digit}", "@")));
    }

    #[test]
    fn immediate_values_accept_base_ten() {
        let tokens = Lexer::lex_string("LDA #10").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("10".into(), ImmediateBase::Base10)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_base_ten_does_not_accept_hex() {
        assert_eq!(Lexer::lex_string("LDA #1A"),
                   Err(LexerError::unexpected_ident("{digit}", "A")));
    }

    #[test]
    fn can_figure_out_zero_page_opcode() {
        let tokens = Lexer::lex_string("LDA $44").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPage("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_zero_page_x_opcode() {
        let tokens = Lexer::lex_string("LDA $44,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPageX("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_x_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteX("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400,Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode_when_excess_whitespace() {
        let tokens = Lexer::lex_string("LDA $4400,        Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn does_skip_comments() {
        let tokens = Lexer::lex_string("LDA $4400, Y ; This loads the value at $4400 + Y into \
                                        the A register")
            .unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_x_register() {
        let tokens = Lexer::lex_string("LDA ($20,X)").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectX("20".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_y_register() {
        let tokens = Lexer::lex_string("LDA ($20),      Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectY("20".into())],
                   &tokens[0][..]);
    }
}
