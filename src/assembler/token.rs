use opcodes::OpCode;

#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    Unknown(String),
    Comment(String),
    Label(String),
    OpCode(String),
    ArgumentList(Vec<String>),
}