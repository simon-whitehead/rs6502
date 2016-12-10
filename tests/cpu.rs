
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