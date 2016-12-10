#[derive(Debug, PartialEq)]
pub enum CpuErrorKind {
    SegFault,
    InvalidOpCode,
}

#[derive(Debug, PartialEq)]
pub struct CpuError {
    message: String,
    addr: u16,
    kind: CpuErrorKind,
}

impl CpuError {
    pub fn code_segment_out_of_range(addr: u16) -> CpuError {
        CpuError {
            message: format!("CODE segment out of bounds"),
            addr: addr,
            kind: CpuErrorKind::SegFault,
        }
    }

    pub fn unknown_opcode(addr: u16, opcode: u8) -> CpuError {
        CpuError {
            message: format!("Unknown opcode {:02X} at {:04X}", opcode, addr),
            addr: addr,
            kind: CpuErrorKind::InvalidOpCode,
        }
    }
}