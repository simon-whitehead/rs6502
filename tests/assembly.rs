
extern crate rs6502;

#[test]
#[allow(non_snake_case)]
fn INTEGRATION_ASSEMBLY_can_assemble_disassemble_basic_opcodes() {
    let asm = "LDA $4400";

    let mut assembler = rs6502::Assembler::new();
    let disassembler = rs6502::Disassembler::with_code_only();

    let segments = assembler.assemble_string(asm, None).unwrap();
    println!("Found {} segments", segments.len());
    let disassembled =
        rs6502::Disassembler::clean_asm(disassembler.disassemble(&segments[0].code[..]));

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

    let segments = assembler.assemble_string(asm, None).unwrap();
    let disassembled = rs6502::Disassembler::clean_asm(disassembler.disassemble(&segments[0].code));

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
#[allow(non_snake_case)]
fn INTEGRATION_ASSEMBLY_can_assemble_disassemble_random_memory_segments() {
    let asm = "
        .ORG $D006
        .BYTE #$10, #$D0
    ";

    let mut assembler = rs6502::Assembler::new();
    let disassembler = rs6502::Disassembler::new();

    let segments = assembler.assemble_string(asm, None).unwrap();
    let disassembled = rs6502::Disassembler::clean_asm(disassembler.disassemble(&segments[0].code));

    let clean_disassembled = disassembled.join("\n");

    assert_eq!(rs6502::Disassembler::clean_asm("
        0000 BPL $00D0
    ")
                   .join("\n"),
               clean_disassembled);
}