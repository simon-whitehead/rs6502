use assembler::token::Token;

#[derive(Debug, PartialEq)]
pub struct ParserError {
    message: String,
    line: u32,
    col: u32
}

impl ParserError {
}

pub struct Parser {
    tokens: Vec<Vec<Token>>
}

/// Parser processes a list of 6502 Assembly tokens
impl Parser {
    pub fn new(tokens: Vec<Vec<Token>>) -> Parser {
        Parser { tokens: tokens }
    }

/// Processes its tokens and either returns them to the caller
/// or produces an error
    pub fn parse() -> Result<Vec<Vec<Token>>, ParserError> {
    }
}