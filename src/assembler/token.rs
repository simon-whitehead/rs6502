use opcodes::OpCode;

pub enum Token {
    Comment(&'static str),
    Label(&'static str),
    OpCode(OpCode),
    ArgumentList(Vec<&'static str>),
}