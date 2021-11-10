mod ops;
mod reg;
mod addr;

use crate::memory::MemoryMap;
use crate::cpu::reg::RegisterSet;
use crate::cpu::ops::Opcode;
use crate::cpu::addr::AddressMode;

use std::ops::Range;
use std::result;

use self::ops::Mnemonic;


/// The NES CPU - Ricoh 2A03 (Modified MOS 6502)
#[derive(Default)]
pub struct CPU {
    /// CPU Register Set
    reg: RegisterSet,
    /// CPU Memory
    mem: MemoryMap<0xFFFF>,
}

fn twos_add_overflow_carry(value: u8, operand: u8) ->  (u8, bool, bool) {
    let val_pos = is_positive(value);
    let op_pos = is_positive(operand);

    let (result, carry) = value.overflowing_add(operand);
    let res_pos = is_positive(result);
    let overflow = if res_pos {
        !val_pos && !op_pos
    } else {
        val_pos && op_pos
    };

    (result, overflow, carry)
}

fn is_negative(val:u8) -> bool {
    (val >> 7) != 0
}

fn is_positive(val:u8) -> bool {
    (val >> 7) == 0
}

fn sign_bit(val: u8) -> u8 {
    val >> 7
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

    fn update_zn_from_accumulator(&mut self) {
        self.reg.set_negative(self.reg.a >= 0x80);
        self.reg.set_zero(self.reg.a == 0x00);
    }

    fn update_zn_from_value(&mut self, value: u8) {
        self.reg.set_negative(value >= 0x80);
        self.reg.set_zero(value == 0x00);
    }

    fn get_operand_u8(&mut self, opcode: &Opcode) -> u8 {
        self.mem
            .read_u8(
                self.get_operand_address(&opcode.mode)
        )
    }

    fn increment_pc(&mut self, opcode: &Opcode) {
        self.reg.pc += opcode.bytes - 1;
    }

    fn do_base_add(&mut self, operand: u8, carry: u8) {
        let (partial_result, partial_overflow, partial_carry) = twos_add_overflow_carry(self.reg.a, operand);
        let (carried_result, carried_overflow, carried_carry) = twos_add_overflow_carry(partial_result, carry);

        self.reg.a = carried_result;

        self.update_zn_from_accumulator();
        self.reg.set_carry(partial_carry || carried_carry);
        self.reg.set_overflow(partial_overflow || carried_overflow);
    }

    fn do_add_sub(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);
        let carry_multiplier = self.reg.get_carry() as u8;

        if self.reg.get_decimal() {
            panic!("ERROR: Decimal mode is not avaliable on the 6502");
        } else {
            match &opcode.mnemonic {
                Mnemonic::ADC => self.do_base_add(operand, 0x01 * carry_multiplier),
                Mnemonic::SBC => self.do_base_add(!operand + 1, 0xFF * carry_multiplier),
                x => panic!("ERROR: Addition not a valid instruction for: {:?}", x)
            }
        }

        self.increment_pc(opcode);
    }

    fn do_and(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);
        
        self.reg.a &= operand;

        self.update_zn_from_accumulator();

        self.increment_pc(opcode);
    }

    fn do_left_shift(&mut self, opcode: &Opcode) {
        
        match &opcode.mode {
            AddressMode::Accumulator =>{
                self.reg.set_carry(is_positive(self.reg.a));
                self.reg.a <<= 1;
                self.update_zn_from_accumulator();
            },
            mode => {
                let addr = self.get_operand_address(mode);
                let operand = self.mem.read_u8(addr);
                self.reg.set_carry(is_positive(operand));
                let result = operand << 1;
                self.mem.write_u8(addr, result);
                self.update_zn_from_value(result);
            },
        }
        
        self.increment_pc(opcode);

        
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

    fn do_flag(&mut self, opcode: &Opcode) {
        use ops::Mnemonic::{CLC, SEC, CLI, SEI, CLV, CLD, SED};

        match &opcode.mnemonic {
            CLC => self.reg.set_carry(false),
            SEC => self.reg.set_carry(true),
            CLI => self.reg.set_interrupt(false),
            SEI => self.reg.set_interrupt(true),
            CLV => self.reg.set_overflow(false),
            CLD => self.reg.set_decimal(false),
            SED => self.reg.set_decimal(true),
            x => panic!("{:?} is not a flag operation", x),
        }
        
        self.increment_pc(opcode);
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

    fn do_shift(&self, opcode: &Opcode) {
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
            // Add or Subtract
            ADC | SBC => self.do_add_sub(opcode),
            // Bitwise AND with accumulator
            AND => self.do_and(opcode),
            // Arithmetic shift left
            ASL => self.do_left_shift(opcode),
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
            CLC => self.reg.set_carry(false),
            SEC => self.reg.set_carry(true),
            CLI => self.reg.set_interrupt(false),
            SEI => self.reg.set_interrupt(true),
            CLV => self.reg.set_overflow(false),
            CLD => self.reg.set_decimal(false),
            SED => self.reg.set_decimal(true),
            // Increment memory
            INC => self.do_increment(opcode),
            // Jump
            JMP | JSR => self.do_jump(opcode),
            // Load register
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
            SBC => self.do_add_sub(opcode),
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
            self.reg.p = self.reg.p & 0b1111_1101;
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
    
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
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

    // #[test]
    // fn test_add_no_carry() {
    //     let mut cpu = CPU::new();
    //     cpu.load_program(&[
    //         0xA9, // load acc immediate
    //         0x10, // 16
    //         0x69, // add acc immediate
    //         0x13, // 19
    //         0x00,
    //     ]);
    //     cpu.interrupt_reset();
    //     cpu.run();
    //     // assert 16 + 19 = 35
    //     assert_eq!(cpu.reg.a, 0x23);
    // }

    // #[test]
    // fn test_add_with_carry() {
    //     let mut cpu = CPU::new();
    //     cpu.load_program(&[
    //         0xA9, // load acc immediate
    //         0xFF, // 255
    //         0x69, // add acc immediate
    //         0x06, // 6
    //         0x00,
    //     ]);
    //     cpu.interrupt_reset();
    //     cpu.run();
    //     // assert 255 + 6 = 301 mod 256 = 5
    //     assert_eq!(cpu.reg.a, 0x05);
    // }

    #[test]
    fn test_set_carry() {
        let mut cpu = CPU::new();
        //  1 + 1 = 2, returns C = 0
        cpu.load_program(&[
            0xA9, 0x01,
            0x69, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), false);

        //  1 + -1 = 0, returns C = 1
        cpu.load_program(&[
            0xA9, 0x01,
            0x69, 0xFF,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), true);

        //  127 + 1 = 128, returns C = 0
        cpu.load_program(&[
            0xA9, 0x7F,
            0x69, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), false);
        
        //  -128 + -1 = -129, returns C = 1
        cpu.load_program(&[
            0xA9, 0x80,
            0x69, 0xFF,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), true);
    }

    #[test]
    fn test_set_overflow() {
        let mut cpu = CPU::new();
        //  1 + 1 = 2, returns V = 0
        cpu.load_program(&[
            0xA9, 0x01,
            0x69, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);

        //  1 + -1 = 0, returns V = 0
        cpu.load_program(&[
            0xA9, 0x01,
            0x69, 0xFF,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);

        //  127 + 1 = 128, returns V = 1
        cpu.load_program(&[
            0xA9, 0x7F,
            0x69, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // -128 + -1 = -129, returns V = 1
        cpu.load_program(&[
            0xA9, 0x80,
            0x69, 0xFF,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // 0 - 1 = -1, returns V = 0
        cpu.load_program(&[
            0xA9, 0x00,
            0xE9, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);
        
        // -128 - 1 = -129, returns V = 1
        cpu.load_program(&[
            0xA9, 0x80,
            0xE9, 0x01,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // 127 - -1 = 128, returns V = 1
        cpu.load_program(&[
            0xA9, 0x7F,
            0xE9, 0xFF,
            0x00
        ]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);
    }

    fn test_use_carry() {
        todo!();
    }
}

