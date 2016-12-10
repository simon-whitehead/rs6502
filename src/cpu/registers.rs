
#[allow(non_snake_case)]
pub struct Registers {
    pub A: u8,
    pub X: u8,
    pub Y: u8,
    pub S: u8,
    pub PC: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            PC: 0,
        }
    }
}
