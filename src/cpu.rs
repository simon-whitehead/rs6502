use registers::Registers;

pub struct Cpu {
    registers: Registers,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { registers: Registers::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_instantiate_cpu() {
        let cpu = Cpu::new();
        assert!(0 == 0);
    }
}