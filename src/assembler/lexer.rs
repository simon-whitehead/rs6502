// This lexer is based on the grammar I found here: https://github.com/antlr/grammars-v4/blob/master/asm6502/asm6502.g4
// It doesn't support the arithmetic operators, however. It looks like it matches the various 6502 assembly examples
// I have seen online and so is good enough.

use std;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::str;
use assembler::token::{ImmediateBase, Token};
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

    fn unexpected_eof() -> LexerError {
        LexerError::from("Unexpected EOF")
    }

    fn expected_indirect_address(line: u32, column: u32) -> LexerError {
        LexerError::from(format!("Expected indirect addressing. Line {} col {}", line, column))
    }

    fn expected_memory_address(line: u32, column: u32) -> LexerError {
        LexerError::from(format!("Expected memory address. Line {} col {}", line, column))
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
    pub fn lex_string<S>(&mut self, input: S) -> Result<Vec<Vec<Token>>, LexerError>
        where S: Into<String>
    {
        Ok(self.lex(input.into())?)
    }

    /// Returns a vector of Tokens given a file
    /// to load 6502 assembly code from
    pub fn lex_file<P>(&mut self, path: P) -> Result<Vec<Vec<Token>>, LexerError>
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
    fn lex(&mut self, source: String) -> Result<Vec<Vec<Token>>, LexerError> {

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
                    let token = self.consume_indirect(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == '$' {
                    let token = self.consume_address(&mut peeker)?;
                    tokens.push(token);
                } else if *peeker.peek().unwrap() == '#' {
                    if let Token::Digits(number, base) = self.consume_number(&mut peeker)? {
                        tokens.push(Token::Immediate(number, base));
                    }
                } else if *peeker.peek().unwrap() == '.' {
                    self.advance(&mut peeker);
                    let token = self.consume_alphanumeric(&mut peeker)?;
                    // "Label"s immediately following a dot are a Directive
                    // in this assembler
                    if let Token::Label(label) = token {
                        tokens.push(Token::Directive(label));
                    }
                } else if *peeker.peek().unwrap() == '=' {
                    self.advance(&mut peeker);
                    tokens.push(Token::Assignment);
                }
            }

            result.push(tokens);
        }

        Ok(result)
    }

    /// Consumes alphanumeric characters until it reachs something that terminates it
    fn consume_alphanumeric<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        let mut tok = String::new();

        loop {
            if let None = peeker.peek() {
                break;
            }
            let c = *peeker.peek().unwrap();
            // Break on possible label endings
            if c == ':' {
                self.advance(&mut peeker);
                break;
            }

            let c = *peeker.peek().unwrap(); // Re-peek just in case the label ending was consumed
            if c.is_whitespace() || c == ';' || c == '\n' {
                break;
            } else {
                tok.push(c);
                self.advance(&mut peeker);
            }
        }

        self.consume_whitespace(&mut peeker);

        Ok(self.classify(&tok.to_uppercase()))
    }

    /// Decides the base of a number we are about to consume
    fn consume_number<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        // Default to base16
        let mut base = ImmediateBase::Base16;

        if let None = peeker.peek() {
            Err(LexerError::unexpected_eof())
        } else {
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
                Err("Error consuming number".into())
            }
        }
    }

    /// Consumes number of a specified base until it can't anymore
    fn consume_digits<I>(&mut self,
                         mut peeker: &mut Peekable<I>,
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
            if let None = peeker.peek() {
                break;
            }
            let c = *peeker.peek().unwrap();
            if c.is_digit(b) {
                result.push(c);
            } else {
                if c == ',' || c == ')' || c.is_whitespace() {
                    break;
                } else {
                    if b == 10 {
                        return Err(LexerError::unexpected_ident("{digit}",
                                                                c,
                                                                self.line,
                                                                self.col + 0x01));
                    } else {
                        return Err(LexerError::unexpected_ident("{hex_digit}",
                                                                c,
                                                                self.line,
                                                                self.col + 0x01));
                    }
                }
            }
            self.advance(&mut peeker);
        }

        self.consume_whitespace(&mut peeker);

        Ok(Token::Digits(result.to_uppercase(), base.clone()))
    }

    /// Consumes a memory address
    fn consume_address<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        // Grab the actual numbers
        if let Token::Digits(val, _) = self.consume_number(&mut peeker)? {
            let val = val.to_uppercase();
            // if the length is greater than 4.. its outside the memory bounds
            if val.len() > 4 {
                return Err(LexerError::out_of_bounds(&val, self.line, self.col - val.len() as u32));
            }

            // If the length is greater than 2.. its not a Zero Page address
            if val.len() > 2 {
                let mut token_type = Token::Absolute(val.clone());
                // Check for AbsoluteX
                if let Some(_) = peeker.peek() {
                    if *peeker.peek().unwrap() == ',' {
                        self.advance(&mut peeker);  // Jump over the comma
                        self.consume_whitespace(&mut peeker);
                        if let None = peeker.peek() {
                            return Err(LexerError::unexpected_eof());
                        }

                        if *peeker.peek().unwrap() == 'X' {
                            self.advance(&mut peeker);
                            token_type = Token::AbsoluteX(val.clone());
                        } else if *peeker.peek().unwrap() == 'Y' {
                            self.advance(&mut peeker);
                            token_type = Token::AbsoluteY(val.clone());
                        }
                    }
                }

                self.consume_whitespace(&mut peeker);

                Ok(token_type)
            } else {
                // Its zero page
                let mut token_type = Token::ZeroPage(val.clone());
                // Check for ZeroPageX:
                if let Some(_) = peeker.peek() {
                    if *peeker.peek().unwrap() == ',' {
                        self.advance(&mut peeker);
                        self.consume_whitespace(&mut peeker);

                        if *peeker.peek().unwrap() == 'X' {
                            token_type = Token::ZeroPageX(val.clone());
                            self.advance(&mut peeker);

                            self.consume_whitespace(&mut peeker);
                            self.advance(&mut peeker);
                            self.consume_whitespace(&mut peeker);
                        }
                    }
                }

                self.consume_whitespace(&mut peeker);

                Ok(token_type)
            }
        } else {
            Err(LexerError::expected_memory_address(self.line, self.col))
        }
    }

    /// Consumes an indirect memory addressing instruction
    fn consume_indirect<I>(&mut self, mut peeker: &mut Peekable<I>) -> Result<Token, LexerError>
        where I: Iterator<Item = char>
    {
        self.advance(&mut peeker);; // Jump the opening parenthesis
        self.consume_whitespace(&mut peeker);

        let addr = self.consume_address(&mut peeker)?;

        if let Token::ZeroPageX(val) = addr {
            let val = val.to_uppercase();

            // Its IndirectX
            self.consume_whitespace(&mut peeker);

            return Ok(Token::IndirectX(val.clone()));
        } else {
            if *peeker.peek().unwrap() == ')' {
                // High chance its IndirectY - lets check:
                self.advance(&mut peeker);
                self.consume_whitespace(&mut peeker);

                if *peeker.peek().unwrap() == ',' {
                    self.advance(&mut peeker); // Skip the comma
                    self.consume_whitespace(&mut peeker);

                    if *peeker.peek().unwrap() == 'Y' {
                        if let Token::ZeroPage(val) = addr {
                            let val = val.to_uppercase();

                            self.advance(&mut peeker);
                            self.consume_whitespace(&mut peeker);

                            return Ok(Token::IndirectY(val.clone()));
                        }
                    }
                } else {
                    return Err(LexerError::expected_indirect_address(self.line, self.col));
                }
            }
        }
        Err(LexerError::expected_indirect_address(self.line, self.col))
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

    /// Classifies an alphanumeric token into either an op code
    /// or a label
    fn classify(&mut self, input: &str) -> Token {
        let tok = String::from(input);
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
    fn can_classify_clearly_wrong_stuff() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LOL this is totally broken
        ")
            .unwrap();

        assert_eq!(&[Token::Label("LOL".into()),
                     Token::Label("THIS".into()),
                     Token::Label("IS".into()),
                     Token::Label("TOTALLY".into()),
                     Token::Label("BROKEN".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_simple_labels_and_opcodes() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            LDA #$20 
        ")
            .unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn does_not_classify_unknown_labels_and_opcodes() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
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
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("MAIN: LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn can_classify_labels_with_no_colon() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("MAIN LDA #$20").unwrap();

        assert_eq!(&[Token::Label("MAIN".into()),
                     Token::OpCode("LDA".into()),
                     Token::Immediate("20".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid() {
        let mut lexer = Lexer::new();
        assert_eq!(lexer.lex_string("
            LDA #$INVALID20
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "I", 2, 19)));
    }

    #[test]
    fn immediate_values_throw_errors_when_invalid_hex_code() {
        let mut lexer = Lexer::new();
        assert_eq!(lexer.lex_string("
            LDA #$2Z
        "),
                   Err(LexerError::unexpected_ident("{hex_digit}", "Z", 2, 20)));
    }

    #[test]
    fn immediate_values_throw_errors_when_not_even_a_decimal_number() {
        let mut lexer = Lexer::new();
        assert_eq!(lexer.lex_string("
            LDA #@hotmail.com
        "),
                   Err(LexerError::unexpected_ident("{digit}", "@", 2, 18)));
    }

    #[test]
    fn immediate_values_accept_base_ten() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA #10").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("10".into(), ImmediateBase::Base10)],
                   &tokens[0][..]);
    }

    #[test]
    fn immediate_values_base_ten_does_not_accept_hex() {
        let mut lexer = Lexer::new();
        assert_eq!(lexer.lex_string("LDA #1A"),
                   Err(LexerError::unexpected_ident("{digit}", "A", 1, 7)));
    }

    #[test]
    fn can_figure_out_zero_page_opcode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $44").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPage("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_zero_page_x_opcode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $44,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::ZeroPageX("44".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_address_opcode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $4400").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_address_opcode_with_all_hex_digits() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $AFFF").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("AFFF".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_x_address_opcode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $4400,X").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteX("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $4400,Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_figure_out_absolute_y_address_opcode_when_excess_whitespace() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA $4400,        Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn does_skip_comments() {
        let mut lexer = Lexer::new();
        let tokens =
            lexer.lex_string("LDA $4400, Y ; This loads the value at $4400 + Y into the A register")
                .unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::AbsoluteY("4400".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_x_register() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA ($20,X)").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectX("20".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_indirect_addressing_y_register() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("LDA ($20),      Y").unwrap();
        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectY("20".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn out_of_bounds_memory_addresses_throw_errors() {
        let mut lexer = Lexer::new();
        assert_eq!(lexer.lex_string("LDA $FFFF0"),
                   Err(LexerError::out_of_bounds("FFFF0", 1, 5)));
    }

    #[test]
    fn can_tokenize_clearmem_implementation() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
CLRMEM  LDA #$00        ; Load 0 into the Accumulator
        TAY             ; Transfer A -> Y (load 0 into Y as well)
CLRM1   STA ($FF),Y     ; Store the value of A (0) into $FF+Y 
        INY             ; Increment Y
        DEX             ; Decrement X
        BNE CLRM1       ; Jump back if X != 0
        RTS             ; Return from the subroutine
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
        assert_eq!(&[Token::OpCode("BNE".into()), Token::Label("CLRM1".into())],
                   &tokens[5][..]);

        assert_eq!(&[Token::OpCode("RTS".into())], &tokens[6][..]);
    }

    #[test]
    fn can_accept_assignments() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            SQUARE_X = $100
            SQUARE_Y = $101
        ")
            .unwrap();

        assert_eq!(&[Token::Label("SQUARE_X".into()),
                     Token::Assignment,
                     Token::Absolute("100".into())],
                   &tokens[0][..]);
        assert_eq!(&[Token::Label("SQUARE_Y".into()),
                     Token::Assignment,
                     Token::Absolute("101".into())],
                   &tokens[1][..]);
    }

    #[test]
    fn can_handle_lots_of_whitespace() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("   LDA     (   $FF        )    ,          Y").unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectY("FF".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_lots_of_whitespace_for_indirect_x() {
        let mut lexer = Lexer::new();
        let tokens =
            lexer.lex_string("   LDA     (   $FF            ,          X                )   ")
                .unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()), Token::IndirectX("FF".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_lots_of_whitespace_for_absolute() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("   LDA        $FF00      ").unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()), Token::Absolute("FF00".into())],
                   &tokens[0][..]);
    }

    #[test]
    fn can_handle_lowercase_code() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            lda #$00
            tax
            inx
        ")
            .unwrap();

        assert_eq!(&[Token::OpCode("LDA".into()),
                     Token::Immediate("00".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);
        assert_eq!(&[Token::OpCode("TAX".into())], &tokens[1][..]);
        assert_eq!(&[Token::OpCode("INX".into())], &tokens[2][..]);
    }

    #[test]
    fn can_handle_variable_assignment_and_argument_use() {
        let mut lexer = Lexer::new();
        let tokens = lexer.lex_string("
            VARIABLE = #$44
            LDA VARIABLE
        ")
            .unwrap();

        assert_eq!(&[Token::Label("VARIABLE".into()),
                     Token::Assignment,
                     Token::Immediate("44".into(), ImmediateBase::Base16)],
                   &tokens[0][..]);

        assert_eq!(&[Token::OpCode("LDA".into()), Token::Label("VARIABLE".into())],
                   &tokens[1][..]);
    }
}
