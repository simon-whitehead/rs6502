
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

impl StatusFlags {
    pub fn to_u8(&self) -> u8 {
        let carry = if self.carry { 0x01 } else { 0 };
        let zero = if self.zero { 0x02 } else { 0 };
        let interrupt_disabled = if self.interrupt_disabled { 0x04 } else { 0 };
        let decimal = if self.decimal { 0x08 } else { 0 };
        let breakpoint = if self.breakpoint { 0x10 } else { 0 };
        let overflow = if self.overflow { 0x40 } else { 0 };
        let sign = if self.sign { 0x80 } else { 0 };

        carry | zero | interrupt_disabled | decimal | breakpoint | overflow | sign
    }
}

impl From<u8> for StatusFlags {
    fn from(byte: u8) -> StatusFlags {
        StatusFlags {
            carry: byte & 0x01 == 0x01,
            zero: byte & 0x02 == 0x02,
            interrupt_disabled: byte & 0x04 == 0x04,
            decimal: byte & 0x08 == 0x08,
            breakpoint: byte & 0x10 == 0x10,
            unused: false,
            overflow: byte & 0x40 == 0x40,
            sign: byte & 0x80 == 0x80,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_four() {
        let f = StatusFlags::default();

        assert_eq!(0x04, f.to_u8());
    }

    #[test]
    fn can_convert_to_u8() {
        let mut f = StatusFlags::default();

        f.carry = true;

        assert_eq!(0x05, f.to_u8());
    }

    #[test]
    fn can_convert_to_and_from() {
        let f = StatusFlags {
            carry: true,
            decimal: true,
            sign: true,
            overflow: true,
            interrupt_disabled: false,
            ..Default::default()
        };

        let byte = f.to_u8();
        let result: StatusFlags = byte.into();

        assert_eq!(true, result.carry);
        assert_eq!(true, result.decimal);
        assert_eq!(true, result.sign);
        assert_eq!(true, result.overflow);

        assert_eq!(false, result.interrupt_disabled);
        assert_eq!(false, result.zero);
        assert_eq!(false, result.breakpoint);
        assert_eq!(false, result.unused);
    }
}