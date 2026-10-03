use anyhow::{Result, anyhow};

use crate::{
    instruction::{self, AddressingMode, Instruction, Mnemonic},
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
    #[inline]
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

    /// return: penalty
    #[inline]
    pub fn write(&mut self, addressing_mode: AddressingMode, byte: u8) -> usize {
        let (addr, _) = self.resolve_address(addressing_mode);
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
        self.memory.write_byte(addr, byte);

        // penalty always taken when one is possible
        match addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Absolute
            | AddressingMode::IndirectX => 0,
            AddressingMode::AbsoluteY | AddressingMode::AbsoluteX | AddressingMode::IndirectY => 1,
        }
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
    #[inline]
    fn exec(&mut self, instruction: Instruction) -> usize {
        match instruction.mnemonic() {
            Mnemonic::ADC | Mnemonic::SBC => self.adc_sbc(instruction),
            Mnemonic::CPX | Mnemonic::CPY | Mnemonic::CMP => self.compare(instruction),
            Mnemonic::AND | Mnemonic::EOR | Mnemonic::ORA => self.logic(instruction),
            Mnemonic::BIT => self.bit(instruction),
            Mnemonic::STA | Mnemonic::STX | Mnemonic::STY => self.store(instruction),
            Mnemonic::LDA | Mnemonic::LDX | Mnemonic::LDY => self.load(instruction),
        }
    }

    /// return: cycles taken
    #[inline]
    fn adc_sbc(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.mnemonic() == Mnemonic::ADC || instruction.mnemonic() == Mnemonic::SBC,
            "Invalid Instruction given to adc_sbc: {instruction:?}",
        );

        let operand_1 = self.a;

        let (operand_2, penalty) = self.fetch(instruction.mode());
        let operand_2 = match instruction.mnemonic() {
            Mnemonic::ADC => operand_2,
            Mnemonic::SBC => (!operand_2).wrapping_add(1),
            _ => unreachable!(),
        };

        let carry_val = if self.flag_get_carry() { 1 } else { 0 };
        let result = operand_1.wrapping_add(operand_2).wrapping_add(carry_val);
        self.a = result;

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || (operand_2 == 0xFF && carry_val == 1));
        self.flag_set_overflow(operand_1, operand_2, result);
        penalty + instruction.cycles()
    }

    #[inline]
    fn compare(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.mnemonic() == Mnemonic::CPX
                || instruction.mnemonic() == Mnemonic::CPY
                || instruction.mnemonic() == Mnemonic::CMP,
            "Invalid Instruction given to compare: {instruction:?}",
        );

        let operand_1 = match instruction.mnemonic() {
            Mnemonic::CPX => self.x,
            Mnemonic::CPY => self.y,
            Mnemonic::CMP => self.a,
            _ => unreachable!(),
        };
        let (operand_2, penalty) = self.fetch(instruction.mode());
        let operand_2 = (!operand_2).wrapping_add(1);

        let result = operand_1.wrapping_add(operand_2).wrapping_add(1);

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || operand_2 == 0xFF);

        penalty + instruction.cycles()
    }

    #[inline]
    fn logic(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            instruction.mnemonic() == Mnemonic::AND
                || instruction.mnemonic() == Mnemonic::EOR
                || instruction.mnemonic() == Mnemonic::ORA,
            "Invalid Instruction given to logic: {instruction:?}"
        );

        let operand_1 = self.a;
        let (operand_2, penalty) = self.fetch(instruction.mode());

        let result = match instruction.mnemonic() {
            Mnemonic::AND => operand_1 & operand_2,
            Mnemonic::EOR => operand_1 ^ operand_2,
            Mnemonic::ORA => operand_1 | operand_2,
            _ => unreachable!(),
        };

        self.a = result;
        self.flag_set_zero(result);
        self.flag_set_negative(result);

        penalty + instruction.cycles()
    }

    #[inline]
    fn bit(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::BIT),
            "Invalid Instruction given to bit: {instruction:?}"
        );

        let operand_1 = self.a;
        let (operand_2, penalty) = self.fetch(instruction.mode());

        self.flag_set_zero(operand_1 & operand_2);
        self.sr &= 0b0011_1111;
        self.sr |= operand_2 & 0b1100_0000;

        penalty + instruction.cycles()
    }

    #[inline]
    fn store(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::STA | Mnemonic::STX | Mnemonic::STY
            ),
            "Invalid Instruction given to store: {instruction:?}"
        );

        let operand = match instruction.mnemonic() {
            Mnemonic::STA => self.a,
            Mnemonic::STX => self.x,
            Mnemonic::STY => self.y,
            _ => unreachable!(),
        };

        self.write(instruction.mode(), operand) + instruction.cycles()
    }

    #[inline]
    fn load(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::LDA | Mnemonic::LDX | Mnemonic::LDY
            ),
            "Invalid Instruction given to load: {instruction:?}"
        );

        let (byte, penalty) = self.fetch(instruction.mode());
        match instruction.mnemonic() {
            Mnemonic::LDA => self.a = byte,
            Mnemonic::LDX => self.x = byte,
            Mnemonic::LDY => self.y = byte,
            _ => unreachable!(),
        };

        self.flag_set_negative(byte);
        self.flag_set_zero(byte);

        penalty + instruction.cycles()
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
