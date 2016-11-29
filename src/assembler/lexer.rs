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
                } else if c == '#' {
                    let token = Self::consume_immediate(&mut idx, line)?;
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

    fn consume_immediate(idx: &mut usize, line: &[u8]) -> Result<Token, LexerError> {
        let mut tok = String::new();
        // Very next character should be a dollar sign
        let c = line[*idx + 0x01] as char;
        let mut base = ImmediateBase::Base16;
        if c != '$' {
            let mut c = line[*idx + 0x02] as char;
            if c.is_digit(10) {
                *idx += 0x02;
                base = ImmediateBase::Base10;
                // Consume every number
                while !c.is_digit(10) {
                    c = line[*idx] as char;
                    tok.push(c);
                }

                return Ok(Token::Immediate(tok.clone(), base));
            } else {
                return Err(LexerError::unexpected_ident("{digit}", line[*idx + 0x01] as char));
            }
        }

        let c = line[*idx + 0x02] as char;
        if c.is_digit(16) {
            tok.push(c as char);
        } else {
            return Err(LexerError::unexpected_ident("{hex_digit}", c));
        }

        let c = line[*idx + 0x03] as char;
        if c.is_digit(16) {
            let val = &line[*idx + 0x02..*idx + 0x04];
            *idx += 0x04;
            return Ok(Token::Immediate(str::from_utf8(val).unwrap().into(), base));
        } else {
            return Err(LexerError::unexpected_ident("{hex_digit}", c));
        }

        Err("An error occurred while consuming an immediate identifier".into())
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
}
