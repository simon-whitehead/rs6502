#[derive(Debug, PartialEq)]
pub enum CpuErrorKind {
    SegFault,
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
}