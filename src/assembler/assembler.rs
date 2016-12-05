use std::collections::HashMap;
use std::path::Path;

use ::opcodes::{AddressingMode, OpCode};
use assembler::lexer::{Lexer, LexerError};
use assembler::parser::{Parser, ParserError};
use assembler::token::LexerToken;

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Label(u16), // Label + its byte offset
    Constant(LexerToken), // The constant value
}

pub struct AssemblerError {
    message: String,
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

pub struct Assembler {
    symbol_table: HashMap<String, Symbol>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler { symbol_table: HashMap::new() }
    }

    pub fn assemble_string<S>(&mut self, code: S) -> Result<Vec<u8>, AssemblerError>
        where S: Into<String>
    {
        let code = code.into();
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string(code)?;
        let mut parser = Parser::new(tokens);
        let tokens = Vec::new(); // TODO: Fix.

        Ok(self.assemble(tokens))
    }

    pub fn assemble_file<P>(&mut self, path: P) -> Result<Vec<u8>, AssemblerError>
        where P: AsRef<Path>
    {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_file(path)?;
        let mut parser = Parser::new(tokens);
        let tokens = Vec::new(); // TODO: Fix

        Ok(self.assemble(tokens))
    }

    fn assemble(&mut self, tokens: Vec<Vec<LexerToken>>) -> Vec<u8> {
        // First, index the labels so we have addresses for them
        Vec::new()
    }
}