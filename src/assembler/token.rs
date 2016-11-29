use opcodes::OpCode;

#[derive(Eq, PartialEq, Debug)]
pub enum ImmediateBase {
    Base10,
    Base16,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    Unknown(String),
    Comment(String),
    Label(String),
    OpCode(String),
    Immediate(String, ImmediateBase),
    Directive(String),
    ArgumentList(Vec<String>),
}