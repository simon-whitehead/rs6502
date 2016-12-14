use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, PartialEq)]
pub struct StackError {
    message: String,
}

impl StackError {
    pub fn overflow() -> StackError {
        StackError { message: "Stack overflow occurred".into() }
    }

    pub fn underflow() -> StackError {
        StackError { message: "Stack underflow; unable to pop from empty stack".into() }
    }
}

pub type StackPushResult = Result<(), StackError>;
pub type StackPopResult<T> = Result<T, StackError>;

pub struct Stack {
    pointer: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { pointer: 0xFF }
    }

    pub fn push(&mut self, stack_area: &mut [u8], val: u8) -> StackPushResult {
        if self.pointer > 0x00 {
            self.pointer -= 0x01;
            stack_area[self.pointer] = val;

            Ok(())
        } else {
            Err(StackError::overflow())
        }
    }

    pub fn push_u16(&mut self, stack_area: &mut [u8], val: u16) -> StackPushResult {
        if self.pointer > 0x01 {
            LittleEndian::write_u16(&mut stack_area[self.pointer - 0x02..], val);
            self.pointer -= 0x02;

            Ok(())
        } else {
            Err(StackError::overflow())
        }
    }

    pub fn pop(&mut self, stack_area: &[u8]) -> StackPopResult<u8> {
        if self.pointer == 0xFF {
            Err(StackError::underflow())
        } else {
            let val = stack_area[self.pointer];
            self.pointer += 0x01;

            Ok(val)
        }
    }

    pub fn pop_u16(&mut self, stack_area: &mut [u8]) -> StackPopResult<u16> {
        if self.pointer < 0xFE {
            let result = LittleEndian::read_u16(&stack_area[self.pointer..]);
            self.pointer += 0x02;

            Ok(result)
        } else {
            Err(StackError::underflow())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_push() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        stack.push(&mut stack_area, 55);

        assert_eq!(55, stack_area[0xFE]);
    }

    #[test]
    fn can_push_then_pop() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        stack.push(&mut stack_area, 55);
        let val = stack.pop(&mut stack_area).unwrap();

        assert_eq!(55, val);
    }

    #[test]
    fn can_push_then_pop_multiple() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        stack.push(&mut stack_area, 5);
        stack.push(&mut stack_area, 10);
        stack.push(&mut stack_area, 15);
        stack.push(&mut stack_area, 20);

        let twenty = stack.pop(&mut stack_area).unwrap();
        let fifteen = stack.pop(&mut stack_area).unwrap();
        let ten = stack.pop(&mut stack_area).unwrap();
        let five = stack.pop(&mut stack_area).unwrap();

        assert_eq!(20, twenty);
        assert_eq!(15, fifteen);
        assert_eq!(10, ten);
        assert_eq!(5, five);
    }

    #[test]
    fn can_not_pop_empty_stack() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        let result = stack.pop(&mut stack_area);

        assert_eq!(Err(StackError::underflow()), result);
    }

    #[test]
    fn can_not_push_to_full_stack() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        for _ in 0..0xFF {
            stack.push(&mut stack_area, 5);
        }

        let result = stack.push(&mut stack_area, 5);

        assert_eq!(Err(StackError::overflow()), result);
    }

    #[test]
    fn can_push_u16() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        stack.push_u16(&mut stack_area, 0x4400);

        assert_eq!(0x44, stack_area[0xFE]);
        assert_eq!(0x00, stack_area[0xFD]);
    }

    #[test]
    fn can_push_then_pop_u16() {
        let mut stack_area = [0u8; 0xFF];
        let mut stack = Stack::new();

        stack.push_u16(&mut stack_area, 0x4400);
        let result = stack.pop_u16(&mut stack_area).unwrap();

        assert_eq!(0x4400, result);
    }
}