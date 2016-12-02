use assembler::token::Token;

#[derive(Debug, PartialEq)]
pub struct ParserError {
    message: String,
}

impl ParserError {
    fn expected_instruction(line: u32) -> ParserError {
        ParserError::from(format!("Instruction expected. Line {}", line))
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
}

/// Parser processes a list of 6502 Assembly tokens
impl Parser {
    pub fn new(tokens: Vec<Vec<Token>>) -> Parser {
        Parser { tokens: tokens }
    }

    /// Processes its tokens and either returns them to the caller
    /// or produces an error
    pub fn parse(&self) -> Result<Vec<Vec<Token>>, ParserError> {
        Ok(self.tokens.iter().map(|v| v.clone()).collect())
    }
}