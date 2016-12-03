use std::path::Path;

use assembler::lexer::{Lexer, LexerError};
use assembler::parser::{Parser, ParserError};

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

pub struct Assembler;

impl Assembler {
    pub fn assemble_string<S>(code: S) -> Result<Vec<u8>, AssemblerError>
        where S: Into<String>
    {
        let code = code.into();
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string(code)?;
        let mut parser = Parser::new(tokens);
        let tokens = parser.parse()?;

        Ok(Vec::new())
    }

    pub fn assemble_file<P>(path: P) -> Result<Vec<u8>, AssemblerError>
        where P: AsRef<Path>
    {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_file(path)?;
        let mut parser = Parser::new(tokens);
        let tokens = parser.parse()?;

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
