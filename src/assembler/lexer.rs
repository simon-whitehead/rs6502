// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It doesn't support the arithmetic operators, however. It looks like it matches the various 6502 assembly examples
// I have seen online and so is good enough.

use std;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::str;
use assembler::token::{ImmediateBase, LexerToken};
use ::opcodes::OpCode;

#[derive(Debug, PartialEq)]
pub struct LexerError {
    pub message: String,
}

impl LexerError {
    fn unexpected_ident<A, B>(expected: A, found: B, line: u32, column: u32) -> LexerError
        where A: std::fmt::Display,
              B: std::fmt::Display
    {
        LexerError {
            message: format!("Unexpected identifier. Found '{}', expected '{}'. Line {}, col {}",
                             found,
                             expected,
                             line,
                             column),
        }
    }
    fn out_of_bounds<A>(addr: A, line: u32, column: u32) -> LexerError
        where A: std::fmt::Display
    {
        LexerError::from(format!("Memory address '{}' too large. Line {}, col {}",
                                 addr,
                                 line,
                                 column))
    }

    fn error_consuming_number(line: u32, column: u32) -> LexerError {
        LexerError::from(format!("Error consuming number. Line {} col {}", line, column))
    }

    fn unexpected_eof() -> LexerError {
        LexerError::from(format!("Unexpected end of file"))
    }

    fn expected_memory_address(line: u32, column: u32) -> LexerError {
        LexerError::from(format!("Expected memory address. Line {} col {}", line, column))
    }

    fn unexpected_token(line: u32, column: u32) -> LexerError {
        LexerError::from(format!("Unexpected token. Line {} col {}", line, column))
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
pub struct Lexer {
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer { line: 0, col: 0 }
    }

    /// Returns a vector of Tokens given an input of
    /// 6502 assembly code
    pub fn lex_string<S>(&mut self, input: S) -> Result<Vec<Vec<LexerToken>>, LexerError>
        where S: Into<String>
    {
        Ok(self.lex(input.into())?)
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(&mut self, path: P) -> Result<Vec<Vec<LexerToken>>, LexerError>
        where P: AsRef<std::path::Path>
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(self.lex(contents)?)
    }

    fn advance<I>(&mut self, mut peeker: &mut Peekable<I>)
        where I: Iterator<Item = char>
    {
        if let None = peeker.peek() {
            return;
        }

        peeker.next();
        self.col += 1;
    }

    /// Performs the bulk of the lexing logic
    fn lex(&mut self, source: String) -> Result<Vec<Vec<LexerToken>>, LexerError> {

        let mut result = Vec::new();

        for line in source.lines() {
            self.line += 1;
            self.col = 0;

            // Skip blank lines
            if line.trim().len() == 0 {
                continue;
            }

            let mut tokens = Vec::new();
            let mut iter = line.chars();
            let mut peeker = iter.peekable();

            loop {
                // Break out if we've reached the end of the line
                if let None = peeker.peek() {
                    break;
                }

                // Consume any leading whitespace voids we're sitting in
                if peeker.peek().unwrap().is_whitespace() {
                    self.consume_whitespace(&mut peeker);
                } else if peeker.peek().unwrap().is_alphanumeric() {
                    let token = self.consume_alphanumeric(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == ';' {
                    // Skip the rest of this line
                    break;
                } else if *peeker.peek().unwrap() == '(' {
                    // Indirect addressing
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::OpenParenthesis);
                } else if *peeker.peek().unwrap() == ')' {
                    // Indirect addressing
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::CloseParenthesis);
                } else if *peeker.peek().unwrap() == '$' {
                    let token = self.consume_address(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == '#' {
                    if let LexerToken::Immediate(number, base) = self.consume_number(&mut peeker)? {
                        tokens.push(LexerToken::Immediate(number, base));
                    }
                } else if *peeker.peek().unwrap() == '.' {
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::Period);
                } else if *peeker.peek().unwrap() == ':' {
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::Colon);
                } else if *peeker.peek().unwrap() == '=' {
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::Assignment);
                } else if *peeker.peek().unwrap() == ',' {
                    self.advance(&mut peeker);
                    tokens.push(LexerToken::Comma);
                } else {
                    return Err(LexerError::unexpected_token(self.line, self.col + 1));
                }
            }

            result.push(tokens);
        }

        Ok(result)
    }

    /// Consumes alphanumeric characters until it reachs something that terminates it
    fn consume_alphanumeric<I>(&mut self,
                               mut peeker: &mut Peekable<I>)
                               -> Result<LexerToken, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();

        loop {
            if let None = peeker.peek() {
                break;
            }
            let c = *peeker.peek().unwrap();

            if c.is_alphanumeric() || c == '_' {
                tok.push(c);
                self.advance(&mut peeker);
            } else {
                break;
            }
        }

