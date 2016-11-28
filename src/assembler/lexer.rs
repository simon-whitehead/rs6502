// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It looks like it matches the various 6502 assembly examples I have seen online.

use std;
use std::fs::File;
use std::io::{self, Read};
use std::iter::Peekable;

use assembler::token::Token;
use ::opcodes::OpCode;

/// Lexer accepts the program code as a string
/// and converts it to a list of Tokens
pub struct Lexer;

impl Lexer {
    /// Returns a vector of Tokens given an input of
    /// 6502 assembly code
    pub fn lex_string<S>(input: S) -> Vec<Vec<Token>>
        where S: Into<String>
    {
        Self::lex(input.into())
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(path: P) -> Result<Vec<Vec<Token>>, io::Error>
        where P: AsRef<std::path::Path>
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(Self::lex(contents))
    }

    /// Performs the bulk of the lexing logic
    fn lex(source: String) -> Vec<Vec<Token>> {

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
                    tokens.push(Self::consume_alphanumeric(&mut idx, line));
                } else {
                    idx += 1;
                }
            }

            result.push(tokens);
        }

        result
    }

    fn consume_alphanumeric(idx: &mut usize, line: &[u8]) -> Token {
        let mut tok = String::new();

        while *idx < line.len() {
            let c = line[*idx] as char;
            if c.is_whitespace() || c == ';' || c == ':' || c == '\n' {
                return Self::classify(&tok);
            } else {
                tok.push(c);
            }

            *idx += 1;
        }

        Token::Unknown(tok)
    }

    fn classify(input: &str) -> Token {
        if let Some(opcode) = OpCode::from_mnemonic(input) {
            Token::OpCode(opcode.mnemonic.into())
        } else {
            Token::Unknown(input.into())
        }
    }

    fn create_token(input: &String) -> Token {
        if let Some(opcode) = OpCode::from_mnemonic(&input[..]) {
            Token::OpCode(opcode.mnemonic.into())
        } else {

            Token::Unknown(input.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::Token;

    #[test]
    fn can_classify_simple_labels_and_opcodes() {
        let tokens = Lexer::lex_string("
            LDA #$20 
        ");

        assert_eq!(&[Token::OpCode("LDA".into()), Token::Unknown("#$20".into())], &tokens[0][..]);
    }
}
