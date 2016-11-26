extern crate byteorder;

mod assembler;
mod cpu;
mod disassembler;
mod opcodes;
mod registers;

pub use cpu::Cpu;
pub use disassembler::Disassembler;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
