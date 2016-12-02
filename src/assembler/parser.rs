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
        let mut line_number = 0;

        for line in &self.tokens {
            line_number += 1;
            let mut peeker = line.iter().peekable();
            // Check what starts the line
            let token = peeker.peek().unwrap().clone();

            match *token {
                Token::Label(_) => {
                    // if its a label, consume it and move on
                    peeker.next();
                    if let None = peeker.peek() {
                        return Err(ParserError::expected_instruction(line_number));
                    }
                    let next = *peeker.peek().unwrap();
                    if let &Token::OpCode(_) = next {
                    } else {
                        return Err(ParserError::expected_instruction(line_number));
                    }
                }
                _ => (),
            }
        }
        Ok(self.tokens.iter().map(|v| v.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, Token};

    #[test]
    fn errors_on_multiple_labels() {
        let parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                           Token::Label("METHOD".into()),
                                           Token::OpCode("LDA".into()),
                                           Token::Immediate("10".into(), ImmediateBase::Base16)]]);

        assert_eq!(Err(ParserError::expected_instruction(1)), parser.parse());
    }

    #[test]
    fn does_not_error_on_single_label() {
        let parser = Parser::new(vec![vec![Token::Label("MAIN".into()),
                                           Token::OpCode("LDA".into()),
                                           Token::Immediate("10".into(), ImmediateBase::Base16)]]);

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("10".into(), ImmediateBase::Base16)],
                   &parser.parse().unwrap()[0][..]);
    }
}