        Ok(LexerToken::Ident(tok))
    }

    /// Decides the base of a number we are about to consume
    fn consume_number<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<LexerToken, LexerError>
        where I: Iterator<Item = char>
    {
        // Default to base16
        let mut base = ImmediateBase::Base16;

        let c = *peeker.peek().unwrap();
        if c == '$' {
            // The number is base16
            self.advance(&mut peeker);
            self.consume_digits(&mut peeker, &base)
        } else if c == '#' {
            // The number is base 10
            self.advance(&mut peeker);
            if let None = peeker.peek() {
                return Err(LexerError::unexpected_eof());
            }

            base = ImmediateBase::Base10;
            if *peeker.peek().unwrap() == '$' {
                // Skip over the dollar sign and revert to base16
                base = ImmediateBase::Base16;
                self.advance(&mut peeker);
            }

            self.consume_digits(&mut peeker, &base)
        } else {
            Err(LexerError::error_consuming_number(self.line, self.col))
        }
    }

    /// Consumes number of a specified base until it can't anymore
    fn consume_digits<I>(&mut self,
                         mut peeker: &mut Peekable<I>,
                         base: &ImmediateBase)
                         -> Result<LexerToken, LexerError>
        where I: Iterator<Item = char>
    {
        let mut result = String::new();

        let b = if let ImmediateBase::Base10 = *base {
            10
        } else {
            16
        };
        loop {
            if let None = peeker.peek() {
                break;
            }
            let c = *peeker.peek().unwrap();
            if c.is_digit(b) {
                result.push(c);
                self.advance(&mut peeker);
            } else {
                break;
            }
        }

        Ok(LexerToken::Immediate(result.to_uppercase(), base.clone()))
    }

    /// Consumes a memory address
    fn consume_address<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<LexerToken, LexerError>
        where I: Iterator<Item = char>
    {
        // Grab the actual numbers
        if let LexerToken::Immediate(val, _) = self.consume_number(&mut peeker)? {
            let val = val.to_uppercase();
            // if the length is greater than 4.. its outside the memory bounds
            if val.len() > 4 {
                return Err(LexerError::out_of_bounds(&val, self.line, self.col - val.len() as u32));
            }

            Ok(LexerToken::Address(val.clone()))

        } else {
            Err(LexerError::expected_memory_address(self.line, self.col))
        }
    }

    /// Consumes whitespace characters until it encounters a
    /// non-whitespace character
    #[inline(always)]
    fn consume_whitespace<I>(&mut self, mut peeker: &mut Peekable<I>)
        where I: Iterator<Item = char>
    {
        loop {
            if let None = peeker.peek() {
                break;
            } else {
                if !peeker.peek().unwrap().is_whitespace() {
                    break;
                } else {
                    self.advance(&mut peeker);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, LexerToken};

    #[test]
    fn can_lex_basic_opcode_and_addressing_mode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA $4400
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("LDA".into()), LexerToken::Address("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_lex_variable_assignment() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            MY_VARIABLE = #$20
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("MY_VARIABLE".into()),
                     LexerToken::Assignment,
                     LexerToken::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_lex_base_ten_variable_assignment() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            MY_VARIABLE = #50
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("MY_VARIABLE".into()),
                     LexerToken::Assignment,
                     LexerToken::Immediate("50".into(), ImmediateBase::Base10)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_lex_absolute_addressing() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA $4400,X
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("LDA".into()),
                     LexerToken::Address("4400".into()),
                     LexerToken::Comma,
                     LexerToken::Ident("X".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_lex_indirect_y_addressing() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA ($FF),Y
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("LDA".into()),
                     LexerToken::OpenParenthesis,
                     LexerToken::Address("FF".into()),
                     LexerToken::CloseParenthesis,
                     LexerToken::Comma,
                     LexerToken::Ident("Y".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_lex_indirect_x_addressing() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA ($FF,X)
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("LDA".into()),
                     LexerToken::OpenParenthesis,
                     LexerToken::Address("FF".into()),
                     LexerToken::Comma,
                     LexerToken::Ident("X".into()),
                     LexerToken::CloseParenthesis],
                   &tokens[0][..]);
    }

    #[test]
    fn errors_on_unexpected_token() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA ($F-----F,X)
        ");

        assert_eq!(Err(LexerError::unexpected_token(2, 20)), tokens);
    }

    #[test]
    fn errors_on_unexpected_token_square_bracket() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA ($FF],X)
        ");

        assert_eq!(Err(LexerError::unexpected_token(2, 21)), tokens);
    }

    #[test]
    fn can_handle_lots_of_whitespace() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA (    $FF      ,   X                             )
        ")
            .unwrap();

        assert_eq!(&[LexerToken::Ident("LDA".into()),
                     LexerToken::OpenParenthesis,
                     LexerToken::Address("FF".into()),
                     LexerToken::Comma,
                     LexerToken::Ident("X".into()),
                     LexerToken::CloseParenthesis],
                   &tokens[0][..]);
    }
}