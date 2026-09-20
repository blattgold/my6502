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
    CPX,
    CPY,
    CMP,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction(pub Mnemonic, pub AddressingMode);

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
            _ => None,
        }
    }

    pub fn cycles(self) -> usize {
        match self {
            Instruction(Mnemonic::ADC, addressing_mode)
            | Instruction(Mnemonic::SBC, addressing_mode)
            | Instruction(Mnemonic::CPX, addressing_mode)
            | Instruction(Mnemonic::CPY, addressing_mode)
            | Instruction(Mnemonic::CMP, addressing_mode) => match addressing_mode {
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
}
