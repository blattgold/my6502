use crate::{
    cpu::{CPU, FLAG_CRY, FLAG_NEG, FLAG_OVF, FLAG_ZRO},
    memory::Memory,
};

#[derive(Clone, Copy, Debug)]
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mnemonic {
    ADC,
    SBC,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction(pub Mnemonic, pub AddressingMode);

fn flag_get_carry(sr: &u8) -> bool {
    (*sr & FLAG_CRY) != 0
}

fn flag_set_zero(sr: &mut u8, result: u8) {
    if result == 0 {
        *sr |= FLAG_ZRO
    } else {
        *sr &= !FLAG_ZRO
    }
}

fn flag_set_negative(sr: &mut u8, result: u8) {
    if result & 0x80 != 0 {
        *sr |= FLAG_NEG
    } else {
        *sr &= !FLAG_NEG
    }
}

fn flag_set_carry(sr: &mut u8, carry: bool) {
    if carry {
        *sr |= FLAG_CRY
    } else {
        *sr &= !FLAG_CRY
    }
}

fn flag_set_overflow(sr: &mut u8, a: u8, b: u8, result: u8) {
    if (a ^ result) & (b ^ result) & 0x80 != 0 {
        *sr |= FLAG_OVF
    } else {
        *sr &= !FLAG_OVF
    }
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Option<Self> {
        match opcode {
            // ADC
            0x69 => Some(Instruction(Mnemonic::ADC, AddressingMode::Immediate)),
            0x65 => Some(Instruction(Mnemonic::ADC, AddressingMode::ZeroPage)),
            0x75 => Some(Instruction(Mnemonic::ADC, AddressingMode::ZeroPageX)),
            0x6d => Some(Instruction(Mnemonic::ADC, AddressingMode::Absolute)),
            0x7d => Some(Instruction(Mnemonic::ADC, AddressingMode::AbsoluteX)),
            0x79 => Some(Instruction(Mnemonic::ADC, AddressingMode::AbsoluteY)),
            0x61 => Some(Instruction(Mnemonic::ADC, AddressingMode::IndirectX)),
            0x71 => Some(Instruction(Mnemonic::ADC, AddressingMode::IndirectY)),
            // SBC
            0xe9 => Some(Instruction(Mnemonic::SBC, AddressingMode::Immediate)),
            0xe5 => Some(Instruction(Mnemonic::SBC, AddressingMode::ZeroPage)),
            0xf5 => Some(Instruction(Mnemonic::SBC, AddressingMode::ZeroPageX)),
            0xed => Some(Instruction(Mnemonic::SBC, AddressingMode::Absolute)),
            0xfd => Some(Instruction(Mnemonic::SBC, AddressingMode::AbsoluteX)),
            0xf9 => Some(Instruction(Mnemonic::SBC, AddressingMode::AbsoluteY)),
            0xe1 => Some(Instruction(Mnemonic::SBC, AddressingMode::IndirectX)),
            0xf1 => Some(Instruction(Mnemonic::SBC, AddressingMode::IndirectY)),
            _ => None,
        }
    }

    fn cycles(self) -> usize {
        match self {
            Instruction(Mnemonic::ADC, addressing_mode)
            | Instruction(Mnemonic::SBC, addressing_mode) => match addressing_mode {
                AddressingMode::Immediate => 2,
                AddressingMode::ZeroPage => 3,
                AddressingMode::ZeroPageX
                | AddressingMode::ZeroPageY
                | AddressingMode::Absolute
                | AddressingMode::AbsoluteX
                | AddressingMode::AbsoluteY => 4,
                AddressingMode::IndirectX => 6,
                AddressingMode::IndirectY => 5,
            },
        }
    }

    /// return: cycles this instruction took
    pub fn exec<M: Memory>(self, cpu: &mut CPU<M>) -> usize {
        match self {
            Instruction(Mnemonic::ADC, _) | Instruction(Mnemonic::SBC, _) => {
                self.adc_sbc(cpu) + self.cycles()
            }
        }
    }

    /// return: page cross penalty
    #[inline]
    fn adc_sbc<M: Memory>(self, cpu: &mut CPU<M>) -> usize {
        let operand_1 = cpu.a;

        let (operand_2, penalty) = cpu.fetch(self.1);
        let operand_2 = if self.0 == Mnemonic::SBC {
            (!operand_2) + 1
        } else {
            operand_2
        };

        let result = operand_1
            .wrapping_add(operand_2)
            .wrapping_add(if flag_get_carry(&cpu.sr) { 1 } else { 0 });
        cpu.a = result;

        flag_set_zero(&mut cpu.sr, result);
        flag_set_negative(&mut cpu.sr, result);
        flag_set_carry(&mut cpu.sr, result < operand_1);
        flag_set_overflow(&mut cpu.sr, operand_1, operand_2, result);
        penalty
    }
}
