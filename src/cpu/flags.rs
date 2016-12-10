
pub struct StatusFlags {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disabled: bool,
    pub decimal: bool,
    pub breakpoint: bool,
    pub unused: bool,
    pub overflow: bool,
    pub sign: bool,
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