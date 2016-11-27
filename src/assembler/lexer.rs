// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It looks like it matches the various 6502 assembly examples I have seen online.

use std;
use std::fs::File;
use std::io::{self, Read};

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
            let mut tok = String::new();
            let mut tokens = Vec::new();
            for c in line.chars() {
                if c != ';' && c != ':' && !c.is_whitespace() {
                    tok.push(c);
                } else {
                    // If the token is not empty, lets classify it and
                    // put it in the tokens Vector
                }
            }

            result.append(&mut tokens);
        }

        result
    }

    fn create_token(input: &String) -> Token {
        if let Some(opcode) = OpCode::from_mnemonic(&input[..]) {
            Token::OpCode(opcode.mnemonic.into())
        } else {
            Token::Unknown(input.clone())
        }
    }
}