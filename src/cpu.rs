mod ops;
mod reg;
mod addr;

use crate::memory::MemoryMap;
use crate::cpu::reg::RegisterSet;
use crate::cpu::ops::Opcode;
use crate::cpu::addr::AddressMode;

use std::ops::Range;


/// The NES CPU - Ricoh 2A03 (Modified MOS 6502)
#[derive(Default)]
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



    fn do_add(&mut self, opcode: &Opcode) {
        let addr = self.get_operand_address(&opcode.mode);
        let operand = self.mem.read_u8(addr);

        let (result, overflow) = self.reg.a.overflowing_add(operand);
        
        self.reg.a = result;
        self.reg.update_overflow(overflow);

        self.reg.pc += opcode.bytes - 1;
    }

    fn do_sub(&mut self, opcode: &Opcode) {
        let addr = self.get_operand_address(&opcode.mode);
        let operand = self.mem.read_u8(addr);

        let (result, overflow) = self.reg.a.overflowing_add(operand);
        
        self.reg.a = result;
        self.reg.update_overflow(overflow);

        self.reg.pc += opcode.bytes - 1;
    }

    fn do_and(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_shift(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_bit_test(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_branch(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_break(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_compare(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_decrement(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_status_check(&mut self, opcode: &Opcode) {
        todo!()
    }
    
    fn do_bitwise_xor(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_increment(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_jump(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_load(&mut self, opcode: &Opcode) {
        if opcode.code == 0xA9 {
            let param = self.mem.read_u8(self.reg.pc);
            self.reg.pc += opcode.bytes - 1;
            self.reg.a = param;
    
            self.update_zero_and_negative_flags(self.reg.a)

        } else {
            todo!()
        }
    }

    fn do_nop(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_bitwise_or(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_transfer(&mut self, opcode: &Opcode) {
        if opcode.code == 0xAA {
            self.reg.x = self.reg.a;
            self.update_zero_and_negative_flags(self.reg.x);
        } else {
            todo!()
        }
    }

    fn do_register_decrement(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_register_increment(&mut self, opcode: &Opcode) {
        if opcode.code == 0xE8 {
            // TODO: this sucks, look into Wrapping<u8>
            self.reg.x = self.reg.x.wrapping_add(1);
            
        } else {
            todo!()
        }
    }

    fn do_rotate(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_return_from_interrupt(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_return_from_subroutine(&mut self, opcode: &Opcode) {
        todo!()
    }

    // fn do_subtract(&mut self, opcode: &Opcode) {
    //     todo!()
    // }

    fn do_store_accumulator(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_stack_transfer(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_stack_op(&mut self, opcode: &Opcode) {
        todo!()
    }

    fn do_store_register(&mut self, opcode: &Opcode) {
        todo!()
    }

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
    
    fn get_operand_address(&self, mode: &AddressMode) -> u16 {
        use AddressMode::*;
        match mode {
            Implicit => panic!("AddressMode Error: No operand for Implicit addressing."),
            Accumulator => self.reg.a as u16, // may need to replace this
            Immediate => self.reg.pc,
            ZeroPage => self.mem.read_u8(self.reg.pc) as u16,
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
            Relative => todo!(),
            Absolute => self.mem.read_u16(self.reg.pc), // may also need to replace this
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
            Indirect => {
                let ptr = self.mem
                .read_u16(self.reg.pc);
                
                self.mem
                .read_u16(ptr)
            }
            IndirectX => {
                let ptr = self.mem
                    .read_u8(self.reg.pc)
                    .wrapping_add(self.reg.x);

                u16::from_le_bytes([
                    self.mem.read_u8(ptr as u16),
                    self.mem.read_u8(ptr.wrapping_add(1) as u16),
                ])
            }
            IndirectY => {
                let ptr = self.mem
                    .read_u8(self.reg.pc);

                u16::from_le_bytes([
                    self.mem.read_u8(ptr as u16),
                    self.mem.read_u8(ptr.wrapping_add(1) as u16),
                ])
                    .wrapping_add(self.reg.y as u16)
            },
            NoAddressing => panic!("AddressMode Error: NoAddressing is not implemented."),
        }
    }

    pub fn step(&mut self, code: u8) {
        use ops::Mnemonic::*;

        self.reg.pc += 1;
        let &opcode = ops::CPU_OPCODES_MAP.get(&code).expect(&format!("ERROR: Opcode {:x} unimplemented", code));

        match opcode.mnemonic {
            // Add with carry
            ADC => self.do_add(opcode),
            // Bitwise AND with accumulator
            AND => self.do_and(opcode),
            // Arithmetic shift left
            ASL => self.do_shift(opcode),
            // Test bits
            BIT => self.do_bit_test(opcode),
            // Branch instructions
            BPL | BMI | BVC | BVS | BCC | BCS | BNE | BEQ => self.do_branch(opcode),
            BRK => self.do_break(opcode),
            CMP | CPX | CPY => self.do_compare(opcode),
            // Decrement memory
            DEC => self.do_decrement(opcode),
            // Bitwise exclusive OR
            EOR => self.do_bitwise_xor(opcode),
            // Flag (processor status) instructions
            CLC | SEC | CLI | SEI | CLV | CLD | SED => self.do_status_check(opcode),
            // Increment memory
            INC => self.do_increment(opcode),
            // Jump
            JMP | JSR => self.do_jump(opcode),
            // Load accumulator
            LDA | LDX | LDY => self.do_load(opcode),
            // Logical shift right
            LSR => self.do_shift(opcode),
            // No operation
            NOP => self.do_nop(opcode),
            // Bitwise OR with accumulator
            ORA => self.do_bitwise_or(opcode),
            // Register instructions
            TAX | TXA | TAY | TYA => self.do_transfer(opcode),
            DEX | DEY => self.do_register_decrement(opcode),
            INX | INY => self.do_register_increment(opcode),
            // Rotate
            ROL | ROR => self.do_rotate(opcode),
            // Return from interrupt
            RTI => self.do_return_from_interrupt(opcode),
            // Return from subroutine
            RTS => self.do_return_from_subroutine(opcode),
            // Subtract with carry
            SBC => self.do_sub(opcode),
            // Store accumulator
            STA => self.do_store_accumulator(opcode),
            // Stack instructions
            TXS | TSX => self.do_stack_transfer(opcode),
            PHA | PLA | PHP | PLP => self.do_stack_op(opcode),
            // Store X register
            STX | STY => self.do_store_register(opcode),
        }
    }

    /// Continuously run program from current location until BRK
    pub fn run(&mut self) {
        loop {
            match self.mem.read_u8(self.reg.pc) {
                0x00 => return, // break (temporary manual check until we implement proper interrupts)
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


    fn tax(&mut self, code: &Opcode) {
        self.reg.x = self.reg.a;

        self.update_zero_and_negative_flags(self.reg.x);
    }

    fn inx(&mut self, code: &Opcode) {
        // TODO: this sucks, look into Wrapping<u8>
        self.reg.x = self.reg.x.wrapping_add(1);
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
    use ops::*;
    
    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load(0, &[0xA9, 0x05, 0x00]);
        cpu.run();

        assert_eq!(cpu.reg.a, 0x05);
        assert!(cpu.reg.p & 0b0000_0010 == 0b00);
        assert!(cpu.reg.p & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load(0, &[0xA9, 0x00, 0x00]);
        cpu.run();

        assert!(cpu.reg.p & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(0, &[0xAA, 0x00]);
        cpu.reg.a = 10;
        cpu.run();

        assert_eq!(cpu.reg.x, 10);
    }

    #[test]
    fn test_load_transfer_increment() {
        let mut cpu = CPU::new();
        cpu.load(0, &[0xA9, 0xC0, 0xAA, 0xE8, 0x00]);
        cpu.run();

        assert_eq!(cpu.reg.x, 0xC1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(0, &[0xE8, 0xE8, 0x00]);
        cpu.reg.x = 0xFF;
        cpu.run();

        assert_eq!(cpu.reg.x, 0x01);
    }

    #[test]
    fn test_program_load() {
        let mut cpu = CPU::new();
        cpu.load_program(&[0xA9, 0xC0, 0xAA, 0xE8, 0x00]);
        cpu.interrupt_reset();
        cpu.run();

        assert_eq!(cpu.reg.x, 0xC1);
    }

    #[test]
    fn test_add_no_carry() {
        let mut cpu = CPU::new();
        cpu.load_program(&[
            0xA9, // load acc immediate
            0x10, // 16
            0x69, // add acc immediate
            0x13, // 19
            0x00,
        ]);
        cpu.interrupt_reset();
        cpu.run();
        // assert 16 + 19 = 35
        assert_eq!(cpu.reg.a, 0x23);
    }

    #[test]
    fn test_add_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_program(&[
            0xA9, // load acc immediate
            0xFF, // 255
            0x69, // add acc immediate
            0x06, // 6
            0x00,
        ]);
        cpu.interrupt_reset();
        cpu.run();
        // assert 255 + 6 = 301 mod 256 = 5
        assert_eq!(cpu.reg.a, 0x05);
    }
}

