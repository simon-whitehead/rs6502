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
    LabelArg(String),
    OpCode(OpCode),
    Absolute(String),
    RawByte(u8),
    Directive(String),
}
