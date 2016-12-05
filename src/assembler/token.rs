use ::opcodes::{AddressingMode, OpCode};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ImmediateBase {
    Base10,
    Base16,
}

#[derive(Clone, Debug, PartialEq )]
pub enum LexerToken {
    Ident(String),
    Assignment,
    Address(String),
    OpenParenthesis,
    CloseParenthesis,
    Comma,
    Period,
    Immediate(String, ImmediateBase),
    Colon,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ParserToken {
    Label(String),
    OpCode(OpCode),
    RawByte(u8),
    Directive(String),
}

impl Token {
    pub fn to_addressing_mode(&self) -> AddressingMode {
        match *self {
            Token::Immediate(_, _) => AddressingMode::Immediate,
            Token::ZeroPage(_) => AddressingMode::ZeroPage,
            Token::ZeroPageX(_) => AddressingMode::ZeroPageX,
            Token::Absolute(_) => AddressingMode::Absolute,
            Token::AbsoluteX(_) => AddressingMode::AbsoluteX,
            Token::AbsoluteY(_) => AddressingMode::AbsoluteY,
            Token::Indirect(_) => AddressingMode::Indirect,
            Token::IndirectX(_) => AddressingMode::IndirectX,
            Token::IndirectY(_) => AddressingMode::IndirectY,
            _ => AddressingMode::Implied,
        }
    }
}