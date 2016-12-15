
extern crate rs6502;

#[test]
#[allow(non_snake_case)]
fn INTEGRATION_ASSEMBLY_can_assemble_disassemble_basic_opcodes() {
    let asm = "LDA $4400";

    let mut assembler = rs6502::Assembler::new();
    let disassembler = rs6502::Disassembler::with_code_only();

    let bytecode = assembler.assemble_string(asm).unwrap();
    let disassembled = rs6502::Disassembler::clean_asm(disassembler.disassemble(&bytecode));

    assert_eq!(asm, disassembled.join("\n"));
}

#[test]
#[allow(non_snake_case)]
fn INTEGRATION_ASSEMBLY_can_assemble_disassemble_clearmem_implementation() {
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