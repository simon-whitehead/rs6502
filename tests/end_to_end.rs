
extern crate rs6502;

#[test]
fn INTEGRATION_can_assemble_disassemble_basic_opcodes() {
    let asm = "LDA $4400";

    let mut assembler = rs6502::Assembler::new();
    let disassembler = rs6502::Disassembler::with_code_only();

    let bytecode = assembler.assemble_string(asm).unwrap();
    let disassembled = rs6502::Disassembler::clean_asm(disassembler.disassemble(&bytecode));

    assert_eq!(asm, disassembled.join("\n"));
}

#[test]
fn INTEGRATION_can_assemble_disassemble_clearmem_implementation() {
    let asm = "
            CLRMEM  LDA #$00
                    TAY             
            CLRM1   STA ($FF),Y
                    INY             
                    DEX             
                    BNE CLRM1
                    RTS             
    ";

    let mut assembler = rs6502::Assembler::new();
    let disassembler = rs6502::Disassembler::with_code_only();

    let bytecode = assembler.assemble_string(asm).unwrap();
    let disassembled = rs6502::Disassembler::clean_asm(disassembler.disassemble(&bytecode));

    let clean_disassembled = disassembled.join("\n");

    assert_eq!(rs6502::Disassembler::clean_asm("
        LDA #$00
        TAY
        STA ($FF),Y
        INY
        DEX
        BNE $0003
        RTS
    ")
                   .join("\n"),
               clean_disassembled);
}

#[test]
fn INTEGRATION_can_add_basic_numbers_in_accumulator() {
    let asm = "
        LDA #$20
        ADC #$10    ; A register should equal 48
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step();
    cpu.step();

    assert_eq!(0x30, cpu.registers.A);
}

#[test]
fn INTEGRATION_can_add_binary_coded_decimal_numbers_in_accumulator() {
    let asm = "
        SED
        LDA #$20
        ADC #$05    ; A register should equal 0x25
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step();
    cpu.step();
    cpu.step();

    assert_eq!(0x25, cpu.registers.A);
}

#[test]
fn INTEGRATION_can_add_mixed_mode_numbers_in_accumulator() {
    let asm = "
        LDA #$20
        ADC #10    ; A register should equal 0x2A
    ";

    let mut cpu = rs6502::Cpu::new();
    let mut assembler = rs6502::Assembler::new();

    let bytecode = assembler.assemble_string(asm).unwrap();
    cpu.load(&bytecode[..], None);

    cpu.step();
    cpu.step();

    assert_eq!(0x2A, cpu.registers.A);
}