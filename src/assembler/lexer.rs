// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It looks like it matches the various 6502 assembly examples I have seen online.

use std;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::iter::Peekable;
use std::str;
use std::str::Chars;

use assembler::token::{ImmediateBase, Token};
use ::opcodes::OpCode;

#[derive(Debug, PartialEq)]
pub struct LexerError {
    pub message: String,
}

impl LexerError {
    fn unexpected_ident<A, B>(expected: A, found: B) -> LexerError
        where A: std::fmt::Display,
              B: std::fmt::Display
    {
        LexerError {
            message: format!("ERR: Unexpected identifier. Found '{}', expected '{}'",
                             found,
                             expected),
        }
    }
    fn out_of_bounds<A>(addr: A) -> LexerError
        where A: std::fmt::Display
    {
        LexerError { message: format!("ERR: Memory address '{}' too large", addr) }
    }

    fn unexpected_eof() -> LexerError {
        LexerError { message: "Unexpected EOF".into() }
    }

    fn expected_indirect_address() -> LexerError {
        LexerError { message: "Expected indirect addressing".into() }
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
pub struct Lexer;

impl Lexer {
    /// Returns a vector of Tokens given an input of
    /// 6502 assembly code
    pub fn lex_string<S>(input: S) -> Result<Vec<Vec<Token>>, LexerError>
        where S: Into<String>
    {
        Ok(Self::lex(input.into())?)
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(path: P) -> Result<Vec<Vec<Token>>, LexerError>
        where P: AsRef<std::path::Path>
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(Self::lex(contents)?)
    }

    /// Performs the bulk of the lexing logic
    fn lex(source: String) -> Result<Vec<Vec<Token>>, LexerError> {

        let mut result = Vec::new();

        for line in source.lines() {
            if line.trim().len() == 0 {
                continue;
            }
            let mut tokens = Vec::new();
            let mut idx = 0;
            let mut iter = line.chars();
            let mut peeker = iter.peekable();
            loop {
                if let None = peeker.peek() {
                    break;
                }

                if peeker.peek().unwrap().is_whitespace() {
                    Self::consume_whitespace(&mut peeker);
                } else if peeker.peek().unwrap().is_alphanumeric() {
                    let token = Self::consume_alphanumeric(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == ';' {
                    // Skip the rest of this line
                    break;
                } else if *peeker.peek().unwrap() == '(' {
                    // Indirect addressing
                    let token = Self::consume_indirect(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == '$' {
                    let token = Self::consume_address(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == '#' {
                    if let Token::Digits(number, base) = Self::consume_number(&mut peeker)? {
                        tokens.push(Token::Immediate(number, base));
                    }
                } else if *peeker.peek().unwrap() == '.' {
                    peeker.next();
                    let token = Self::consume_alphanumeric(&mut peeker)?;
                    if let Token::Label(label) = token {
                        tokens.push(Token::Directive(label));
                    }
                }
            }

            result.push(tokens);
        }

        Ok(result)
    }

    fn consume_alphanumeric<I>(mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();

        loop {
            {
                if let None = peeker.peek() {
                    break;
                }
                if *peeker.peek().unwrap() == ':' {
                    peeker.next();
                    break;
                }
                let c = peeker.peek().unwrap();
                if c.is_whitespace() || *c == ';' || *c == '\n' {
                    break;
                } else {
                    tok.push(*c);
                }
            }
            peeker.next();
        }

        println!("Classifying: {}", tok);

        Ok(Self::classify(&tok))
    }

    fn consume_number<I>(mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();

        let mut base = ImmediateBase::Base16;

        if let None = peeker.peek() {
            Err(LexerError::unexpected_eof())
        } else {
            if *peeker.peek().unwrap() == '$' {
                peeker.next();
                Self::consume_digits(&mut peeker, &base)
            } else if *peeker.peek().unwrap() == '#' {
                if let None = peeker.peek() {
                    return Err(LexerError::unexpected_eof());
                }

                peeker.next();
                base = ImmediateBase::Base10;
                if *peeker.peek().unwrap() == '$' {
                    // Skip over the dollar sign and revert to base16
                    base = ImmediateBase::Base16;
                    peeker.next();
                }

                Self::consume_digits(&mut peeker, &base)
            } else {
                Err("Error consuming number".into())
            }
        }
    }

    fn consume_digits<I>(peeker: &mut Peekable<I>,
                         base: &ImmediateBase)
                         -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut result = String::new();

        let b = if let ImmediateBase::Base10 = *base {
            10
        } else {
            16
        };
        loop {
            if let Some(c) = peeker.peek() {
                if c.is_digit(b) {
                    result.push(*c);
                } else {
                    if *c == ',' || *c == ')' || c.is_whitespace() {
                        break;
                    } else {
                        if b == 10 {
                            return Err(LexerError::unexpected_ident("{digit}", c));
                        } else {
                            return Err(LexerError::unexpected_ident("{hex_digit}", c));
                        }
                    }
                }
            } else {
                break;
            }
            peeker.next();
        }

        Ok(Token::Digits(result, base.clone()))
    }

    fn consume_address<I>(mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();
        if let Token::Digits(val, base) = Self::consume_number(&mut peeker)? {
            // if the length is greater than 4.. its outside the memory bounds
            if val.len() > 4 {
                return Err(LexerError::out_of_bounds(val));
            }
            // If the length is greater than 2.. its not a Zero Page address
            if val.len() > 2 {
                let mut token_type = Token::Absolute(val.clone());
                // Check for AbsoluteX
                if let Some(_) = peeker.peek() {
                    if *peeker.peek().unwrap() == ',' {
                        peeker.next();  // Jump over the comma
                        Self::consume_whitespace(&mut peeker);
                        if let None = peeker.peek() {
                            return Err(LexerError::unexpected_eof());
                        }

                        if *peeker.peek().unwrap() == 'X' {
                            peeker.next();
                            token_type = Token::AbsoluteX(val.clone());
                        } else if *peeker.peek().unwrap() == 'Y' {
                            peeker.next();
                            token_type = Token::AbsoluteY(val.clone());
                        }
                    }
                }

                Ok(token_type)
            } else {
                let mut token_type = Token::ZeroPage(val.clone());
                // Check for ZeroPageX:
                if let Some(_) = peeker.peek() {
                    if *peeker.peek().unwrap() == ',' {
                        peeker.next();
                        if *peeker.peek().unwrap() == 'X' {
                            token_type = Token::ZeroPageX(val.clone());
                            peeker.next();
                            peeker.next();
                        }
                    }
                }

                Ok(token_type)
            }
        } else {
            Err(LexerError::from("Error consuming address"))
        }
    }

    fn consume_indirect<I>(mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();
        peeker.next(); // Jump the opening parenthesis
        let addr = Self::consume_address(&mut peeker)?;

        if let Token::ZeroPageX(val) = addr {
            // Its IndirectX
            println!("IndirectX");
            return Ok(Token::IndirectX(val.clone()));
        } else {
            if *peeker.peek().unwrap() == ')' {
                // High chance its IndirectY - lets check:
                peeker.next();
                if *peeker.peek().unwrap() == ',' {
                    peeker.next(); // Skip the comma
                    Self::consume_whitespace(&mut peeker);
                    if *peeker.peek().unwrap() == 'Y' {
                        if let Token::ZeroPage(val) = addr {
                            peeker.next();
                            return Ok(Token::IndirectY(val.clone()));
                        }
                    }
                } else {
                    return Err(LexerError::expected_indirect_address());
                }
            }
        }
        Err(LexerError::from("ERR: Error while parsing Indirect address"))
    }

    fn consume_whitespace<I>(peeker: &mut Peekable<I>)
        where I: Iterator<Item = char>
    {
        loop {
            if let None = peeker.peek() {
                break;
            } else {
                if !peeker.peek().unwrap().is_whitespace() {
                    break;
                } else {
                    peeker.next();
                }
            }
        }
    }

    fn classify(input: &str) -> Token {
        let mut tok = String::from(input);
        if let Some(opcode) = OpCode::from_mnemonic(tok.clone()) {
            Token::OpCode(tok.clone())
        } else {
            Token::Label(input.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::assembler::token::{ImmediateBase, Token};

    #[test]
    fn can_classify_simple_labels_and_opcodes() {
        let tokens = Lexer::lex_string("
            LDA #$20 
        ")
            .unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn does_not_classify_unknown_labels_and_opcodes() {
        let tokens = Lexer::lex_string("
            .LOL LDA #$20 
        ")
            .unwrap();

        assert_eq!(&[Token::Directive("LOL".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_labels() {
        let tokens = Lexer::lex_string("MAIN: LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_labels_with_no_colon() {
        let tokens = Lexer::lex_string("MAIN LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid() {
        assert_eq!(Lexer::lex_string("
            LDA #$INVALID20
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "I")));
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid_hex_code() {
        assert_eq!(Lexer::lex_string("
            LDA #$2Z
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "Z")));
    }

    #[test]
    fn immediate_values_throw_errors_when_not_even_a_decimal_number() {
        assert_eq!(Lexer::lex_string("
            LDA #@hotmail.com
        "),
                   Err(LexerError::unexpected_ident("{digit}", "@")));
    }

    #[test]
    fn immediate_values_accept_base_ten() {
        let tokens = Lexer::lex_string("LDA #10").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("10".into(), ImmediateBase::Base10)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_base_ten_does_not_accept_hex() {
        assert_eq!(Lexer::lex_string("LDA #1A"),
                   Err(LexerError::unexpected_ident("{digit}", "A")));
    }

    #[test]
    fn can_figure_out_zero_page_opcode() {
        let tokens = Lexer::lex_string("LDA $44").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPage("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_zero_page_x_opcode() {
        let tokens = Lexer::lex_string("LDA $44,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPageX("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_address_opcode_with_all_hex_digits() {
        let tokens = Lexer::lex_string("LDA $AFFF").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("AFFF".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_x_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteX("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode() {
        let tokens = Lexer::lex_string("LDA $4400,Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode_when_excess_whitespace() {
        let tokens = Lexer::lex_string("LDA $4400,        Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn does_skip_comments() {
        let tokens = Lexer::lex_string("LDA $4400, Y ; This loads the value at $4400 + Y into \
                                        the A register")
            .unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_x_register() {
        let tokens = Lexer::lex_string("LDA ($20,X)").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectX("20".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_y_register() {
        let tokens = Lexer::lex_string("LDA ($20),      Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectY("20".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn out_of_bounds_memory_addresses_throw_errors() {
        assert_eq!(Lexer::lex_string("LDA $FFFF0"),
                   Err(LexerError::out_of_bounds("FFFF0")));
    }

    #[test]
    fn can_tokenize_clearmem_implementation() {
        let tokens = Lexer::lex_string("
CLRMEM  LDA #$00
        TAY             
CLRM1   STA ($FF),Y
        INY            
        DEX             
BNE OTHERAGAIN
        RTS 
        ")
            .unwrap();

        assert_eq!(&[Token::Label("CLRMEM".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("00".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);

        assert_eq!(&[Token::OpCode("TAY".into())], &tokens[1][..]);

        assert_eq!(&[Token::Label("CLRM1".into()),
                     Token::OpCode("STA".into()),
                     Token::IndirectY("FF".into())],
                   &tokens[2][..]);

        assert_eq!(&[Token::OpCode("INY".into())], &tokens[3][..]);
        assert_eq!(&[Token::OpCode("DEX".into())], &tokens[4][..]);
        assert_eq!(&[Token::OpCode("BNE".into()), Token::Label("OTHERAGAIN".into())],
                   &tokens[5][..]);

        assert_eq!(&[Token::OpCode("RTS".into())], &tokens[6][..]);
    }
}
