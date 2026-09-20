use anyhow::{Result, anyhow};

use crate::{
    instruction::{AddressingMode, Instruction},
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
            instruction.exec(self);
            Ok(())
        } else {
            Err(anyhow!("Invalid or unimplemented opcode: {opcode}"))
        }
    }
}
