
pub struct StatusFlags {
    carry: bool,
    zero: bool,
    interrupt_disabled: bool,
    decimal: bool,
    breakpoint: bool,
    unused: bool,
    overflow: bool,
    sign: bool,
}

impl Default for StatusFlags {
    fn default() -> StatusFlags {
        StatusFlags {
            carry: false,
            zero: false,
            interrupt_disabled: true,
            decimal: false,
            breakpoint: false,
            unused: false,
            overflow: false,
            sign: false,
        }
    }
}

#[allow(non_snake_case)]
pub struct Registers {
    A: u8,
    X: u8,
    Y: u8,
    S: u8,
    PC: u16,
    P: StatusFlags,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            PC: 0,
            P: Default::default(),
        }
    }
}
