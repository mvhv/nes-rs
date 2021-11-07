mod op;
mod reg;

use crate::memory::MemoryMap;
use crate::cpu::reg::RegisterSet;
use crate::cpu::op::Opcode;

use std::ops::Range;

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    None
}

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

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        use AddressingMode::*;
        match mode {
            Immediate => self.reg.pc,
            ZeroPage => self.mem.read_u8(self.reg.pc) as u16,
            Absolute => self.mem.read_u16(self.reg.pc),
            ZeroPageX => {
                self.mem
                    .read_u8(self.reg.pc)
                    .wrapping_add(self.reg.x) as u16
            },
            ZeroPageY => {
                self.mem
                    .read_u8(self.reg.pc)
                    .wrapping_add(self.reg.y) as u16
            },
            AbsoluteX => {
                self.mem
                    .read_u16(self.reg.pc)
                    .wrapping_add(self.reg.x as u16)
            }
            AbsoluteY => {
                self.mem
                    .read_u16(self.reg.pc)
                    .wrapping_add(self.reg.y as u16)
            },
            IndirectX => {
                let ptr = self.mem
                    .read_u8(self.reg.pc)
                    .wrapping_add(self.reg.x);

                u16::from_le_bytes([self.mem.read_u8(ptr as u16), self.mem.read_u8(ptr.wrapping_add(1) as u16)])
            }
            IndirectY => {
                let ptr = self.mem
                    .read_u8(self.reg.pc);

                u16::from_le_bytes([self.mem.read_u8(ptr as u16), self.mem.read_u8(ptr.wrapping_add(1) as u16)])
                    .wrapping_add(self.reg.y as u16)
            },
            None => panic!("Addressing Mode Failure: AddressingMode::None"),
        }
    }

    pub fn step(&mut self, code: u8) {
        self.reg.pc += 1;
        let opcode = op::CPU_OPCODES_MAP.get(&code).expect(&format!("ERROR: Opcode {:x} unimplemented", code));

        match code {
            // LDA
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(opcode),
            // STA
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(opcode),
            // TAX
            0xAA => self.tax(opcode),
            // INX
            0xE8 => self.inx(opcode),
            // CPY
            0xC0 => self.cpy(opcode),
            _ => todo!()
        }
    }

    /// Continuously run program from current location until BRK
    pub fn run(&mut self) {
        loop {
            match self.mem.read_u8(self.reg.pc) {
                op::BRK => return,
                opcode => self.step(opcode),
            }
        }
    }

    fn lda(&mut self, code: &Opcode) {
        let param = self.mem.read_u8(self.reg.pc);
        self.reg.pc += 1;
        self.reg.a = param;

        self.update_zero_and_negative_flags(self.reg.a)
    }

    fn sta(&mut self, code: &Opcode){
        todo!();
    }

    fn tax(&mut self, code: &Opcode) {
        self.reg.x = self.reg.a;

        self.update_zero_and_negative_flags(self.reg.x);
    }

    fn inx(&mut self, code: &Opcode) {
        // TODO: this sucks, look into Wrapping<u8>
        self.reg.x = self.reg.x.wrapping_add(1);
    }

    fn cpy(&mut self, code: &Opcode) {
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

    // fn reset_state(&mut self) {
    //     self.reg = RegisterSet::default();
    // }
    
    /// Reset register state and initialise program counter to value at 0xFFFC
    fn interrupt_reset(&mut self) {
        self.reg.reset();
        self.reg.pc = self.mem.read_u16(CPU::PRG_START_ADDR);
    }

    pub fn load_program(&mut self, program: &[u8]) {
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
        cpu.load(0, &[LDA, 0xC0, TAX, INX, BRK]);
        cpu.run();

        assert_eq!(cpu.reg.x, 0xC1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(0, &[INX, INX, BRK]);
        cpu.reg.x = 0xFF;
        cpu.run();

        assert_eq!(cpu.reg.x, 0x01);
    }

    #[test]
    fn test_program_load() {
        let mut cpu = CPU::new();
        cpu.load_program(&[LDA, 0xC0, TAX, INX, BRK]);
        cpu.interrupt_reset();
        cpu.run();

        assert_eq!(cpu.reg.x, 0xC1);
    }
}

