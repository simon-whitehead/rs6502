use opcodes::OpCode;

pub enum Token {
    Unknown(String),
    Comment(String),
    Label(String),
    OpCode(String),
    ArgumentList(Vec<String>),
}