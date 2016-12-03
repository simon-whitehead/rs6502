use ::opcodes::AddressingMode;

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
    Indirect(String),
    IndirectX(String),
    IndirectY(String),
    Directive(String),
    Digits(String, ImmediateBase),
    Assignment,
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