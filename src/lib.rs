extern crate byteorder;

mod cpu;
mod disassembler;
mod opcodes;
mod registers;

pub use cpu::Cpu;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
