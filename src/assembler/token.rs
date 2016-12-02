#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ImmediateBase {
    Base10,
    Base16,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Token {
    Unknown(String),
    Label(String),
    OpCode(String),
    Immediate(String, ImmediateBase),
    ZeroPage(String),
    ZeroPageX(String),
    Absolute(String),
    AbsoluteX(String),
    AbsoluteY(String),
    IndirectX(String),
    IndirectY(String),
    Directive(String),
    Digits(String, ImmediateBase),
    Assignment,
}