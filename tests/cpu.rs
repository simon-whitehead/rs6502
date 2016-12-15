extern crate rs6502;

#[cfg(test)]
mod tests {
    mod cpu {

        use rs6502::*;

        #[test]
        fn can_instantiate_cpu() {
            let cpu = Cpu::new();

            assert!(0 == 0);
        }

        #[test]
        fn can_load_code_segment_into_memory() {
            let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
            let mut cpu = Cpu::new();
            cpu.load(&fake_code[..], None);

            let memory_sum: u32 = cpu.memory.iter().map(|n| *n as u32).sum();
            assert_eq!(46, memory_sum);
        }

        #[test]
        fn can_load_code_segment_at_default_address() {
            let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
            let mut cpu = Cpu::new();
            cpu.load(&fake_code[..], None);

            assert_eq!(0x0D, cpu.memory.read_byte(0xC003));
            assert_eq!(0x0C, cpu.memory.read_byte(0xC002));
            assert_eq!(0x0B, cpu.memory.read_byte(0xC001));
            assert_eq!(0x0A, cpu.memory.read_byte(0xC000));
        }

        #[test]
        fn can_load_code_segment_at_specific_address() {
            let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
            let mut cpu = Cpu::new();
            cpu.load(&fake_code[..], 0xF000);

            assert_eq!(0x0D, cpu.memory.read_byte(0xF003));
            assert_eq!(0x0C, cpu.memory.read_byte(0xF002));
            assert_eq!(0x0B, cpu.memory.read_byte(0xF001));
            assert_eq!(0x0A, cpu.memory.read_byte(0xF000));
        }

        #[test]
        fn errors_when_code_segment_extends_past_memory_bounds() {
            let fake_code = vec![0x0A, 0x0B, 0x0C, 0x0D];
            let mut cpu = Cpu::new();
            let load_result = cpu.load(&fake_code[..], 0xFFFD);

            assert_eq!(Err(CpuError::code_segment_out_of_range(0xFFFD)),
                       load_result);
        }

        #[test]
        fn errors_on_unknown_opcode() {
            let fake_code = vec![0xC3];
            let mut cpu = Cpu::new();
            cpu.load(&fake_code[..], None);
            let step_result: CpuStepResult = cpu.step();

            assert_eq!(Err(CpuError::unknown_opcode(0xC000, 0xC3)), step_result);// This is the unofficial DCP (d,X) opcode
        }

        #[test]
        fn can_get_operand_from_opcode() {
            let fake_code = vec![0xC3];
            let mut cpu = Cpu::new();
            cpu.load(&fake_code[..], None);
            let step_result: CpuStepResult = cpu.step();
        }

        #[test]
        fn adc_can_set_decimal_flag() {
            let code = vec![0xF8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step();

            assert_eq!(true, cpu.flags.decimal);
        }

        #[test]
        fn adc_can_disable_decimal_flag() {
            let code = vec![0xD8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step();

            assert_eq!(false, cpu.flags.decimal);
        }

        #[test]
        fn adc_can_add_basic_numbers() {
            let code = vec![0xA9, 0x05, 0x69, 0x03];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(8, cpu.registers.A);
        }

        #[test]
        fn adc_can_add_basic_numbers_set_carry_and_wrap_around() {
            let code = vec![0xA9, 0xFD, 0x69, 0x05];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(2, cpu.registers.A);
            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn adc_can_add_numbers_in_binary_coded_decimal() {
            let code = vec![0xF8, 0xA9, 0x05, 0x69, 0x05];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(3);

            assert_eq!(true, cpu.flags.decimal);
            assert_eq!(0x10, cpu.registers.A);
        }

        #[test]
        fn adc_can_add_numbers_in_binary_coded_decimal_and_set_carry() {
            let code = vec![0xF8, 0xA9, 0x95, 0x69, 0x10];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(3);

            assert_eq!(true, cpu.flags.carry);
            assert_eq!(true, cpu.flags.decimal);
            assert_eq!(0x05, cpu.registers.A);
        }

        #[test]
        fn sta_can_store_bytes_in_memory() {
            let code = vec![0xA9, 0x20, 0x8D, 0x00, 0x20];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x20, cpu.registers.A);
            assert_eq!(0x20, cpu.memory[0x2000]);
        }

        #[test]
        fn and_can_apply_logical_and_operation() {
            // Load 255 into A and mask it against 0x0F
            let code = vec![0xA9, 0xFF, 0x29, 0x0F];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x0F, cpu.registers.A);
            assert_eq!(false, cpu.flags.sign);
        }

        #[test]
        fn and_can_apply_logical_and_operation_and_set_sign_flag() {
            // Load 2 into the A register and shift it left
            let code = vec![0xA9, 0x02, 0x0A];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x04, cpu.registers.A);
            assert_eq!(false, cpu.flags.sign);
        }

