# rs6502 [![Build Status](https://travis-ci.org/simon-whitehead/rs6502.svg?branch=master)](https://travis-ci.org/simon-whitehead/rs6502)
A crate for the 6502 microprocessor.

This crate includes:

* A 6502 Disassembler.
* A 6502 Assembler.
* A 6502 Emulator.

## The Disassembler
The disassembler is quite basic and supports a few options. It can output just basic
6502 assembly or it can include memory offsets and the bytecode. For example:

```
let dasm = Disassembler::new();
let code: Vec<u8> = vec![0xA9, 0x00, 0xA8, 0x91, 0xFF, 0xC8, 0xCA, 0xD0, 0xFA, 0x60];
let asm = dasm.disassemble(&code);
```

produces this output:

```
0000 A9 99 LDA #$00
0002 A8    TAY
0003 91 FF STA ($FF),Y
0005 C8    INY
0006 CA    DEX
0007 D0 FA BNE $0003
0009 60    RTS
```
The disassembler automatically adjusts relative branching offsets to be memory offsets.

## The Assembler

The assembler is a very basic assembler that currently only supports a few basic things.

### Variables

The assembler happily supports variables for addresses. It does not currently support immediate values as variables.

Example:

```
MEMORY_ADDRESS = $0100
LDA #$FF
STA MEMORY_ADDRESS
```
Will compile to `A9 FF 85 00 10`.

The assembler does not currently have the ability to segment code and load it at specific addresses. This
is, however, a feature I would like to bring to it in future.

## The Emulator
The emulator supports all _supported_ opcodes for the 6502 Microprocessor. It does not currently support any of the
undocumented/unsupported upcodes.

### Timing
The emulator does not currently include any timing code. That is an exercise left to the consumer. As it stands, the
emulator will happily smash through as much code as fast as it possibly can.

## Contributing
I will accept any contributors with open arms. Whether you're interested in adding documentation, fixing code, writing tests
or even as far as converting the parser to be based on a parser-combinator library. Open to all suggestions. So please, feel
free to open an issue to discuss your ideas!

### A special mention

A special mention needs to go out to [Retro6502](https://github.com/seasalim/retro6502). My implementation of the `ADC` and `SBC`
opcodes was quite broken and when researching solutions I came across this repository. I implemented the masking tricks in the
Retro6502 implementation which fixed the issues I was facing. So thank you for that.

## LICENSE
This repository is MIT licensed. I hope its helpful for someone - I certainly learned a lot implementing it.
