mod op;
mod reg;

use crate::memory::MemoryMap;
use crate::cpu::reg::RegisterSet;

use std::ops::Range;

/// The NES CPU - Ricoh 2A03 (Modified MOS 6502)
/// #[derive(default)]
pub struct CPU {
    /// CPU Register Set
    reg: RegisterSet,
    /// CPU Memory
    mem: MemoryMap<0xFFFF>,
}

#[allow(unused)]
impl CPU {
    const CPU_RAM_ADDR_MIN: u16 = 0x0000;
    const IO_REG_ADDR_MIN: u16 = 0x2000;
    const EXP_ROM_ADDR_MIN: u16 = 0x4020;
    const SAVE_RAM_ADDR_MIN: u16 = 0x6000;
    const PRG_ROM_ADDR_MIN: u16 = 0x8000;
    const NES_ADDR_MAX: u16 = 0xFFFF;

    const CPU_RAM: Range<u16> = CPU::CPU_RAM_ADDR_MIN..CPU::IO_REG_ADDR_MIN;
    const IO_REG: Range<u16> = CPU::IO_REG_ADDR_MIN..CPU::EXP_ROM_ADDR_MIN;

    const STACK_ADDR_MIN: u16 = 0x0100;
    const STACK_ADDR_MAX: u16 = 0x01FF;

    const PRG_START_ADDR: u16 = 0xFFFC;
}


impl CPU {
    pub fn new() -> Self {
        CPU {
            reg: RegisterSet::default(),
            mem: MemoryMap::default()
        }
    }

    pub fn load(&mut self, addr: u16, data: &[u8]) {
        self.mem.load(addr, data);
    }

    /// Continuously run program from current location until BRK
    pub fn run(&mut self) {

        loop {
            let opcode = self.mem.read_u8(self.reg.pc);
            self.reg.pc += 1;

            match opcode {
                op::BRK => return,
                op::LDA => self.lda(opcode),
                op::TAX => self.tax(opcode),
                op::INX => self.inx(opcode),
                op::CPY => self.cpy(opcode),
                _ => unimplemented!()
            }
        }
    }

    fn lda(&mut self, code: u8) {
        let param = self.mem.read_u8(self.reg.pc);
        self.reg.pc += 1;
        self.reg.a = param;

        self.update_zero_and_negative_flags(self.reg.a)
    }

    fn tax(&mut self, code: u8) {
        self.reg.x = self.reg.a;

        self.update_zero_and_negative_flags(self.reg.x);
    }

    fn inx(&mut self, code: u8) {
        // TODO: this sucks, look into Wrapping<u8>
        self.reg.x = self.reg.x.wrapping_add(1);
    }

    fn cpy(&mut self, code: u8) {
        todo!();
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.reg.p = self.reg.p | 0b0000_0010;
        } else {
            self.reg.p = self.reg.p | 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.reg.p = self.reg.p | 0b1000_0000;
        } else {
            self.reg.p = self.reg.p & 0b0111_1111;
        }
    }

    fn reset_state(&mut self) {
        self.reg = RegisterSet::default();
    }

    fn interrupt_reset(&mut self) {
        self.reg.reset();
        self.reg.pc = CPU::PRG_START_ADDR;
    }

    fn load_program(&mut self, program: &[u8]) {
        self.mem.load(CPU::PRG_ROM_ADDR_MIN, program);
        self.mem.write_u16(0xFFFC, CPU::PRG_ROM_ADDR_MIN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use op::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load(0, &[LDA, 0x00, BRK]);
        cpu.run();

        assert!(cpu.reg.p & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(0, &[TAX, BRK]);
        cpu.reg.a = 10;
        cpu.run();

        assert_eq!(cpu.reg.x, 10);
    }

    #[test]
    fn test_load_transfer_increment() {
        let mut cpu = CPU::new();
        cpu.load(0, &[LDA, 0xc0, TAX, INX, BRK]);
        cpu.run();

        assert_eq!(cpu.reg.x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(0, &[INX, INX, BRK]);
        cpu.reg.x = 0xff;
        cpu.run();

        assert_eq!(cpu.reg.x, 1u8);
    }
}