        #[test]
        fn asl_can_shift_bits_left() {
            let code = vec![0xA9, 0x02, 0x0A];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x04, cpu.registers.A);
            assert_eq!(false, cpu.flags.sign);
        }

        #[test]
        fn asl_shifts_last_bit_into_carry() {
            let code = vec![0xA9, 0x80, 0x0A];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x00, cpu.registers.A);
            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn bcc_can_jump_forward() {
            let code = vec![0xA9, 0xFE, 0x69, 0x01, 0x90, 0x03, 0xA9, 0x00];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(3);

            assert_eq!(0xFF, cpu.registers.A);
            assert_eq!(false, cpu.flags.carry);
            assert_eq!(0xC009, cpu.registers.PC);
        }

        #[test]
        fn bcc_can_jump_backward() {
            let code = vec![0xA9, 0xF0, 0x69, 0x01, 0x90, 0xFC];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(50);

            assert_eq!(0x00, cpu.registers.A);
        }

        #[test]
        fn bcs_can_jump_forward() {
            let code = vec![0xA9, 0xFF, 0x69, 0x01, 0xB0, 0x03, 0xA9, 0xAA];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x00, cpu.registers.A);
            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn beq_can_jump_forward() {
            let code = vec![0xA9, 0xF0, 0x69, 0x10, 0xF0, 0x03, 0xA9, 0xAA];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x00, cpu.registers.A);
        }

        #[test]
        fn bit_can_set_flags_and_preserve_registers() {
            let code = vec![0xA9, 0xF0, 0x24, 0x00];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(true, cpu.flags.zero);
            assert_eq!(0xF0, cpu.registers.A);  // Preserves A
        }

        #[test]
        fn bit_can_set_overflow_flag() {
            let code = vec![0xA9, 0xF0, 0x85, 0x44, 0x24, 0x44];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(false, cpu.flags.zero);
            assert_eq!(true, cpu.flags.overflow);
            assert_eq!(true, cpu.flags.sign);
            assert_eq!(0xF0, cpu.registers.A);  // Preserves A
        }

        #[test]
        fn bmi_can_jump_forward() {
            let code = vec![0xA9, 0x7F, 0x69, 0x01, 0x30, 0x03, 0xA9, 0x00];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x80, cpu.registers.A);
            assert_eq!(true, cpu.flags.sign);
        }

        #[test]
        fn bne_jumps_on_non_zero() {
            let code = vec![0xA9, 0xFE, 0x69, 0x01, 0xD0, 0x03, 0xA9, 0xAA];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xFF, cpu.registers.A);
            assert_eq!(false, cpu.flags.zero);
        }

        #[test]
        fn bne_does_not_jump_on_zero() {
            let code = vec![0xA9, 0xFF, 0x69, 0x01, 0xD0, 0x03, 0xA9, 0xAA];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xAA, cpu.registers.A);
        }

        #[test]
        fn bpl_does_not_jump_on_sign_set() {
            let code = vec![0xA9, 0xFE, 0x10, 0x03, 0xA9, 0xF3];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xF3, cpu.registers.A);
            assert_eq!(true, cpu.flags.sign);
        }

        #[test]
        fn bpl_does_jump_on_sign_not_set() {
            let code = vec![0xA9, 0x0E, 0x10, 0x03, 0xA9, 0xF3];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x0E, cpu.registers.A);
            assert_eq!(false, cpu.flags.sign);
        }

        #[test]
        fn brk_does_store_pc_and_status_flags_on_stack() {
            let code = vec![0xA9, 0x0E, 0x00];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xC0, cpu.memory[0x1FE]);
            assert_eq!(0x03, cpu.memory[0x1FD]);
        }

        #[test]
        fn bvc_does_not_jump_on_overflow_set() {
            let code = vec![0xA9, 0x7F, 0x69, 0x01, 0x50, 0x03, 0xA9, 0xFF];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xFF, cpu.registers.A);
            assert_eq!(true, cpu.flags.overflow);
        }

        #[test]
        fn bvc_does_jump_on_overflow_clear() {
            let code = vec![0xA9, 0x7E, 0x69, 0x01, 0x50, 0x03, 0xA9, 0xFF];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x7F, cpu.registers.A);
            assert_eq!(false, cpu.flags.overflow);
        }

        #[test]
        fn bvs_does_not_jump_on_overflow_clear() {
            let code = vec![0xA9, 0x7E, 0x69, 0x01, 0x70, 0x03, 0xA9, 0xFF];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0xFF, cpu.registers.A);
            assert_eq!(false, cpu.flags.overflow);
        }

        #[test]
        fn bvs_does_jump_on_overflow_set() {
            let code = vec![0xA9, 0x7F, 0x69, 0x01, 0x70, 0x03, 0xA9, 0xFF];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x80, cpu.registers.A);
            assert_eq!(true, cpu.flags.overflow);
        }

        #[test]
        fn clc_clears_carry_flag() {
            let code = vec![0x18];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = true;

            cpu.step();

            assert_eq!(false, cpu.flags.carry);
        }

        #[test]
        fn cld_clears_decimal_flag() {
            let code = vec![0xD8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.decimal = true;

            cpu.step();

            assert_eq!(false, cpu.flags.decimal);
        }

        #[test]
        fn cli_clears_interrupt_flag() {
            let code = vec![0x58];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.interrupt_disabled = true;

            cpu.step();

            assert_eq!(false, cpu.flags.interrupt_disabled);
        }

        #[test]
        fn clv_clears_overflow_flag() {
            let code = vec![0xB8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.overflow = true;

            cpu.step();

            assert_eq!(false, cpu.flags.overflow);
        }

        #[test]
        fn cmp_sets_zero_flag() {
            let code = vec![0xA9, 0x55, 0xC9, 0x55];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.zero = false;

            cpu.step_n(2);

            assert_eq!(true, cpu.flags.zero);
        }

        #[test]
        fn cmp_clears_carry_flag() {
            let code = vec![0xA9, 0x55, 0xC9, 0x65];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = true;

            cpu.step_n(2);

            assert_eq!(false, cpu.flags.carry);
        }

        #[test]
        fn cmp_sets_carry_flag() {
            let code = vec![0xA9, 0x65, 0xC9, 0x55];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = false;

            cpu.step_n(2);

            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn cpx_clears_carry_flag() {
            let code = vec![0xA2, 0x55, 0xE0, 0x65];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = true;

            cpu.step_n(2);

            assert_eq!(false, cpu.flags.carry);
        }

        #[test]
        fn cpx_sets_carry_flag() {
            let code = vec![0xA2, 0x65, 0xE0, 0x55];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = false;

            cpu.step_n(2);

            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn cpy_clears_carry_flag() {
            let code = vec![0xA0, 0x55, 0xC0, 0x65];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = true;

            cpu.step_n(2);

            assert_eq!(false, cpu.flags.carry);
        }

        #[test]
        fn cpy_sets_carry_flag() {
            let code = vec![0xA0, 0x65, 0xC0, 0x55];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);
            cpu.flags.carry = false;

            cpu.step_n(2);

            assert_eq!(true, cpu.flags.carry);
        }

        #[test]
        fn dec_decrements() {
            let code = vec![0xA9, 0x55, 0x85, 0x85, 0xC6, 0x85];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x54, cpu.memory[0x85]);
        }

        #[test]
        fn dex_decrements() {
            let code = vec![0xA2, 0x55, 0xCA];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x54, cpu.registers.X);
        }

        #[test]
        fn dey_decrements() {
            let code = vec![0xA0, 0x55, 0x88];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x54, cpu.registers.Y);
        }

        #[test]
        fn eor_xors() {
            let code = vec![0xA9, 0x00, 0x49, 0x80];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x80, cpu.registers.A);
        }

        #[test]
        fn inc_increments() {
            let code = vec![0xA9, 0x55, 0x85, 0x85, 0xE6, 0x85];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x56, cpu.memory[0x85]);
        }

        #[test]
        fn inx_increments_x() {
            let code = vec![0xA2, 0x55, 0xE8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(10);

            assert_eq!(0x56, cpu.registers.X);
        }

        #[test]
        fn iny_increments_y() {
            let code = vec![0xA0, 0x55, 0xC8];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(20);

            assert_eq!(0x56, cpu.registers.Y);
        }

        #[test]
        fn jmp_jumps() {
            let code = vec![0xA9, 0x55, 0x4C, 0x07, 0x00, 0xA9, 0xFF];
            let mut cpu = Cpu::new();
            cpu.load(&code[..], None);

            cpu.step_n(2);

            assert_eq!(0x55, cpu.registers.A);
            assert_eq!(0xC007, cpu.registers.PC);
        }
    }
}