use std::collections::HashMap;
use std::path::Path;

use ::opcodes::{AddressingMode, OpCode};
use assembler::lexer::{Lexer, LexerError};
use assembler::parser::{Parser, ParserError};
use assembler::token::Token;

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Label(u16), // Label + its byte offset
    Constant(Token), // The constant value
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
        let tokens = parser.parse()?;

        Ok(self.assemble(tokens))
    }

    pub fn assemble_file<P>(&mut self, path: P) -> Result<Vec<u8>, AssemblerError>
        where P: AsRef<Path>
    {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_file(path)?;
        let mut parser = Parser::new(tokens);
        let tokens = parser.parse()?;

        Ok(self.assemble(tokens))
    }

    fn assemble(&mut self, tokens: Vec<Vec<Token>>) -> Vec<u8> {
        // First, index the labels so we have addresses for them
        self.index_labels(&tokens);
        Vec::new()
    }

    /// Iterates over the tokens, counting the total byte counts as it
    /// goes, indexing the labels
    fn index_labels(&mut self, tokens: &[Vec<Token>]) {
        let mut addr: u16 = 0;
        for line in tokens {
            let mut peeker = line.iter().peekable();
            for token in line {
                peeker.next();
                if let &Token::OpCode(ref mnemonic) = token {
                    // Determine an addressing mode that was attempted,
                    let addressing_mode = if let None = peeker.peek() {
                        // if there was no token, then its implied
                        AddressingMode::Implied
                    } else {
                        // Otherwise, lets try and convert
                        // the next token to a mode
                        let next = *peeker.peek().unwrap();
                        next.to_addressing_mode()
                    };

                    if let Some(opcode) =
                           OpCode::from_mnemonic_and_addressing_mode(mnemonic.clone(),
                                                                     addressing_mode) {
                        addr += opcode.length as u16;
                    }
                }

                if let &Token::Label(ref name) = token {
                    // Check if the next thing is an assignment:
                    let next = *peeker.peek().unwrap();
                    if let &Token::Assignment = next {
                        peeker.next();
                        let ref val = *peeker.next().unwrap();
                        // Storethe token we're assigning
                        self.symbol_table.insert(name.clone(), Symbol::Constant(val.clone()));
                        peeker.next();
                    } else {
                        self.symbol_table.insert(name.clone(), Symbol::Label(addr));
                    }
                }
            }
        }
        for (key, ref value) in self.symbol_table.iter() {
            println!("{:?} -> {:?}", key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assembler::token::Token;

    #[test]
    fn can_index_labels() {
        let mut assembler = Assembler::new();
        assembler.assemble_string("
            VARIABLE = $4400
            MAIN LDA $4400
            LOOP INX
        ");

        assert_eq!(&Symbol::Constant(Token::Absolute("4400".into())),
                   assembler.symbol_table.get("VARIABLE").unwrap());
    }
}
