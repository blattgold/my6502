#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    /// Implied and Accumulator
    Implied,
    /// Immediate and Relative
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
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
    // load
    LDA,
    LDX,
    LDY,
    // transfer
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    // shift & rotate
    ASL,
    LSR,
    ROL,
    ROR,
    // branch
    BEQ,
    BMI,
    BCS,
    BVS,
    BNE,
    BCC,
    BPL,
    BVC,
    // JUMP
    JMP,
    JSR,
    RTS,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction(Mnemonic, AddressingMode);

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
            // LDA
            0xa9 => Some(Instruction(Mnemonic::LDA, AddressingMode::Immediate)),
            0xa5 => Some(Instruction(Mnemonic::LDA, AddressingMode::ZeroPage)),
            0xb5 => Some(Instruction(Mnemonic::LDA, AddressingMode::ZeroPageX)),
            0xad => Some(Instruction(Mnemonic::LDA, AddressingMode::Absolute)),
            0xbd => Some(Instruction(Mnemonic::LDA, AddressingMode::AbsoluteX)),
            0xb9 => Some(Instruction(Mnemonic::LDA, AddressingMode::AbsoluteY)),
            0xa1 => Some(Instruction(Mnemonic::LDA, AddressingMode::IndirectX)),
            0xb1 => Some(Instruction(Mnemonic::LDA, AddressingMode::IndirectY)),
            // LDX
            0xa2 => Some(Instruction(Mnemonic::LDX, AddressingMode::Immediate)),
            0xa6 => Some(Instruction(Mnemonic::LDX, AddressingMode::ZeroPage)),
            0xb6 => Some(Instruction(Mnemonic::LDX, AddressingMode::ZeroPageY)),
            0xae => Some(Instruction(Mnemonic::LDX, AddressingMode::Absolute)),
            0xbe => Some(Instruction(Mnemonic::LDX, AddressingMode::AbsoluteY)),
            // LDY
            0xa0 => Some(Instruction(Mnemonic::LDY, AddressingMode::Immediate)),
            0xa4 => Some(Instruction(Mnemonic::LDY, AddressingMode::ZeroPage)),
            0xb4 => Some(Instruction(Mnemonic::LDY, AddressingMode::ZeroPageX)),
            0xac => Some(Instruction(Mnemonic::LDY, AddressingMode::Absolute)),
            0xbc => Some(Instruction(Mnemonic::LDY, AddressingMode::AbsoluteX)),
            // TRANSFER
            0xaa => Some(Instruction(Mnemonic::TAX, AddressingMode::Implied)),
            0xa8 => Some(Instruction(Mnemonic::TAY, AddressingMode::Implied)),
            0xba => Some(Instruction(Mnemonic::TSX, AddressingMode::Implied)),
            0x8a => Some(Instruction(Mnemonic::TXA, AddressingMode::Implied)),
            0x9a => Some(Instruction(Mnemonic::TXS, AddressingMode::Implied)),
            0x98 => Some(Instruction(Mnemonic::TYA, AddressingMode::Implied)),
            // ASL
            0x0a => Some(Instruction(Mnemonic::ASL, AddressingMode::Implied)),
            0x06 => Some(Instruction(Mnemonic::ASL, AddressingMode::ZeroPage)),
            0x16 => Some(Instruction(Mnemonic::ASL, AddressingMode::ZeroPageX)),
            0x0e => Some(Instruction(Mnemonic::ASL, AddressingMode::Absolute)),
            0x1e => Some(Instruction(Mnemonic::ASL, AddressingMode::AbsoluteX)),
            // LSR
            0x4a => Some(Instruction(Mnemonic::LSR, AddressingMode::Implied)),
            0x46 => Some(Instruction(Mnemonic::LSR, AddressingMode::ZeroPage)),
            0x56 => Some(Instruction(Mnemonic::LSR, AddressingMode::ZeroPageX)),
            0x4e => Some(Instruction(Mnemonic::LSR, AddressingMode::Absolute)),
            0x5e => Some(Instruction(Mnemonic::LSR, AddressingMode::AbsoluteX)),
            // ROL
            0x2a => Some(Instruction(Mnemonic::ROL, AddressingMode::Implied)),
            0x26 => Some(Instruction(Mnemonic::ROL, AddressingMode::ZeroPage)),
            0x36 => Some(Instruction(Mnemonic::ROL, AddressingMode::ZeroPageX)),
            0x2e => Some(Instruction(Mnemonic::ROL, AddressingMode::Absolute)),
            0x3e => Some(Instruction(Mnemonic::ROL, AddressingMode::AbsoluteX)),
            // ROR
            0x6a => Some(Instruction(Mnemonic::ROR, AddressingMode::Implied)),
            0x66 => Some(Instruction(Mnemonic::ROR, AddressingMode::ZeroPage)),
            0x76 => Some(Instruction(Mnemonic::ROR, AddressingMode::ZeroPageX)),
            0x6e => Some(Instruction(Mnemonic::ROR, AddressingMode::Absolute)),
            0x7e => Some(Instruction(Mnemonic::ROR, AddressingMode::AbsoluteX)),
            // BRANCHING
            0xf0 => Some(Instruction(Mnemonic::BEQ, AddressingMode::Immediate)),
            0xd0 => Some(Instruction(Mnemonic::BNE, AddressingMode::Immediate)),
            0xb0 => Some(Instruction(Mnemonic::BCS, AddressingMode::Immediate)),
            0x90 => Some(Instruction(Mnemonic::BCC, AddressingMode::Immediate)),
            0x30 => Some(Instruction(Mnemonic::BMI, AddressingMode::Immediate)),
            0x10 => Some(Instruction(Mnemonic::BPL, AddressingMode::Immediate)),
            0x70 => Some(Instruction(Mnemonic::BVS, AddressingMode::Immediate)),
            0x50 => Some(Instruction(Mnemonic::BVC, AddressingMode::Immediate)),
            // JMP
            0x4c => Some(Instruction(Mnemonic::JMP, AddressingMode::Absolute)),
            0x6c => Some(Instruction(Mnemonic::JMP, AddressingMode::Indirect)),
            // JSR
            0x20 => Some(Instruction(Mnemonic::JSR, AddressingMode::Absolute)),
            // RTS
            0x60 => Some(Instruction(Mnemonic::RTS, AddressingMode::Implied)),
            _ => None,
        }
    }

    #[inline]
    pub fn cycles(self) -> usize {
        match self {
            // most instructions share cycle counts
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
            | Instruction(Mnemonic::STY, addressing_mode)
            | Instruction(Mnemonic::LDA, addressing_mode)
            | Instruction(Mnemonic::LDX, addressing_mode)
            | Instruction(Mnemonic::LDY, addressing_mode)
            | Instruction(Mnemonic::TAX, addressing_mode)
            | Instruction(Mnemonic::TAY, addressing_mode)
            | Instruction(Mnemonic::TSX, addressing_mode)
            | Instruction(Mnemonic::TXA, addressing_mode)
            | Instruction(Mnemonic::TXS, addressing_mode)
            | Instruction(Mnemonic::TYA, addressing_mode)
            | Instruction(Mnemonic::BEQ, addressing_mode)
            | Instruction(Mnemonic::BMI, addressing_mode)
            | Instruction(Mnemonic::BCS, addressing_mode)
            | Instruction(Mnemonic::BVS, addressing_mode)
            | Instruction(Mnemonic::BNE, addressing_mode)
            | Instruction(Mnemonic::BCC, addressing_mode)
            | Instruction(Mnemonic::BPL, addressing_mode)
            | Instruction(Mnemonic::BVC, addressing_mode) => match addressing_mode {
                AddressingMode::Immediate | AddressingMode::Implied => 2,
                AddressingMode::ZeroPage => 3,
                AddressingMode::ZeroPageX
                | AddressingMode::ZeroPageY
                | AddressingMode::Absolute
                | AddressingMode::AbsoluteX
                | AddressingMode::AbsoluteY => 4,
                AddressingMode::IndirectX => 6,
                AddressingMode::IndirectY => 5,

                _ => panic!(
                    "Main Cycle Group should not have this addressing mode: {addressing_mode:?}"
                ),
            },
            // shifts and rotates
            Instruction(Mnemonic::ASL, addressing_mode)
            | Instruction(Mnemonic::LSR, addressing_mode)
            | Instruction(Mnemonic::ROL, addressing_mode)
            | Instruction(Mnemonic::ROR, addressing_mode) => match addressing_mode {
                AddressingMode::Implied => 2,
                AddressingMode::ZeroPage => 5,
                AddressingMode::ZeroPageX | AddressingMode::Absolute => 6,
                AddressingMode::AbsoluteX => 7,

                _ => {
                    panic!(
                        "Shifts & Rotates should not have this addressing mode: {addressing_mode:?}"
                    )
                }
            },
            // JMP
            Instruction(Mnemonic::JMP, addressing_mode) => match addressing_mode {
                AddressingMode::Absolute => 3,
                AddressingMode::Indirect => 5,

                _ => panic!(
                    "JMP Cycle Group should not have this addressing mode: {addressing_mode:?}"
                ),
            },
            // JSR
            Instruction(Mnemonic::JSR, addressing_mode) => match addressing_mode {
                AddressingMode::Absolute => 6,

                _ => panic!(
                    "JSR Cycle Group should not have this addressing mode: {addressing_mode:?}"
                ),
            },
            // RTS
            Instruction(Mnemonic::RTS, addressing_mode) => match addressing_mode {
                AddressingMode::Implied => 6,

                _ => panic!(
                    "RTS Cycle Group should not have this addressing mode: {addressing_mode:?}"
                ),
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
