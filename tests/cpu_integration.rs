
extern crate rs6502;

#[test]
fn INTEGRATION_CPU_can_add_basic_numbers_in_accumulator() {
    let asm = "
        LDA #$20
        ADC #$10    ; A register should equal 48
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(2);

    assert_eq!(0x30, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_add_binary_coded_decimal_numbers_in_accumulator() {
    let asm = "
        SED
        LDA #$20
        ADC #$05    ; A register should equal 0x25
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(3);

    assert_eq!(0x25, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_add_mixed_mode_numbers_in_accumulator() {
    let asm = "
        LDA #$20
        ADC #10    ; A register should equal 0x2A
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(2);

    assert_eq!(0x2A, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_store_bytes_in_memory() {
    let asm = "
        LDA #$20
        STA $2000
        LDA #10
        STA $2001
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(4);

    assert_eq!(0x20, cpu.memory[0x2000]);
    assert_eq!(0x0A, cpu.memory[0x2001]);
    assert_eq!(0x00, cpu.memory[0x2002]);
}

#[test]
fn INTEGRATION_CPU_can_overwrite_own_memory() {
    let asm = "
        LDA #$20
        STA $C006
        LDA #10
        STA $2000
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(4);

    assert_eq!(0x20, cpu.memory[0x2000]);
}

#[test]
fn INTEGRATION_CPU_can_load_byte_into_memory_and_logical_AND_it_with_A_register() {
    let asm = "
        LDA #$0F
        STA $2000   ; Load the mask 0x0F into $2000
        LDA #$FF    ; Load 0xFF into A
        AND $2000   ; AND it with 0x0F
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(4);

    assert_eq!(0x0F, cpu.memory[0x2000]);
}

#[test]
fn INTEGRATION_CPU_can_load_byte_into_memory_and_logical_AND_it_with_A_register_using_a_variable
    () {
    let asm = "
        MEMORY_LOCATION = $2000

        LDA #$0F
        STA MEMORY_LOCATION     ; Load the mask 0x0F into $2000
        LDA #$FF                ; Load 0xFF into A
        AND MEMORY_LOCATION     ; AND it with 0x0F
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(4);

    assert_eq!(0x0F, cpu.memory[0x2000]);
}

#[test]
fn INTEGRATION_CPU_does_not_branch_on_clear_carry_flag() {
    let asm = "
        LDA #$FE
        ADC #1      ; This won't cause a carry
        BCC FINISH
        LDA #$00    ; Clear the A register
    FINISH:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(3);

    assert_eq!(0xFF, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_branch_on_carry_flag() {
    let asm = "
        LDA #$FE
        ADC #10     ; This will cause a carry
        BCC FINISH
        LDA #$00    ; Clear the A register
    FINISH:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(4);

    assert_eq!(0x00, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_branch_on_carry_flag_to_correct_offset() {
    let asm = "
        LDA #$FE
        ADC #1      ; This will not cause a carry, and execution
        BCC FINISH  ; should jump to the FINISH label
        LDA #$00
        LDA #$01
        LDA #$02
        LDA #$03
        LDA #$04
    FINISH:
        LDA #$AA
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(5);

    assert_eq!(0xAA, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_loop_on_bcc() {
    let asm = "
        LDA #$F0
    ADDER:
        ADC #1
        BCC ADDER
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0xFF, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_branch_on_bcs() {
    let asm = "
        LDA #$FE
        ADC #$05    ; This will carry
        BCS FINISH
        LDA #$00
    FINISH:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0x03, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_can_branch_on_beq() {
    let asm = "
        LDA #$FF
        ADC #$01    ; This will result in a zero result
        BEQ FINISH
        LDA #$FF
    FINISH:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0x00, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_does_not_branch_on_beq() {
    let asm = "
        LDA #$F0
        ADC #$01
        BEQ FINISH
        LDA #$FF    ; The branch above should not be taken
    FINISH:         ; and this should load 0xFF into A
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0xFF, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_preserves_flags_on_bit() {
    let asm = "
        LDA #$0F
        STA $44
        LDA #$F0
        BIT $44
        BEQ FINISH
        LDA #$35    ; The branch above will be taken
    FINISH:         ; because 0x0F & 0xF0 will be 0x00
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0xF0, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_bmi_branches_on_sign_bit_set() {
    let asm = "
        LDA #$7F
        ADC #1
        BMI FINISH
        LDA #$00    ; The branch above will be taken
    FINISH:         ; because the sign flag is set
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(30);

    assert_eq!(0x80, cpu.registers.A);
    assert_eq!(true, cpu.flags.sign);
}

#[test]
fn INTEGRATION_CPU_bne_branches_on_zero_clear() {
    let asm = "
        LDA #$F0
    MAIN:
        ADC #1
        BNE MAIN
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0x00, cpu.registers.A);
    assert_eq!(true, cpu.flags.zero);
}

#[test]
fn INTEGRATION_CPU_bpl_branches_on_sign_clear() {
    let asm = "
        LDA #$0A
        BPL END
        LDA #$FF
    END:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0x0A, cpu.registers.A);
    assert_eq!(false, cpu.flags.sign);
}

#[test]
fn INTEGRATION_CPU_bpl_does_not_branch_on_sign_set() {
    let asm = "
        LDA #$F0
        BPL END
        LDA #$FF
    END:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0xFF, cpu.registers.A);
    assert_eq!(true, cpu.flags.sign);
}

#[test]
fn INTEGRATION_CPU_cmp_does_branch_on_accumulator_less_than_memory_bcc() {
    let asm = "
        LDA #$0F
        CMP #$FF
        BCC LESS
        LDA #$02
        JMP END
    LESS:
        LDA #$01
    END
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0x01, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_cmp_does_branch_on_accumulator_greater_than_memory_bcs() {
    let asm = "
        LDA #$FF
        CMP #$FE
        BCS MORE
        LDA #$01
        JMP END
    MORE:
        LDA #$02
    END
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0x02, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_cmp_does_branch_on_accumulator_less_than_equal_to_bcc() {
    let asm = "
        LDA #$FF
        CMP #$FF
        BCS EQUAL
        LDA #$01
        JMP END
    EQUAL:
        LDA #$03
    END
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(50);

    assert_eq!(0x03, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_dec_decrements() {
    let asm = "
        LDA #$FF
        STA $0100
        DEC $0100
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(3);

    assert_eq!(0xFE, cpu.memory[0x100]);
}

#[test]
fn INTEGRATION_CPU_dex_decrements() {
    let asm = "
        LDX #$05
        LDA #$FF
        STA $0100
    LOOP:
        DEC $0100
        DEX
        BNE LOOP
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(20);

    assert_eq!(0xFA, cpu.memory[0x100]);
}

#[test]
fn INTEGRATION_CPU_jsr_rts_combination_works() {
    let asm = "
        LDA #$FF
        LDA #$FE
        JSR SUBROUTINE
        LDA #$0A
        JMP END

    SUBROUTINE:
        LDA #$AA
        RTS
    END:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(20);

    assert_eq!(0x0A, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_jsr_rts_combination_works_when_code_segment_loaded_at_weird_address() {
    let asm = "
        LDA #$FF
        LDA #$FE
        JSR SUBROUTINE
        LDA #$0A
        JMP END

    SUBROUTINE:
        LDA #$AA
        RTS
    END:
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], 0xABCD);  // Load it at a weird address

    cpu.step_n(20);

    assert_eq!(0x0A, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_lsr_can_halve_a_number() {
    let asm = "
        ; Halve the value at $1000
        LDA #$56
        STA $1000
        LSR $1000

        ; Halve the value in the Accumulator
        LDA #$40
        LSR
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(20);

    assert_eq!(0x20, cpu.registers.A);
    assert_eq!(0x2B, cpu.memory[0x1000]);
}

#[test]
fn INTEGRATION_CPU_ora_ors_against_accumulator() {
    let asm = "
        LDA #$E7    ; 1110 0111
        ORA #$18
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(10);

    assert_eq!(0xFF, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_pha_pla() {
    let asm = "
        LDA #$55
        PHA
        LDA #$FF
        PLA
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(3);

    assert_eq!(0xFF, cpu.registers.A);

    cpu.step();

    assert_eq!(0x55, cpu.registers.A);
}

#[test]
fn INTEGRATION_CPU_rol() {
    let asm = "
        ; To explain this: 0xFF + 0x0A will wrap to
        ; 0x09 + Carry. 0x09 << 1 is 0x12 + 1 for the
        ; Carry. Therefore, it should equal 0x13.
        LDA #$FF
        ADC #$0A
        ROL
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step_n(3);

    assert_eq!(0x13, cpu.registers.A);
}
