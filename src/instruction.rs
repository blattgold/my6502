use crate::instruction::AddressingMode::IndirectX;

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
    // arithmetic
    ADC,
    SBC,
    // compare
    CPX,
    CPY,
    CMP,
    // logic
    AND,
    EOR,
    ORA,
    // bit test
    BIT,
    // store
    STA,
    STX,
    STY,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction(pub Mnemonic, pub AddressingMode);

impl Instruction {
    #[inline]
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
            // CMP
            0xc9 => Some(Instruction(Mnemonic::CMP, AddressingMode::Immediate)),
            0xc5 => Some(Instruction(Mnemonic::CMP, AddressingMode::ZeroPage)),
            0xd5 => Some(Instruction(Mnemonic::CMP, AddressingMode::ZeroPageX)),
            0xcd => Some(Instruction(Mnemonic::CMP, AddressingMode::Absolute)),
            0xdd => Some(Instruction(Mnemonic::CMP, AddressingMode::AbsoluteX)),
            0xd9 => Some(Instruction(Mnemonic::CMP, AddressingMode::AbsoluteY)),
            0xc1 => Some(Instruction(Mnemonic::CMP, AddressingMode::IndirectX)),
            0xd1 => Some(Instruction(Mnemonic::CMP, AddressingMode::IndirectY)),
            // CPX
            0xe0 => Some(Instruction(Mnemonic::CPX, AddressingMode::Immediate)),
            0xe4 => Some(Instruction(Mnemonic::CPX, AddressingMode::ZeroPage)),
            0xec => Some(Instruction(Mnemonic::CPX, AddressingMode::Absolute)),
            // CPY
            0xc0 => Some(Instruction(Mnemonic::CPY, AddressingMode::Immediate)),
            0xc4 => Some(Instruction(Mnemonic::CPY, AddressingMode::ZeroPage)),
            0xcc => Some(Instruction(Mnemonic::CPY, AddressingMode::Absolute)),
            // AND
            0x29 => Some(Instruction(Mnemonic::AND, AddressingMode::Immediate)),
            0x25 => Some(Instruction(Mnemonic::AND, AddressingMode::ZeroPage)),
            0x35 => Some(Instruction(Mnemonic::AND, AddressingMode::ZeroPageX)),
            0x2d => Some(Instruction(Mnemonic::AND, AddressingMode::Absolute)),
            0x3d => Some(Instruction(Mnemonic::AND, AddressingMode::AbsoluteX)),
            0x39 => Some(Instruction(Mnemonic::AND, AddressingMode::AbsoluteY)),
            0x21 => Some(Instruction(Mnemonic::AND, AddressingMode::IndirectX)),
            0x31 => Some(Instruction(Mnemonic::AND, AddressingMode::IndirectY)),
            // EOR
            0x49 => Some(Instruction(Mnemonic::EOR, AddressingMode::Immediate)),
            0x45 => Some(Instruction(Mnemonic::EOR, AddressingMode::ZeroPage)),
            0x55 => Some(Instruction(Mnemonic::EOR, AddressingMode::ZeroPageX)),
            0x4d => Some(Instruction(Mnemonic::EOR, AddressingMode::Absolute)),
            0x5d => Some(Instruction(Mnemonic::EOR, AddressingMode::AbsoluteX)),
            0x59 => Some(Instruction(Mnemonic::EOR, AddressingMode::AbsoluteY)),
            0x41 => Some(Instruction(Mnemonic::EOR, AddressingMode::IndirectX)),
            0x51 => Some(Instruction(Mnemonic::EOR, AddressingMode::IndirectY)),
            // ORA
            0x09 => Some(Instruction(Mnemonic::ORA, AddressingMode::Immediate)),
            0x05 => Some(Instruction(Mnemonic::ORA, AddressingMode::ZeroPage)),
            0x15 => Some(Instruction(Mnemonic::ORA, AddressingMode::ZeroPageX)),
            0x0d => Some(Instruction(Mnemonic::ORA, AddressingMode::Absolute)),
            0x1d => Some(Instruction(Mnemonic::ORA, AddressingMode::AbsoluteX)),
            0x19 => Some(Instruction(Mnemonic::ORA, AddressingMode::AbsoluteY)),
            0x01 => Some(Instruction(Mnemonic::ORA, AddressingMode::IndirectX)),
            0x11 => Some(Instruction(Mnemonic::ORA, AddressingMode::IndirectY)),
            // BIT
            0x24 => Some(Instruction(Mnemonic::BIT, AddressingMode::ZeroPage)),
            0x2c => Some(Instruction(Mnemonic::BIT, AddressingMode::Absolute)),
            // STA
            0x85 => Some(Instruction(Mnemonic::STA, AddressingMode::ZeroPage)),
            0x95 => Some(Instruction(Mnemonic::STA, AddressingMode::ZeroPageX)),
            0x8d => Some(Instruction(Mnemonic::STA, AddressingMode::Absolute)),
            0x9d => Some(Instruction(Mnemonic::STA, AddressingMode::AbsoluteX)),
            0x99 => Some(Instruction(Mnemonic::STA, AddressingMode::AbsoluteY)),
            0x81 => Some(Instruction(Mnemonic::STA, AddressingMode::IndirectX)),
            0x91 => Some(Instruction(Mnemonic::STA, AddressingMode::IndirectY)),
            // STX
            0x86 => Some(Instruction(Mnemonic::STX, AddressingMode::ZeroPage)),
            0x96 => Some(Instruction(Mnemonic::STX, AddressingMode::ZeroPageY)),
            0x8e => Some(Instruction(Mnemonic::STX, AddressingMode::Absolute)),
            // STY
            0x84 => Some(Instruction(Mnemonic::STY, AddressingMode::ZeroPage)),
            0x94 => Some(Instruction(Mnemonic::STY, AddressingMode::ZeroPageX)),
            0x8c => Some(Instruction(Mnemonic::STY, AddressingMode::Absolute)),
            _ => None,
        }
    }

    #[inline]
    pub fn cycles(self) -> usize {
        match self {
            Instruction(Mnemonic::ADC, addressing_mode)
            | Instruction(Mnemonic::SBC, addressing_mode)
            | Instruction(Mnemonic::CPX, addressing_mode)
            | Instruction(Mnemonic::CPY, addressing_mode)
            | Instruction(Mnemonic::CMP, addressing_mode)
            | Instruction(Mnemonic::AND, addressing_mode)
            | Instruction(Mnemonic::EOR, addressing_mode)
            | Instruction(Mnemonic::ORA, addressing_mode)
            | Instruction(Mnemonic::BIT, addressing_mode)
            | Instruction(Mnemonic::STA, addressing_mode)
            | Instruction(Mnemonic::STX, addressing_mode)
            | Instruction(Mnemonic::STY, addressing_mode) => match addressing_mode {
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

    #[inline]
    pub fn mode(self) -> AddressingMode {
        self.1
    }

    #[inline]
    pub fn mnemonic(self) -> Mnemonic {
        self.0
    }
}
