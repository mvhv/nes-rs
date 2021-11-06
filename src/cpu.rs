use crate::memory::Memory;

use std::ops::Range;

/// The NES CPU - Ricoh 2A03 (Modified MOS 6502)
pub struct CPU {
    /// Program Counter
    pc: u16,
    /// Stack Pointer
    sp: u8,
    /// Accumulator
    a: u8,
    /// Index Register X
    x: u8,
    /// Index Register Y
    y: u8,
    /// Processor Status
    p: u8,
    /// CPU Memory
    mem: Memory<0xFFFF>
}

#[allow(unused)]
impl CPU {
    const CPU_RAM_ADDR_MIN: u16 = 0x0000;
    const IO_REG_ADDR_MIN: u16 = 0x2000;
    const EXP_ROM_ADDR_MIN: u16 = 0x4020;
    const SAVE_RAM_ADDR_MIN: u16 = 0x6000;
    const PRG_ROM_ADDR_MIN: u16 = 0x8000;
    const ADDR_MAX: u16 = 0xFFFF;

    const CPU_RAM: Range<u16> = CPU::CPU_RAM_ADDR_MIN..CPU::IO_REG_ADDR_MIN;
    const IO_REG: Range<u16> = CPU::IO_REG_ADDR_MIN..CPU::EXP_ROM_ADDR_MIN;

    const STACK_ADDR_MIN: u16 = 0x0100;
    const STACK_ADDR_MAX: u16 = 0x01FF;
}


impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: u16::MIN,
            sp: u8::MAX,
            a: u8::MIN,
            x: u8::MIN,
            y: u8::MIN,
            p: u8::MIN,
            mem: Memory::default()
        }
    }

    pub fn load_at(&mut self, addr: usize, data: &[u8]) {
        self.mem.load_at(addr, data);
    }

    /// basic test signatureW
    pub fn run(&mut self) {

        loop {
            let opcode = self.mem.get(self.pc as usize);
            self.pc += 1;

            match opcode {
                0xA9 => {
                    let param = self.mem.get(self.pc as usize);
                    self.pc += 1;
                    self.a = param;

                    if self.a == 0 {
                        self.p = self.p | 0b0000_0010;
                    } else {
                        self.p = self.p & 0b1111_1101;
                    }

                    if self.a & 0b1000_0000 != 0 {
                        self.p = self.p | 0b1000_0000;
                    } else {
                        self.p = self.p & 0b0111_1111;
                    }
                },
                0x00 => return,
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_at(0, &vec![0xa9, 0x00, 0x00]);
        cpu.pc = 0;
        cpu.run();
        assert!(cpu.p & 0b0000_0010 == 0b10);
    }

}

