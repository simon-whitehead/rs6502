// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It looks like it matches the various 6502 assembly examples I have seen online.

use std;
use std::fs::File;
use std::io::{self, Read};

use assembler::token::Token;

/// Lexer accepts the program code as a string
/// and converts it to a list of Tokens
pub struct Lexer;

impl Lexer {
    /// Returns a vector of Tokens given an input of
    /// 6502 assembly code
    pub fn lex<S>(input: S) -> Vec<Token>
        where S: Into<String>
    {
        Self::lex(input.into())
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(path: P) -> Result<Vec<Token>, io::Error>
        where P: AsRef<std::path::Path>
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(Self::lex(contents))
    }

    /// Performs the bulk of the lexing logic
    fn lex(source: String) -> Vec<Token> {

        Vec::new()
    }
}