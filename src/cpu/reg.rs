pub struct RegisterSet {
    /// Program Counter
    pub pc: u16,
    /// Stack Pointer
    pub sp: u8,
    /// Accumulator
    pub a: u8,
    /// Index Register X
    pub x: u8,
    /// Index Register Y
    pub y: u8,
    /// Processor Status Word \[ N V _ B D I Z C \]
    pub p: u8,
}

impl Default for RegisterSet {
    fn default() -> Self {
        RegisterSet {
            pc: u16::MIN,
            sp: u8::MAX,
            a: u8::MIN,
            x: u8::MIN,
            y: u8::MIN,
            p: u8::MIN,
        }
    }
}

impl RegisterSet {
    const BITMASK_OVERFLOW: u8 = 0b0100_0000;

    pub fn update_overflow(&mut self, overflow: bool) {
        if overflow {
            self.p |= Self::BITMASK_OVERFLOW;
        }
    }

    pub fn reset(&mut self) {
        *self = RegisterSet::default();
    }
}