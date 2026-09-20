use anyhow::{Result, anyhow};

use crate::{
    instruction::{AddressingMode, Instruction, Mnemonic},
    memory::Memory,
};

pub const FLAG_CRY: u8 = 0b0000_0001;
pub const FLAG_ZRO: u8 = 0b0000_0010;
pub const FLAG_INT: u8 = 0b0000_0100;
pub const FLAG_DEC: u8 = 0b0000_1000;
pub const FLAG_BRK: u8 = 0b0001_0000;
pub const FLAG_IGN: u8 = 0b0010_0000;
pub const FLAG_OVF: u8 = 0b0100_0000;
pub const FLAG_NEG: u8 = 0b1000_0000;

pub const RESET_VECTOR_LO: u16 = 0xFFFC;
pub const RESET_VECTOR_HI: u16 = 0xFFFD;

pub struct CPU<M: Memory> {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub sr: u8,
    pub pc: u16,

    memory: M,
}

impl<M: Memory> CPU<M> {
    pub fn new(memory: M) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            sr: 0,
            pc: 0,

            memory,
        }
    }

    /// return: (address, page cross penalty value)
    #[inline]
    fn resolve_address(&mut self, addressing_mode: AddressingMode) -> (u16, usize) {
        match addressing_mode {
            AddressingMode::Immediate => (self.pc, 0),
            AddressingMode::ZeroPage => (self.memory.read_byte(self.pc) as u16, 0),
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
                let addr = self.memory.read_byte(self.pc);
                let index = match addressing_mode {
                    AddressingMode::ZeroPageX => self.x,
                    AddressingMode::ZeroPageY => self.y,
                    _ => unreachable!(),
                };
                (addr.wrapping_add(index) as u16, 0)
            }
            AddressingMode::Absolute => (self.memory.read_word(self.pc), 0),
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                let addr = self.memory.read_word(self.pc);
                let index = match addressing_mode {
                    AddressingMode::AbsoluteX => self.x,
                    AddressingMode::AbsoluteY => self.y,
                    _ => unreachable!(),
                };
                let addr_indexed = addr.wrapping_add(index as u16);
                let penalty = if addr & 0xFF00 != addr_indexed & 0xFF00 {
                    1
                } else {
                    0
                };

                (addr_indexed, penalty)
            }
            AddressingMode::IndirectX => {
                let zp_addr_lo = self.memory.read_byte(self.pc).wrapping_add(self.x);
                let zp_addr_hi = zp_addr_lo.wrapping_add(1);
                let addr_lo = self.memory.read_byte(zp_addr_lo as u16);
                let addr_hi = self.memory.read_byte(zp_addr_hi as u16);
                ((addr_hi as u16) << 8 | addr_lo as u16, 0)
            }
            AddressingMode::IndirectY => {
                let zp_addr_lo = self.memory.read_byte(self.pc);
                let zp_addr_hi = self.memory.read_byte(self.pc.wrapping_add(1));
                let addr_lo = self.memory.read_byte(zp_addr_lo as u16);
                let addr_hi = self.memory.read_byte(zp_addr_hi as u16);
                let addr = (addr_hi as u16) << 8 | addr_lo as u16;
                let addr_indexed = addr.wrapping_add(self.y as u16);
                let penalty = if addr & 0xFF00 != addr_indexed & 0xFF00 {
                    1
                } else {
                    0
                };

                (addr_indexed, penalty)
            }
        }
    }

    /// return: (value, page cross penalty value)
    pub fn fetch(&mut self, addressing_mode: AddressingMode) -> (u8, usize) {
        let (addr, penalty) = self.resolve_address(addressing_mode);
        match addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => self.pc = self.pc.wrapping_add(1),
            AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                self.pc = self.pc.wrapping_add(2)
            }
        };
        (self.memory.read_byte(addr), penalty)
    }

    pub fn step(&mut self) -> Result<()> {
        let opcode = self.memory.read_byte(self.pc);
        println!("{opcode}");
        self.pc = self.pc.wrapping_add(1);

        if let Some(instruction) = Instruction::from_opcode(opcode) {
            println!("Executing: {instruction:?}...");
            self.exec(instruction);
            Ok(())
        } else {
            Err(anyhow!("Invalid or unimplemented opcode: {opcode}"))
        }
    }

    /// return: cycles this instruction took
    pub fn exec(&mut self, instruction: Instruction) -> usize {
        match instruction.0 {
            Mnemonic::ADC | Mnemonic::SBC => self.adc_sbc(instruction) + instruction.cycles(),
            Mnemonic::CPX | Mnemonic::CPY | Mnemonic::CMP => {
                self.compare(instruction) + instruction.cycles()
            }
            Mnemonic::AND | Mnemonic::EOR | Mnemonic::ORA => {
                self.logic(instruction) + instruction.cycles()
            }
        }
    }

    /// return: page cross penalty
    #[inline]
    fn adc_sbc(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.0 == Mnemonic::ADC || instruction.0 == Mnemonic::SBC,
            "Invalid Instruction given to adc_sbc: {instruction:?}",
        );

        let operand_1 = self.a;

        let (operand_2, penalty) = self.fetch(instruction.1);
        let operand_2 = if instruction.0 == Mnemonic::SBC {
            (!operand_2).wrapping_add(1)
        } else {
            operand_2
        };

        let carry_val = if self.flag_get_carry() { 1 } else { 0 };
        let result = operand_1.wrapping_add(operand_2).wrapping_add(carry_val);
        self.a = result;

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || (operand_2 == 0xFF && carry_val == 1));
        self.flag_set_overflow(operand_1, operand_2, result);
        penalty
    }

    #[inline]
    fn compare(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.0 == Mnemonic::CPX
                || instruction.0 == Mnemonic::CPY
                || instruction.0 == Mnemonic::CMP,
            "Invalid Instruction given to compare: {instruction:?}",
        );

        let operand_1 = match instruction.0 {
            Mnemonic::CPX => self.x,
            Mnemonic::CPY => self.y,
            _ => self.a,
        };
        let (operand_2, penalty) = self.fetch(instruction.1);
        let operand_2 = (!operand_2).wrapping_add(1);

        let result = operand_1.wrapping_add(operand_2).wrapping_add(1);

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || operand_2 == 0xFF);

        penalty
    }

    #[inline]
    fn logic(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.0 == Mnemonic::AND
                || instruction.0 == Mnemonic::EOR
                || instruction.0 == Mnemonic::ORA,
            "Invalid Instruction given to logic: {instruction:?}"
        );

        let operand_1 = self.a;
        let (operand_2, penalty) = self.fetch(instruction.1);

        let result = match instruction.0 {
            Mnemonic::AND => operand_1 & operand_2,
            Mnemonic::EOR => operand_1 ^ operand_2,
            _ => operand_1 | operand_2,
        };
        self.a = result;

        self.flag_set_zero(result);
        self.flag_set_negative(result);

        penalty
    }

    fn flag_get_carry(&self) -> bool {
        (self.sr & FLAG_CRY) != 0
    }

    fn flag_set_zero(&mut self, result: u8) {
        if result == 0 {
            self.sr |= FLAG_ZRO
        } else {
            self.sr &= !FLAG_ZRO
        }
    }

    fn flag_set_negative(&mut self, result: u8) {
        if result & 0x80 != 0 {
            self.sr |= FLAG_NEG
        } else {
            self.sr &= !FLAG_NEG
        }
    }

    fn flag_set_carry(&mut self, carry: bool) {
        if carry {
            self.sr |= FLAG_CRY
        } else {
            self.sr &= !FLAG_CRY
        }
    }

    fn flag_set_overflow(&mut self, a: u8, b: u8, result: u8) {
        if (a ^ result) & (b ^ result) & 0x80 != 0 {
            self.sr |= FLAG_OVF
        } else {
            self.sr &= !FLAG_OVF
        }
    }
}
