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
    const BITMASK_NEGATIVE: u8 = 0b1000_0000;
    const BITMASK_OVERFLOW: u8 = 0b0100_0000;
    // 3rd status bit unused
    const BITMASK_BREAK: u8 = 0b0001_0000;
    const BITMASK_DECIMAL: u8 = 0b0000_1000;
    const BITMASK_INTERRUPT: u8 = 0b0000_0100;
    const BITMASK_ZERO: u8 = 0b0000_0010;
    const BITMASK_CARRY: u8 = 0b0000_0001;

    fn set_flag(&mut self, bitmask: u8, value: bool) {
        if value {
            self.p |= bitmask;
        } else {
            self.p &= !bitmask;
        }
    }

    pub fn get_negative(&mut self) -> bool {
        (self.p & Self::BITMASK_NEGATIVE) != 0
    }

    pub fn set_negative(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_NEGATIVE, value);
    }

    pub fn get_overflow(&mut self) -> bool {
        (self.p & Self::BITMASK_OVERFLOW) != 0
    }

    pub fn set_overflow(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_OVERFLOW, value);
    }

    pub fn get_break(&mut self) -> bool {
        (self.p & Self::BITMASK_BREAK) != 0
    }

    pub fn set_break(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_BREAK, value);
    }

    pub fn get_decimal(&mut self) -> bool {
        (self.p & Self::BITMASK_DECIMAL) != 0
    }

    pub fn set_decimal(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_DECIMAL, value);
    }

    pub fn get_interrupt(&mut self) -> bool {
        (self.p & Self::BITMASK_INTERRUPT) != 0
    }

    pub fn set_interrupt(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_INTERRUPT, value);
    }

    pub fn get_zero(&mut self) -> bool {
        (self.p & Self::BITMASK_ZERO) != 0
    }

    pub fn set_zero(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_ZERO, value);
    }

    pub fn get_carry(&mut self) -> bool {
        (self.p & Self::BITMASK_CARRY) != 0
    }

    pub fn set_carry(&mut self, value: bool) {
        self.set_flag(Self::BITMASK_CARRY, value);
    }

    pub fn reset(&mut self) {
        *self = RegisterSet::default();
    }
}
