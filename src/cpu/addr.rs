#[derive(Debug)]
pub enum AddressMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX, // Indexed Indirect
    IndirectY, // Indirect Indexed
    NoAddressing,
}