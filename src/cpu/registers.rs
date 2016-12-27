
#[allow(non_snake_case)]
pub struct Registers {
    pub A: u8,
    pub X: u8,
    pub Y: u8,
    pub PC: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers { ..Default::default() }
    }
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            A: 0,
            X: 0,
            Y: 0,
            PC: 0,
        }
    }
}
