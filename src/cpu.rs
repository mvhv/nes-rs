mod addr;
mod ops;
mod reg;
mod decomp;

use crate::cpu::addr::AddressMode;
use crate::cpu::ops::Opcode;
use crate::cpu::reg::RegisterSet;
use crate::memory::{SimpleMap, MemoryMap};

use self::ops::Mnemonic;

/// The NES CPU - Ricoh 2A03 (Modified MOS 6502)
#[derive(Default)]
pub struct CPU {
    /// CPU Register Set
    reg: RegisterSet,
    /// CPU Memory
    mem: SimpleMap<0xFFFF>,
}

impl std::fmt::Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "CPU Dump:\nreg:\n{:#x?}\nmem:\n{:?}", self.reg, self.mem)
    }
}

fn twos_add_overflow_carry(value: u8, operand: u8) -> (u8, bool, bool) {
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

fn is_negative(val: u8) -> bool {
    val & 0b1000_0000 != 0
}

fn is_positive(val: u8) -> bool {
    val & 0b1000_0000 == 0
}

// fn sign_bit(val: u8) -> u8 {
//     val >> 7
// }

#[allow(unused)]
impl CPU {
    const CPU_RAM_ADDR_MIN: u16 = 0x0000;
    const IO_REG_ADDR_MIN: u16 = 0x2000;
    const EXP_ROM_ADDR_MIN: u16 = 0x4020;
    const SAVE_RAM_ADDR_MIN: u16 = 0x6000;
    const PRG_ROM_ADDR_MIN: u16 = 0x8000;
    const NES_ADDR_MAX: u16 = 0xFFFF;
    // const CPU_RAM: Range<u16> = CPU::CPU_RAM_ADDR_MIN..CPU::IO_REG_ADDR_MIN;
    // const IO_REG: Range<u16> = CPU::IO_REG_ADDR_MIN..CPU::EXP_ROM_ADDR_MIN;

    const STACK_ADDR_MIN: u16 = 0x0100;
    const STACK_ADDR_MAX: u16 = 0x01FF;

    const PRG_START_ADDR: u16 = 0xFFFC;
}

impl CPU {
    fn update_zn_from_accumulator(&mut self) {
        self.reg.set_negative(self.reg.a >= 0x80);
        self.reg.set_zero(self.reg.a == 0x00);
    }

    fn update_zn_from_value(&mut self, value: u8) {
        self.reg.set_negative(value >= 0x80);
        self.reg.set_zero(value == 0x00);
    }

    fn get_operand_u8(&mut self, opcode: &Opcode) -> u8 {
        self.mem.read_u8(self.get_operand_address(&opcode.mode))
    }

    // there is never a u16 operand, any apparent u16 operand is actually an address
    // fn get_operand_u16(&mut self, opcode: &Opcode) -> u16 {
    //     self.mem.read_u16(self.get_operand_address(&opcode.mode))
    // }

    fn increment_pc(&mut self, opcode: &Opcode) {
        self.reg.pc += opcode.bytes - 1;
    }

    fn do_base_add(&mut self, operand: u8, carry: u8) {
        let (partial_result, partial_overflow, partial_carry) =
            twos_add_overflow_carry(self.reg.a, operand);
        let (carried_result, carried_overflow, carried_carry) =
            twos_add_overflow_carry(partial_result, carry);

        self.reg.a = carried_result;

        self.update_zn_from_accumulator();
        self.reg.set_carry(partial_carry || carried_carry);
        self.reg.set_overflow(partial_overflow || carried_overflow);
    }

    fn do_add_sub(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);
        let carry_multiplier = self.reg.get_carry() as u8;

        // On the NES the decimal flag has no effect, else we'd check for it here
        match &opcode.mnemonic {
            Mnemonic::ADC => self.do_base_add(operand, 0x01 * carry_multiplier),
            Mnemonic::SBC => self.do_base_add(!operand + 1, 0xFF * carry_multiplier),
            x => panic!("ERROR: Addition not a valid instruction for: {:?}", x),
        }

        self.increment_pc(opcode);
    }

    fn do_and(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);

        self.reg.a &= operand;

        self.update_zn_from_accumulator();

        self.increment_pc(opcode);
    }

    /// Arithmetic shift is always left
    fn do_arithmetic_shift(&mut self, opcode: &Opcode) {
        match &opcode.mode {
            AddressMode::Accumulator => {
                self.reg.set_carry(self.reg.a & 0b1000_0000 != 0);
                self.reg.a <<= 1;
                self.update_zn_from_accumulator();
            }
            mode => {
                let addr = self.get_operand_address(mode);
                let operand = self.mem.read_u8(addr);
                self.reg.set_carry(self.reg.a & 0b1000_0000 != 0);
                let result = operand << 1;
                self.mem.write_u8(addr, result);
                self.update_zn_from_value(result);
            }
        }

        self.increment_pc(opcode);
    }

        /// Logical shift is always right
        fn do_logical_shift(&mut self, opcode: &Opcode) {
            match &opcode.mode {
                AddressMode::Accumulator => {
                    self.reg.set_carry(self.reg.a & 1 != 0);
                    self.reg.a >>= 1;
                    self.update_zn_from_accumulator();
                }
                mode => {
                    let addr = self.get_operand_address(mode);
                    let operand = self.mem.read_u8(addr);
                    self.reg.set_carry(self.reg.a & 1 != 0);
                    let result = operand >> 1;
                    self.mem.write_u8(addr, result);
                    self.update_zn_from_value(result);
                }
            }
    
            self.increment_pc(opcode);
        }

    fn do_bit_test(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);
        self.reg.set_zero((operand & self.reg.a) != 0);
        self.reg.set_negative(operand & 0b1000_0000 != 0);
        self.reg.set_overflow(operand & 0b0100_0000 != 0);

        self.increment_pc(opcode);
    }

    fn do_branch(&mut self, opcode: &Opcode) {
        let relative_offset = self.get_operand_u8(opcode);

        if match &opcode.mnemonic {
            Mnemonic::BPL => !self.reg.get_negative(),
            Mnemonic::BMI => self.reg.get_negative(),
            Mnemonic::BVC => !self.reg.get_overflow(),
            Mnemonic::BVS => self.reg.get_overflow(),
            Mnemonic::BCC => !self.reg.get_carry(),
            Mnemonic::BCS => self.reg.get_carry(),
            Mnemonic::BNE => !self.reg.get_zero(),
            Mnemonic::BEQ => self.reg.get_zero(),
            x => panic!("ERROR: Branch not a valid instruction for: {:?}", x),
        } {
            self.reg.pc = self.reg.pc.wrapping_add(relative_offset as u16);
        }
        // the relative offsets is from the end of the op
        // so we increment the pc as normal regardless
        self.increment_pc(opcode);
    }

    fn do_break(&mut self, _opcode: &Opcode) {
        self.reg.pc += 1;
        self.push_u16(self.reg.pc);
        // BRK sets bits 4 and 5 in the word pushed to the stack (but not in the actual register)
        self.push_u8(self.reg.p | 0b0011_0000);

        // DO INTERRUPT STUFF ================================================================================
    }

    fn do_compare(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);

        let base_value = match &opcode.mnemonic {
            Mnemonic::CMP => self.reg.a,
            Mnemonic::CPX => self.reg.x,
            Mnemonic::CPY => self.reg.y,
            x => panic!("ERROR: Compare not a valid instruction for: {:?}", x),
        };

        self.reg.set_carry(base_value > operand);
        self.update_zn_from_value(base_value - operand);

        self.increment_pc(opcode);
    }

    fn do_crement(&mut self, opcode: &Opcode) {
        let addr = self.get_operand_address(&opcode.mode);
        let value = self.mem.read_u8(addr);

        let result = match &opcode.mnemonic {
            Mnemonic::DEC => value.wrapping_add(1),
            Mnemonic::INC => value.wrapping_sub(1),
            x => panic!(
                "ERROR: Increment/Decrement not a valid instruction for: {:?}",
                x
            ),
        };

        self.mem.write_u8(addr, result);
        self.update_zn_from_value(result);

        self.increment_pc(opcode);
    }

    fn do_update_flag(&mut self, opcode: &Opcode) {
        use ops::Mnemonic::{CLC, CLD, CLI, CLV, SEC, SED, SEI};

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
        let operand = self.get_operand_u8(opcode);

        self.reg.a ^= operand;

        self.update_zn_from_accumulator();

        self.increment_pc(opcode);
    }

    fn do_jump(&mut self, opcode: &Opcode) {
        // jumps are absolute addressed, but the operand is the target, not the contents
        let destination = self.get_operand_address(&opcode.mode);
        self.reg.pc = destination;
    }

    fn do_subroutine_jump(&mut self, opcode: &Opcode) {
        // jumps are absolute addressed, but the operand is the target, not the contents
        let destination = self.get_operand_address(&opcode.mode);
        self.increment_pc(opcode);
        self.push_u16(self.reg.pc - 1);
        self.reg.pc = destination;
    }

    /// Returns the next (free) address on the stack
    fn get_sp(&self) -> u16 {
        CPU::STACK_ADDR_MIN + self.reg.sp as u16
    }
    
    fn increment_sp(&mut self) {
        self.reg.sp = self.reg.sp.wrapping_add(1);
    }
    
    fn decrement_sp(&mut self) {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }
    
    fn push_u8(&mut self, value: u8) {
        self.mem.write_u8(self.get_sp(), value);
        self.decrement_sp();
    }

    fn push_u16(&mut self, value: u16) {
        // TODO: Ensure that u16 values are written correctly in stack wrapping conditions
        let [lo, hi] = value.to_le_bytes();
        // most significant byte is pushed first
        self.push_u8(hi);
        self.push_u8(lo);
    }

    fn pull_u8(&mut self) -> u8 {
        self.increment_sp();
        self.mem.read_u8(self.get_sp())
    }

    fn pull_u16(&mut self) -> u16 {
        // TODO: Ensure that u16 values are read correctly in stack wrapping condition
        // least significant byte loaded first
        let lo = self.pull_u8();
        let hi = self.pull_u8();
        u16::from_le_bytes([lo, hi])
    }

    fn do_load(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);

        match &opcode.mnemonic {
            Mnemonic::LDA => self.reg.a = operand,
            Mnemonic::LDX => self.reg.x = operand,
            Mnemonic::LDY => self.reg.y = operand,
            x => panic!("ERROR: Load not a valid instruction for: {:?}", x),
        }

        self.update_zn_from_value(operand);
        self.increment_pc(opcode);
    }

    fn do_nop(&mut self, opcode: &Opcode) {
        self.increment_pc(opcode);
    }

    fn do_bitwise_or(&mut self, opcode: &Opcode) {
        let operand = self.get_operand_u8(opcode);

        self.reg.a |= operand;

        self.update_zn_from_accumulator();

        self.increment_pc(opcode);
    }

    fn do_register_update(&mut self, opcode: &Opcode) {
        let value = match &opcode.mnemonic {
            Mnemonic::TAX | Mnemonic::TAY => self.reg.a,
            Mnemonic::TXA => self.reg.x,
            Mnemonic::DEX => self.reg.x.wrapping_add(0xFF),
            Mnemonic::INX => self.reg.x.wrapping_add(0x01),
            Mnemonic::TYA => self.reg.y,
            Mnemonic::DEY => self.reg.y.wrapping_add(0xFF),
            Mnemonic::INY => self.reg.y.wrapping_add(0x01),
            x => panic!(
                "ERROR: Register update not a valid instruction for: {:?}",
                x
            ),
        };

        match &opcode.mnemonic {
            Mnemonic::TAX | Mnemonic::DEX | Mnemonic::INX => self.reg.x = value,
            Mnemonic::TAY | Mnemonic::DEY | Mnemonic::INY => self.reg.y = value,
            Mnemonic::TXA | Mnemonic::TYA => self.reg.a = value,
            x => panic!(
                "ERROR: Register update not a valid instruction for: {:?}",
                x
            ),
        };

        self.update_zn_from_value(value);
        self.increment_pc(opcode);
    }

    fn do_rotate_left(&mut self, opcode: &Opcode) {
        let new_carry = self.reg.a & 0b1000_0000 != 0;

        self.reg.a = if self.reg.get_carry() {
            self.reg.a | 0b1000_0000
        } else {
            self.reg.a & 0b0111_1111
        }
        .rotate_left(1);

        self.reg.set_carry(new_carry);
        self.update_zn_from_accumulator();
        self.increment_pc(opcode);
    }

    fn do_rotate_right(&mut self, opcode: &Opcode) {
        let new_carry = self.reg.a & 0b0000_0001 != 0;

        self.reg.a = if self.reg.get_carry() {
            self.reg.a | 0b0000_0001
        } else {
            self.reg.a & 0b1111_1110
        }
        .rotate_right(1);

        self.reg.set_carry(new_carry);
        self.update_zn_from_accumulator();
        self.increment_pc(opcode);
    }

    fn do_return_from_interrupt(&mut self, _opcode: &Opcode) {
        self.reg.p = self.pull_u8();
        self.reg.pc = self.pull_u16();
    }

    fn do_return_from_subroutine(&mut self, _opcode: &Opcode) {
        self.reg.pc = self.pull_u16().wrapping_add(1);
    }

    fn do_stack_transfer(&mut self, opcode: &Opcode) {
        match &opcode.mnemonic {
            Mnemonic::TXS => self.reg.sp = self.reg.x,
            Mnemonic::TSX => self.reg.x = self.reg.sp,
            Mnemonic::PHA => self.push_u8(self.reg.a),
            Mnemonic::PLA => self.reg.a = self.pull_u8(),
            // PHP pushes the processor word with 4 and 5 set, and PLP ignores them when pulling
            Mnemonic::PHP => self.push_u8(self.reg.p | 0b0011_0000),
            Mnemonic::PLP => self.reg.p = self.pull_u8() & !0b0011_0000,
            x => panic!("ERROR: Stack transfer not a valid instruction for: {:?}", x),
        }

        self.increment_pc(opcode);
    }

    fn do_store_register(&mut self, opcode: &Opcode) {
        let addr = self.get_operand_address(&opcode.mode);
        self.mem.write_u8(
            addr,
            match &opcode.mnemonic {
                Mnemonic::STA => self.reg.a,
                Mnemonic::STX => self.reg.x,
                Mnemonic::STY => self.reg.y,
                x => panic!("ERROR: Register store not a valid instruction for: {:?}", x),
            },
        );

        self.increment_pc(opcode);
    }

    pub fn new() -> Self {
        CPU {
            reg: RegisterSet::default(),
            mem: SimpleMap::default(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mem.read_u8(addr)
    }

    pub fn load(&mut self, addr: u16, data: &[u8]) {
        self.mem.load(addr, data);
    }

    fn get_operand_address(&self, mode: &AddressMode) -> u16 {
        use AddressMode::*;
        match mode {
            Implicit => panic!("AddressMode Error: No operand for Implicit addressing."),
            Accumulator => panic!("AddressMode Error: No operand for Accumulator addressing."), // may need to replace this
            Immediate => self.reg.pc,
            ZeroPage => self.mem.read_u8(self.reg.pc) as u16,
            ZeroPageX => self.mem.read_u8(self.reg.pc).wrapping_add(self.reg.x) as u16,
            ZeroPageY => self.mem.read_u8(self.reg.pc).wrapping_add(self.reg.y) as u16,
            Relative => self.reg.pc, // for branch instructions, there is no operand really.
            Absolute => self.mem.read_u16(self.reg.pc), // may also need to replace this
            AbsoluteX => self
                .mem
                .read_u16(self.reg.pc)
                .wrapping_add(self.reg.x as u16),
            AbsoluteY => self
                .mem
                .read_u16(self.reg.pc)
                .wrapping_add(self.reg.y as u16),
            Indirect => {
                let ptr = self.mem.read_u16(self.reg.pc);

                self.mem.read_u16(ptr)
            },
            IndirectX => {
                let ptr = self.mem.read_u8(self.reg.pc).wrapping_add(self.reg.x);

                u16::from_le_bytes([
                    self.mem.read_u8(ptr as u16),
                    self.mem.read_u8(ptr.wrapping_add(1) as u16),
                ])
            },
            IndirectY => {
                let ptr = self.mem.read_u8(self.reg.pc);

                u16::from_le_bytes([
                    self.mem.read_u8(ptr as u16),
                    self.mem.read_u8(ptr.wrapping_add(1) as u16),
                ])
                .wrapping_add(self.reg.y as u16)
            }
        }
    }

    pub fn step(&mut self, code: u8) {
        use ops::Mnemonic::*;

        self.reg.pc += 1;
        let &opcode = ops::CPU_OPCODES_MAP
            .get(&code)
            .expect(&format!("ERROR: Opcode {:#x?} unimplemented\nDump:\n {:#?}", code, self));

        match opcode.mnemonic {
            // Add or Subtract
            ADC | SBC => self.do_add_sub(opcode),
            // Bitwise AND with accumulator
            AND => self.do_and(opcode),
            // Arithmetic shift left
            ASL => self.do_arithmetic_shift(opcode),
            // Test bits
            BIT => self.do_bit_test(opcode),
            // Branch instructions
            BPL | BMI | BVC | BVS | BCC | BCS | BNE | BEQ => self.do_branch(opcode),
            BRK => self.do_break(opcode),
            CMP | CPX | CPY => self.do_compare(opcode),
            // 'crement memory
            DEC | INC => self.do_crement(opcode),
            // Bitwise exclusive OR
            EOR => self.do_bitwise_xor(opcode),
            // Flag (processor status) instructions
            CLC | SEC | CLI | SEI | CLV | CLD | SED => self.do_update_flag(opcode),
            // Jump
            JMP => self.do_jump(opcode),
            JSR => self.do_subroutine_jump(opcode),
            // Load register
            LDA | LDX | LDY => self.do_load(opcode),
            // Logical shift right
            LSR => self.do_logical_shift(opcode),
            // No operation
            NOP => self.do_nop(opcode),
            // Bitwise OR with accumulator
            ORA => self.do_bitwise_or(opcode),
            // Register instructions
            TAX | TXA | TAY | TYA | DEX | DEY | INX | INY => self.do_register_update(opcode),
            // Rotate
            ROL => self.do_rotate_left(opcode),
            ROR => self.do_rotate_right(opcode),
            // Return from interrupt
            RTI => self.do_return_from_interrupt(opcode),
            // Return from subroutine
            RTS => self.do_return_from_subroutine(opcode),
            // Store register
            STA | STX | STY => self.do_store_register(opcode),
            // Stack instructions
            TXS | TSX | PHA | PLA | PHP | PLP => self.do_stack_transfer(opcode),
        }
    }

    /// Continuously run program from current location until BRK
    pub fn run(&mut self) {
        self.run_with_callback(|_|Ok(()));
    }

    /// Reset register state and initialise program counter to value at 0xFFFC
    pub fn interrupt_reset(&mut self) {
        self.reg.reset();
        self.reg.pc = self.mem.read_u16(CPU::PRG_START_ADDR);
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.mem.load(CPU::PRG_ROM_ADDR_MIN, program);
        self.mem.write_u16(0xFFFC, CPU::PRG_ROM_ADDR_MIN);
    }


    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where F: FnMut(&mut CPU) -> Result<(), Box<dyn std::error::Error>>,
    {
        //let ref opcodes: HashMap<u8, &'static Opcode> = *ops::CPU_OPCODES_MAP;

        loop {
            if let Err(error) = callback(self) {
                panic!("ERROR in UI Callback: {:?}", error);
            }
            match self.mem.read_u8(self.reg.pc) {
                0x00 => return, // break (temporary manual check until we implement proper interrupts)
                opcode => self.step(opcode),
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
        // Set C = 1
        // 1 + 1 + C = 3
        cpu.load_program(&[0x38, 0xA9, 0x01, 0x69, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.a, 0x03);
    }

    #[test]
    fn test_set_carry() {
        let mut cpu = CPU::new();
        //  1 + 1 = 2, returns C = 0
        cpu.load_program(&[0xA9, 0x01, 0x69, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), false);

        //  1 + -1 = 0, returns C = 1
        cpu.load_program(&[0xA9, 0x01, 0x69, 0xFF, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), true);

        //  127 + 1 = 128, returns C = 0
        cpu.load_program(&[0xA9, 0x7F, 0x69, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), false);

        //  -128 + -1 = -129, returns C = 1
        cpu.load_program(&[0xA9, 0x80, 0x69, 0xFF, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_carry(), true);
    }

    #[test]
    fn test_set_overflow() {
        let mut cpu = CPU::new();
        //  1 + 1 = 2, returns V = 0
        cpu.load_program(&[0xA9, 0x01, 0x69, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);

        //  1 + -1 = 0, returns V = 0
        cpu.load_program(&[0xA9, 0x01, 0x69, 0xFF, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);

        //  127 + 1 = 128, returns V = 1
        cpu.load_program(&[0xA9, 0x7F, 0x69, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // -128 + -1 = -129, returns V = 1
        cpu.load_program(&[0xA9, 0x80, 0x69, 0xFF, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // 0 - 1 = -1, returns V = 0
        cpu.load_program(&[0xA9, 0x00, 0xE9, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), false);

        // -128 - 1 = -129, returns V = 1
        cpu.load_program(&[0xA9, 0x80, 0xE9, 0x01, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);

        // 127 - -1 = 128, returns V = 1
        cpu.load_program(&[0xA9, 0x7F, 0xE9, 0xFF, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.get_overflow(), true);
    }

    #[test]
    fn test_update_zn() {
        let mut cpu = CPU::new();
        cpu.interrupt_reset();
        cpu.reg.a = 0;
        cpu.update_zn_from_accumulator();
        assert_eq!(cpu.reg.get_zero(), true);
        assert_eq!(cpu.reg.get_negative(), false);

        cpu.interrupt_reset();
        cpu.reg.a = -1_i8 as u8;
        cpu.update_zn_from_accumulator();
        assert_eq!(cpu.reg.get_zero(), false);
        assert_eq!(cpu.reg.get_negative(), true);
    
        cpu.interrupt_reset();
        cpu.reg.a = 1;
        cpu.update_zn_from_accumulator();
        assert_eq!(cpu.reg.get_zero(), false);
        assert_eq!(cpu.reg.get_negative(), false);

        cpu.interrupt_reset();
        cpu.update_zn_from_value(0);
        assert_eq!(cpu.reg.get_zero(), true);
        assert_eq!(cpu.reg.get_negative(), false);

        cpu.interrupt_reset();
        cpu.update_zn_from_value(-1_i8 as u8);
        assert_eq!(cpu.reg.get_zero(), false);
        assert_eq!(cpu.reg.get_negative(), true);
    
        cpu.interrupt_reset();
        cpu.update_zn_from_value(1);
        assert_eq!(cpu.reg.get_zero(), false);
        assert_eq!(cpu.reg.get_negative(), false);
    }

    #[test]
    fn test_get_address() {
        let mut cpu = CPU::new();
        let address = 0x1F1F;
        cpu.load(address, &[0x10, 0x20]);
        // little endian so 0x1F1F should represent 0x10 as a u8 and 0x2010 as a u16
        assert_eq!(cpu.mem.read_u8(address), 0x10u8);
        assert_eq!(cpu.mem.read_u16(address), 0x2010u16);
    }

    #[test]
    fn test_arithmetic_shift() {
        let mut cpu = CPU::new();
        let pre_shifted_value = 0b1001_0100;
        let expected_value = 0b0010_1000;
        cpu.load_program(&[0xA9, pre_shifted_value, 0x0A, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.a, expected_value);
        assert_eq!(cpu.reg.get_carry(), true);

        let mut cpu = CPU::new();
        let pre_shifted_value = 0b0011_0111;
        let expected_value = 0b0110_1110;
        cpu.load_program(&[0xA9, pre_shifted_value, 0x0A, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.a, expected_value);
        assert_eq!(cpu.reg.get_carry(), false);
    }

    #[test]
    fn test_logical_shift() {
        let mut cpu = CPU::new();
        let pre_shifted_value = 0b1001_0100;
        let expected_value = 0b0100_1010;
        cpu.load_program(&[0xA9, pre_shifted_value, 0x4A, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.a, expected_value);
        assert_eq!(cpu.reg.get_carry(), false);

        let mut cpu = CPU::new();
        let pre_shifted_value = 0b0011_0111;
        let expected_value = 0b0001_1011;
        cpu.load_program(&[0xA9, pre_shifted_value, 0x4A, 0x00]);
        cpu.interrupt_reset();
        cpu.run();
        assert_eq!(cpu.reg.a, expected_value);
        assert_eq!(cpu.reg.get_carry(), true);
    }

    // #[test]
    // fn test_
}
