use anyhow::{Result, anyhow};

use crate::{
    instruction::{AddressingMode, CycleProfile, Instruction, Mnemonic},
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

pub const RESET_VECTOR_LO: u16 = 0xfffc;
pub const RESET_VECTOR_HI: u16 = 0xfffd;

pub const IRQ_VECTOR_LO: u16 = 0xfffe;
pub const IRQ_VECTOR_HI: u16 = 0xffff;

pub const STACK_OFFSET: u16 = 256 * 1;

pub struct ResolvedAddress {
    addr: u16,
    bytes_read: u16,
    penalty: usize,
}

/// the state is set to JAM if one of the JAM opcodes is run.
pub enum CPUState {
    Reset,
    Running,
    Jam,
}

/// reproduces (almost) all quirks of the original NMOS 6502
pub struct CPU<M: Memory> {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub sr: u8,
    pub pc: u16,

    pub state: CPUState,
    pub cycles: usize,
    pub memory: M,
}

impl<M: Memory> CPU<M> {
    pub fn new(memory: M, skip_reset: bool) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            sr: 0,
            pc: 0,

            state: if skip_reset {
                CPUState::Running
            } else {
                CPUState::Reset
            },
            cycles: 0,
            memory,
        }
    }

    /// returns the resolved address, as well as cycle penalty.
    /// the penalty should only be applied to instructions which only read from memory once and don't write
    #[inline(always)]
    fn resolve_address(&mut self, addressing_mode: AddressingMode) -> ResolvedAddress {
        debug_assert!(
            !matches!(addressing_mode, AddressingMode::Implied),
            "invalid AddressingMode given to resolve_address: {addressing_mode:?}"
        );

        let resolved = match addressing_mode {
            AddressingMode::Immediate => ResolvedAddress {
                addr: self.pc,
                bytes_read: 1,
                penalty: 0,
            },
            AddressingMode::ZeroPage => ResolvedAddress {
                addr: self.memory.read_byte(self.pc) as u16,
                bytes_read: 1,
                penalty: 0,
            },
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
                let addr = self.memory.read_byte(self.pc);
                let index = match addressing_mode {
                    AddressingMode::ZeroPageX => self.x,
                    AddressingMode::ZeroPageY => self.y,
                    _ => unreachable!(),
                };

                ResolvedAddress {
                    addr: addr.wrapping_add(index) as u16,
                    bytes_read: 1,
                    penalty: 0,
                }
            }
            AddressingMode::Absolute => ResolvedAddress {
                addr: self.memory.read_word(self.pc),
                bytes_read: 2,
                penalty: 0,
            },
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

                ResolvedAddress {
                    addr: addr_indexed,
                    bytes_read: 2,
                    penalty,
                }
            }
            AddressingMode::IndirectX => {
                let zp_addr_lo = self.memory.read_byte(self.pc).wrapping_add(self.x);
                let zp_addr_hi = zp_addr_lo.wrapping_add(1);
                let addr_lo = self.memory.read_byte(zp_addr_lo as u16);
                let addr_hi = self.memory.read_byte(zp_addr_hi as u16);
                let addr = (addr_hi as u16) << 8 | addr_lo as u16;

                ResolvedAddress {
                    addr,
                    bytes_read: 1,
                    penalty: 0,
                }
            }
            AddressingMode::IndirectY => {
                let zp_addr_lo = self.memory.read_byte(self.pc);
                let zp_addr_hi = zp_addr_lo.wrapping_add(1);
                let addr_lo = self.memory.read_byte(zp_addr_lo as u16);
                let addr_hi = self.memory.read_byte(zp_addr_hi as u16);
                let addr = ((addr_hi as u16) << 8) | (addr_lo as u16);
                let addr_indexed = addr.wrapping_add(self.y as u16);
                let penalty = if addr & 0xFF00 != addr_indexed & 0xFF00 {
                    1
                } else {
                    0
                };

                ResolvedAddress {
                    addr: addr_indexed,
                    bytes_read: 1,
                    penalty,
                }
            }
            AddressingMode::Indirect => {
                let lookup_lo = self.memory.read_byte(self.pc);
                let lookup_hi = self.memory.read_byte(self.pc.wrapping_add(1));

                let lookup_addr_1 = (lookup_hi as u16) << 8 | lookup_lo as u16;
                let lookup_addr_2 = (lookup_hi as u16) << 8 | lookup_lo.wrapping_add(1) as u16;

                let addr_lo = self.memory.read_byte(lookup_addr_1);
                let addr_hi = self.memory.read_byte(lookup_addr_2);
                let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

                ResolvedAddress {
                    addr,
                    bytes_read: 2,
                    penalty: 0,
                }
            }
            AddressingMode::Implied => unreachable!(),
        };

        self.pc = self.pc.wrapping_add(resolved.bytes_read);

        resolved
    }

    /// return: (address, page cross penalty value)
    /// TODO: migrate everything to resolve_address
    #[inline]
    fn resolve_address_old(&mut self, addressing_mode: AddressingMode) -> (u16, usize) {
        debug_assert!(
            !matches!(addressing_mode, AddressingMode::Implied),
            "invalid AddressingMode given to resolve_address: {addressing_mode:?}"
        );

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
                let zp_addr_hi = zp_addr_lo.wrapping_add(1);
                let addr_lo = self.memory.read_byte(zp_addr_lo as u16);
                let addr_hi = self.memory.read_byte(zp_addr_hi as u16);
                let addr = ((addr_hi as u16) << 8) | (addr_lo as u16);
                let addr_indexed = addr.wrapping_add(self.y as u16);
                let penalty = if addr & 0xFF00 != addr_indexed & 0xFF00 {
                    1
                } else {
                    0
                };

                (addr_indexed, penalty)
            }
            AddressingMode::Indirect => {
                let lookup_lo = self.memory.read_byte(self.pc);
                let lookup_hi = self.memory.read_byte(self.pc.wrapping_add(1));

                let lookup_addr_1 = (lookup_hi as u16) << 8 | lookup_lo as u16;
                let lookup_addr_2 = (lookup_hi as u16) << 8 | lookup_lo.wrapping_add(1) as u16;

                let addr_lo = self.memory.read_byte(lookup_addr_1);
                let addr_hi = self.memory.read_byte(lookup_addr_2);
                let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

                (addr, 0)
            }
            AddressingMode::Implied => unreachable!(),
        }
    }

    /// return: (value, page cross penalty value)
    #[inline]
    pub fn fetch(&mut self, addressing_mode: AddressingMode) -> (u8, usize) {
        debug_assert!(
            !matches!(addressing_mode, AddressingMode::Implied),
            "invalid AddressingMode given to fetch: {addressing_mode:?}"
        );

        let (addr, penalty) = self.resolve_address_old(addressing_mode);
        match addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => self.pc = self.pc.wrapping_add(1),
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => self.pc = self.pc.wrapping_add(2),
            AddressingMode::Implied => unreachable!(),
        };
        (self.memory.read_byte(addr), penalty)
    }

    /// return: penalty
    #[inline]
    pub fn write(&mut self, addressing_mode: AddressingMode, byte: u8) -> usize {
        debug_assert!(
            !matches!(addressing_mode, AddressingMode::Implied),
            "invalid AddressingMode given to write: {addressing_mode:?}"
        );

        let (addr, _) = self.resolve_address_old(addressing_mode);
        match addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => self.pc = self.pc.wrapping_add(1),
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => self.pc = self.pc.wrapping_add(2),
            AddressingMode::Implied => unreachable!(),
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
            AddressingMode::Implied | AddressingMode::Indirect => unreachable!(),
        }
    }

    pub fn step(&mut self) -> Result<()> {
        match self.state {
            CPUState::Reset => {
                self.sp = self.sp.wrapping_add(253);
                self.sr |= FLAG_INT;
                self.pc = self.memory.read_word(RESET_VECTOR_LO);

                self.cycles = 6;
                self.state = CPUState::Running;
                Ok(())
            }
            CPUState::Running => {
                let opcode = self.memory.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);

                if let Some(instruction) = Instruction::from_opcode(opcode) {
                    self.cycles += self.exec(instruction);
                    Ok(())
                } else {
                    Err(anyhow!("Invalid or unimplemented opcode: {opcode}"))
                }
            }
            CPUState::Jam => Ok(()),
        }
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.state = CPUState::Reset;
    }

    /// return: cycles this instruction took
    fn exec(&mut self, instruction: Instruction) -> usize {
        match instruction.mode() {
            AddressingMode::Implied => {
                // without operands
                match instruction.mnemonic() {
                    Mnemonic::INX => {
                        self.x = self.x.wrapping_add(1);
                        self.flag_set_negative(self.x);
                        self.flag_set_zero(self.x);
                    }
                    Mnemonic::TXS => self.sp = self.x,
                    Mnemonic::TAY => {
                        self.y = self.a;
                        self.flag_set_zero(self.y);
                        self.flag_set_negative(self.y);
                    }
                    Mnemonic::TSX => {
                        self.x = self.sp;
                        self.flag_set_zero(self.x);
                        self.flag_set_negative(self.x);
                    }
                    Mnemonic::TXA => {
                        self.a = self.x;
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                    }
                    Mnemonic::TAX => {
                        self.x = self.a;
                        self.flag_set_zero(self.x);
                        self.flag_set_negative(self.x);
                    }
                    Mnemonic::TYA => {
                        self.a = self.y;
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                    }
                    Mnemonic::ASL => {
                        let carry = self.a & 0b1000_0000 != 0;
                        self.a = self.a << 1;
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                        self.flag_set_carry(carry);
                    }
                    Mnemonic::LSR => {
                        let carry = self.a & 0b0000_0001 != 0;
                        self.a = self.a >> 1;
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                        self.flag_set_carry(carry);
                    }
                    Mnemonic::ROL => {
                        let carry = self.a & 0b1000_0000 != 0;
                        self.a = (self.a << 1) | if self.flag_get_carry() { 1 } else { 0 };
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                        self.flag_set_carry(carry);
                    }
                    Mnemonic::ROR => {
                        let carry = self.a & 0b0000_0001 != 0;
                        self.a = (self.a >> 1)
                            | if self.flag_get_carry() {
                                0b1000_0000
                            } else {
                                0
                            };
                        self.flag_set_zero(self.a);
                        self.flag_set_negative(self.a);
                        self.flag_set_carry(carry);
                    }
                    Mnemonic::CLC => self.sr &= !FLAG_CRY,
                    Mnemonic::CLD => self.sr &= !FLAG_DEC,
                    Mnemonic::CLI => self.sr &= !FLAG_INT,
                    Mnemonic::CLV => self.sr &= !FLAG_OVF,
                    Mnemonic::SEC => self.sr |= FLAG_CRY,
                    Mnemonic::SED => self.sr |= FLAG_DEC,
                    Mnemonic::SEI => self.sr |= FLAG_INT,
                    Mnemonic::NOP => {}
                    Mnemonic::JAM02
                    | Mnemonic::JAM12
                    | Mnemonic::JAM22
                    | Mnemonic::JAM32
                    | Mnemonic::JAM42
                    | Mnemonic::JAM52
                    | Mnemonic::JAM62
                    | Mnemonic::JAM72
                    | Mnemonic::JAM92
                    | Mnemonic::JAMB2
                    | Mnemonic::JAMD2
                    | Mnemonic::JAMF2 => self.state = CPUState::Jam,
                    _ => todo!(),
                };

                instruction.cycles()
            }
            // with operands
            _ => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                match instruction.mnemonic() {
                    Mnemonic::ADC => {
                        if self.flag_get_decimal() {
                            self.adc_decimal_part(addr);
                        } else {
                            self.arithmetic_binary_part(addr, false);
                        }
                    }
                    Mnemonic::SBC | Mnemonic::USBC => {
                        if self.flag_get_decimal() {
                            self.sbc_decimal_part(addr);
                        } else {
                            self.arithmetic_binary_part(addr, true);
                        }
                    }
                    Mnemonic::STA => {
                        self.memory.write_byte(addr, self.a);
                    }
                    Mnemonic::STX => {
                        self.memory.write_byte(addr, self.x);
                    }
                    Mnemonic::STY => {
                        self.memory.write_byte(addr, self.y);
                    }
                    Mnemonic::LDA => {
                        self.lda_part(addr);
                    }
                    Mnemonic::LDX => {
                        self.ldx_part(addr);
                    }
                    Mnemonic::LDY => {
                        self.ldy_part(addr);
                    }
                    Mnemonic::CMP | Mnemonic::CPX | Mnemonic::CPY => {
                        let reg = match instruction.mnemonic() {
                            Mnemonic::CMP => self.a,
                            Mnemonic::CPX => self.x,
                            Mnemonic::CPY => self.y,
                            _ => unreachable!(),
                        };

                        let operand = !self.memory.read_byte(addr);
                        let operand_carry = operand.wrapping_add(1);
                        let sum = reg.wrapping_add(operand_carry);

                        self.flag_set_zero(sum);
                        self.flag_set_negative(sum);
                        self.flag_set_carry(sum < reg || operand_carry < operand);
                    }
                    Mnemonic::AND | Mnemonic::EOR | Mnemonic::ORA => {
                        let operand = self.memory.read_byte(addr);
                        self.a = match instruction.mnemonic() {
                            Mnemonic::AND => self.a & operand,
                            Mnemonic::EOR => self.a ^ operand,
                            Mnemonic::ORA => self.a | operand,
                            _ => unreachable!(),
                        };
                        self.flag_set_negative(self.a);
                        self.flag_set_zero(self.a);
                    }
                    Mnemonic::BIT => {
                        let operand = self.memory.read_byte(addr);

                        self.flag_set_zero(self.a & operand);
                        self.sr &= 0b0011_1111;
                        self.sr |= operand & 0b1100_0000;
                    }
                    Mnemonic::ASL => {
                        self.asl_mem_part(addr);
                    }
                    Mnemonic::LSR => {
                        self.lsr_mem_part(addr);
                    }
                    Mnemonic::ROL => {
                        self.rol_mem_part(addr);
                    }
                    Mnemonic::ROR => {
                        self.ror_mem_part(addr);
                    }
                    Mnemonic::ISC => {
                        self.inc_part(addr);

                        if self.flag_get_decimal() {
                            self.sbc_decimal_part(addr);
                        } else {
                            self.arithmetic_binary_part(addr, true);
                        }
                    }

                    _ => todo!(),
                }

                instruction.cycles()
                    + if matches!(instruction.profile(), CycleProfile::Read) {
                        penalty
                    } else {
                        0
                    }
            }
        }

        /*
        match instruction.mnemonic() {
            Mnemonic::ADC => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());
                if self.flag_get_decimal() {
                    self.adc_decimal_part(addr);
                } else {
                    self.arithmetic_binary_part(addr, false);
                }

                instruction.cycles() + penalty
            }
            Mnemonic::SBC | Mnemonic::USBC => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());
                if self.flag_get_decimal() {
                    self.sbc_decimal_part(addr);
                } else {
                    self.arithmetic_binary_part(addr, true);
                }

                instruction.cycles() + penalty
            }
            Mnemonic::CPX | Mnemonic::CPY | Mnemonic::CMP => self.compare(instruction),
            Mnemonic::AND | Mnemonic::EOR | Mnemonic::ORA => self.logic(instruction),
            Mnemonic::BIT => self.bit(instruction),
            Mnemonic::STA => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.memory.write_byte(addr, self.a);

                instruction.cycles()
            }
            Mnemonic::STX => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.memory.write_byte(addr, self.a);

                instruction.cycles()
            }
            Mnemonic::STY => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.memory.write_byte(addr, self.a);

                instruction.cycles()
            }
            Mnemonic::STA | Mnemonic::STX | Mnemonic::STY => self.store(instruction),
            Mnemonic::LDA => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.lda_part(addr);

                instruction.cycles() + penalty
            }
            Mnemonic::LDX => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.ldx_part(addr);

                instruction.cycles() + penalty
            }
            Mnemonic::LDY => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                self.ldy_part(addr);

                instruction.cycles() + penalty
            }
            Mnemonic::TAX
            | Mnemonic::TAY
            | Mnemonic::TSX
            | Mnemonic::TXA
            | Mnemonic::TXS
            | Mnemonic::TYA => self.transfer(instruction),
            Mnemonic::ASL | Mnemonic::LSR | Mnemonic::ROL | Mnemonic::ROR => {
                self.shift_rotate(instruction)
            }
            Mnemonic::BEQ
            | Mnemonic::BCS
            | Mnemonic::BCC
            | Mnemonic::BMI
            | Mnemonic::BNE
            | Mnemonic::BPL
            | Mnemonic::BVC
            | Mnemonic::BVS => self.branch(instruction),
            Mnemonic::JMP => self.jmp(instruction),
            Mnemonic::JSR => self.jsr(instruction),
            Mnemonic::RTS => self.rts(instruction),
            Mnemonic::NOP => instruction.cycles(),
            Mnemonic::INC => {
                let ResolvedAddress { addr, .. } = self.resolve_address(instruction.mode());

                self.inc_part(addr);
                instruction.cycles()
            }
            Mnemonic::INX => self.inx(instruction),
            Mnemonic::INY => self.iny(instruction),
            Mnemonic::DEC => self.dec(instruction),
            Mnemonic::DEX => self.dex(instruction),
            Mnemonic::DEY => self.dey(instruction),
            Mnemonic::CLC
            | Mnemonic::CLD
            | Mnemonic::CLI
            | Mnemonic::CLV
            | Mnemonic::SEC
            | Mnemonic::SED
            | Mnemonic::SEI => self.flag(instruction),
            Mnemonic::BRK => self.brk(instruction),
            Mnemonic::RTI => self.rti(instruction),
            Mnemonic::PHA | Mnemonic::PHP => self.push_op(instruction),
            Mnemonic::PLA | Mnemonic::PLP => self.pull_op(instruction),
            Mnemonic::ISC => {
                let ResolvedAddress { addr, .. } = self.resolve_address(instruction.mode());

                self.inc_part(addr);

                if self.flag_get_decimal() {
                    self.sbc_decimal_part(addr);
                } else {
                    self.arithmetic_binary_part(addr, true);
                }

                instruction.cycles()
            }
            Mnemonic::LAS => {
                let ResolvedAddress { addr, penalty, .. } =
                    self.resolve_address(instruction.mode());

                instruction.cycles()
            }

            // -- undocumented --
            Mnemonic::ALR => self.alr(instruction),
            Mnemonic::ANC | Mnemonic::ANC2 => self.anc(instruction),
            Mnemonic::ARR => self.arr(instruction),
            Mnemonic::DCP => self.dcp(instruction),

            Mnemonic::JAM02
            | Mnemonic::JAM12
            | Mnemonic::JAM22
            | Mnemonic::JAM32
            | Mnemonic::JAM42
            | Mnemonic::JAM52
            | Mnemonic::JAM62
            | Mnemonic::JAM72
            | Mnemonic::JAM92
            | Mnemonic::JAMB2
            | Mnemonic::JAMD2
            | Mnemonic::JAMF2 => {
                self.state = CPUState::Jam;
                instruction.cycles()
            }
        }

        */
    }

    /// operand_1 is always A
    #[inline(always)]
    fn arithmetic_binary_part(&mut self, addr: u16, sbc: bool) {
        let operand_1 = self.a;
        let operand_2 = self.memory.read_byte(addr);
        let operand_2 = if sbc { !operand_2 } else { operand_2 };
        let carry_in = self.flag_get_carry() as u8;

        let sum = operand_1.wrapping_add(operand_2).wrapping_add(carry_in);
        let carry_out = sum < operand_1 || operand_2.wrapping_add(carry_in) < operand_2;

        self.a = sum;

        self.flag_set_zero(sum);
        self.flag_set_negative(sum);
        self.flag_set_carry(carry_out);
        self.flag_set_overflow(operand_1, operand_2, sum);
    }

    /// BCD MODE
    /// ---
    /// On the original NMOS 6502, the flag behaviour is quirky:
    /// - *Z* is set based on the binary sum.
    /// - *C* is set based on the BCD carry.
    /// - *N* and *V* are set based on an intermediate sum, where the high nibble has not been decimal adjusted.
    #[inline(always)]
    fn adc_decimal_part(&mut self, addr: u16) {
        let operand_1 = self.a;
        let operand_2 = self.memory.read_byte(addr);

        let carry_in = self.flag_get_carry() as u8;

        let binary_sum = operand_1.wrapping_add(operand_2).wrapping_add(carry_in);

        let lo = (operand_1 & 0x0f)
            .wrapping_add(operand_2 & 0x0f)
            .wrapping_add(carry_in);

        let (lo_adjusted, lo_carry) = if lo > 0x09 {
            let adjusted = lo.wrapping_add(6);
            ((adjusted & 0x0f) | 0x10, 1)
        } else {
            (lo, 0)
        };

        // overflow and negative are calculated using this intermediate sum on the NMOS 6502.
        // this is a quirk of the original hardware.
        let decimal_intermediate_sum = (operand_1 & 0xf0)
            .wrapping_add(operand_2 & 0xf0)
            .wrapping_add(lo_adjusted);

        let hi = ((operand_1 >> 4) & 0x0f)
            .wrapping_add(operand_2 >> 4)
            .wrapping_add(lo_carry);

        let (hi_adjusted, carry_out) = if hi > 0x09 {
            (hi.wrapping_add(6), true)
        } else {
            (hi, false)
        };

        let decimal_sum = (hi_adjusted << 4) | (lo_adjusted & 0x0f);
        self.a = decimal_sum;

        self.flag_set_zero(binary_sum);
        self.flag_set_carry(carry_out);
        self.flag_set_overflow(operand_1, operand_2, decimal_intermediate_sum);
        self.flag_set_negative(decimal_intermediate_sum);
    }

    /// BCD MODE
    /// ---
    /// - *Z*, *N* and *V* are set based on the binary sum.
    /// - *C* is set based on the BCD borrow.
    #[inline(always)]
    fn sbc_decimal_part(&mut self, addr: u16) {
        let operand_1 = self.a;
        let operand_2 = self.memory.read_byte(addr);

        let carry_in = self.flag_get_carry() as u8;

        let binary_sum = operand_1.wrapping_add(!operand_2).wrapping_add(carry_in);

        let lo = (operand_1 & 0x0f)
            .wrapping_add(!operand_2 & 0x0f)
            .wrapping_add(carry_in);

        let borrow_in = (carry_in == 0) as u8;
        let lo_borrow = (operand_1 & 0x0f) < (operand_2 & 0x0f).wrapping_add(borrow_in);

        let (lo_adjusted, lo_carry) = if lo_borrow {
            let adjusted = lo.wrapping_add(10);
            (adjusted & 0x0f, 1)
        } else {
            (lo, 0)
        };

        let hi = binary_sum >> 4;

        let hi_borrow =
            ((operand_1 >> 4) & 0x0f) < ((operand_2 >> 4) & 0x0f).wrapping_add(lo_carry);

        let (hi_adjusted, carry_out) = if hi_borrow {
            let adjusted = hi.wrapping_add(10);
            (adjusted & 0x0f, true)
        } else {
            (hi, false)
        };

        let decimal_sum = (hi_adjusted << 4) | (lo_adjusted & 0x0f);
        self.a = decimal_sum;

        self.flag_set_zero(binary_sum);
        self.flag_set_carry(!carry_out);
        self.flag_set_overflow(operand_1, !operand_2, binary_sum);
        self.flag_set_negative(binary_sum);
    }

    #[inline(always)]
    fn lda_part(&mut self, addr: u16) {
        self.a = self.memory.read_byte(addr);
        self.flag_set_negative(self.a);
        self.flag_set_zero(self.a);
    }

    #[inline(always)]
    fn ldx_part(&mut self, addr: u16) {
        self.x = self.memory.read_byte(addr);
        self.flag_set_negative(self.x);
        self.flag_set_zero(self.x);
    }

    #[inline(always)]
    fn ldy_part(&mut self, addr: u16) {
        self.y = self.memory.read_byte(addr);
        self.flag_set_negative(self.y);
        self.flag_set_zero(self.y);
    }

    #[inline(always)]
    fn asl_mem_part(&mut self, addr: u16) {
        let operand = self.memory.read_byte(addr);
        let result = operand << 1;
        let carry = operand & 0b1000_0000 != 0;
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.flag_set_carry(carry);
        self.memory.write_byte(addr, result);
    }

    #[inline(always)]
    fn lsr_mem_part(&mut self, addr: u16) {
        let operand = self.memory.read_byte(addr);
        let result = operand >> 1;
        let carry = operand & 0b0000_0001 != 0;
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.flag_set_carry(carry);
        self.memory.write_byte(addr, result);
    }

    #[inline(always)]
    fn rol_mem_part(&mut self, addr: u16) {
        let operand = self.memory.read_byte(addr);
        let result = (operand << 1) | if self.flag_get_carry() { 1 } else { 0 };
        let carry = operand & 0b1000_0000 != 0;
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.flag_set_carry(carry);
        self.memory.write_byte(addr, result);
    }

    #[inline(always)]
    fn ror_mem_part(&mut self, addr: u16) {
        let operand = self.memory.read_byte(addr);
        let result = (operand >> 1)
            | if self.flag_get_carry() {
                0b1000_0000
            } else {
                0
            };
        let carry = operand & 0b0000_0001 != 0;
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.flag_set_carry(carry);
        self.memory.write_byte(addr, result);
    }

    #[inline]
    fn branch(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::BEQ
                    | Mnemonic::BCS
                    | Mnemonic::BCC
                    | Mnemonic::BNE
                    | Mnemonic::BMI
                    | Mnemonic::BPL
                    | Mnemonic::BVS
                    | Mnemonic::BVC
            ),
            "Invalid Instruction given to branch: {instruction:?}"
        );

        debug_assert!(
            matches!(instruction.mode(), AddressingMode::Immediate),
            "Branch must always be 'Immediate' Addressing Mode: {instruction:?}"
        );

        let branch_taken = match instruction.mnemonic() {
            Mnemonic::BEQ => self.flag_get_zero(),
            Mnemonic::BNE => !self.flag_get_zero(),
            Mnemonic::BCS => self.flag_get_carry(),
            Mnemonic::BCC => !self.flag_get_carry(),
            Mnemonic::BMI => self.flag_get_negative(),
            Mnemonic::BPL => !self.flag_get_negative(),
            Mnemonic::BVS => self.flag_get_overflow(),
            Mnemonic::BVC => !self.flag_get_overflow(),
            _ => unreachable!(),
        };

        if branch_taken {
            let offset = self.fetch(instruction.mode()).0 as i8 as i16 as u16;
            let pc_target = self.pc.wrapping_add(offset);

            let cycles = if pc_target & 0xff00 == self.pc & 0xff00 {
                instruction.cycles() + 1
            } else {
                instruction.cycles() + 2
            };

            self.pc = pc_target;
            cycles
        } else {
            _ = self.fetch(instruction.mode());
            instruction.cycles()
        }
    }

    #[inline]
    fn jmp(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::JMP),
            "Invalid Instruction given to jmp: {instruction:?}"
        );

        self.pc = self.resolve_address_old(instruction.mode()).0;
        instruction.cycles()
    }

    #[inline]
    fn jsr(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::JSR),
            "Invalid Instruction given to jsr: {instruction:?}"
        );

        let target_lo = self.memory.read_byte(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let return_addr = self.pc;

        let return_addr_hi = (return_addr >> 8) as u8;
        self.memory.write_byte(self.sp_addr(), return_addr_hi);
        self.sp = self.sp.wrapping_add(255);

        let return_addr_lo = return_addr as u8;
        self.memory.write_byte(self.sp_addr(), return_addr_lo);
        self.sp = self.sp.wrapping_add(255);

        let target_hi = self.memory.read_byte(self.pc) as u16;
        self.pc = (target_hi << 8) | target_lo;

        instruction.cycles()
    }

    #[inline]
    fn rts(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::RTS),
            "Invalid Instruction given to rts: {instruction:?}"
        );

        self.sp = self.sp.wrapping_add(1);
        let pc_lo = self.memory.read_byte(self.sp_addr());

        self.sp = self.sp.wrapping_add(1);
        let pc_hi = self.memory.read_byte(self.sp_addr());

        self.pc = (((pc_hi as u16) << 8) | (pc_lo as u16)).wrapping_add(1);
        instruction.cycles()
    }

    #[inline(always)]
    fn inc_part(&mut self, addr: u16) {
        let result = self.memory.read_byte(addr).wrapping_add(1);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.memory.write_byte(addr, result);
    }

    #[inline]
    fn inx(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::INX),
            "Invalid Instruction given to inx: {instruction:?}"
        );

        debug_assert!(
            matches!(instruction.mode(), AddressingMode::Implied),
            "Invalid Mode for INX: {:?}",
            instruction.mode()
        );

        let result = self.x.wrapping_add(1);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.x = result;

        instruction.cycles()
    }

    #[inline]
    fn iny(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::INY),
            "Invalid Instruction given to iny: {instruction:?}"
        );

        debug_assert!(
            matches!(instruction.mode(), AddressingMode::Implied),
            "Invalid Mode for INY: {:?}",
            instruction.mode()
        );

        let result = self.y.wrapping_add(1);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.y = result;

        instruction.cycles()
    }

    #[inline]
    fn dec(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::DEC),
            "Invalid Instruction given to dec: {instruction:?}"
        );

        let addr = self.resolve_address_old(instruction.mode()).0;
        let result = self.memory.read_byte(addr).wrapping_add(255);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.write(instruction.mode(), result);

        instruction.cycles()
    }

    #[inline]
    fn dex(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::DEX),
            "Invalid Instruction given to dex: {instruction:?}"
        );

        debug_assert!(
            matches!(instruction.mode(), AddressingMode::Implied),
            "Invalid Mode for DEX: {:?}",
            instruction.mode()
        );

        let result = self.x.wrapping_add(255);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.x = result;

        instruction.cycles()
    }

    #[inline]
    fn dey(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::DEY),
            "Invalid Instruction given to dey: {instruction:?}"
        );

        debug_assert!(
            matches!(instruction.mode(), AddressingMode::Implied),
            "Invalid Mode for DEY: {:?}",
            instruction.mode()
        );

        let result = self.y.wrapping_add(255);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.y = result;

        instruction.cycles()
    }

    #[inline]
    fn flag(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::CLC
                    | Mnemonic::CLD
                    | Mnemonic::CLV
                    | Mnemonic::CLI
                    | Mnemonic::SEC
                    | Mnemonic::SEI
                    | Mnemonic::SED
            ),
            "Invalid Instruction given to flag: {instruction:?}"
        );

        match instruction.mnemonic() {
            Mnemonic::CLC => self.sr &= !FLAG_CRY,
            Mnemonic::CLD => self.sr &= !FLAG_DEC,
            Mnemonic::CLV => self.sr &= !FLAG_OVF,
            Mnemonic::CLI => self.sr &= !FLAG_INT,
            Mnemonic::SEC => self.sr |= FLAG_CRY,
            Mnemonic::SEI => self.sr |= FLAG_INT,
            Mnemonic::SED => self.sr |= FLAG_DEC,
            _ => unreachable!(),
        }

        instruction.cycles()
    }

    #[inline]
    fn brk(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::BRK),
            "Invalid Instruction given to brk: {instruction:?}"
        );

        let return_lo = self.pc.wrapping_add(1) as u8;
        let return_hi = ((self.pc.wrapping_add(1) & 0xff00) >> 8) as u8;

        self.push_stack(return_hi);
        self.push_stack(return_lo);
        self.push_stack(self.sr | FLAG_BRK | FLAG_IGN);

        self.sr |= FLAG_INT;

        let irq_addr_lo = self.memory.read_byte(IRQ_VECTOR_LO);
        let irq_addr_hi = self.memory.read_byte(IRQ_VECTOR_HI);
        self.pc = ((irq_addr_hi as u16) << 8) | (irq_addr_lo as u16);

        instruction.cycles()
    }

    #[inline]
    fn rti(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::RTI),
            "Invalid Instruction given to rti: {instruction:?}"
        );

        let sr = self.pop_stack();
        let return_lo = self.pop_stack();
        let return_hi = self.pop_stack();

        self.sr = (sr & !FLAG_BRK) | FLAG_IGN;
        self.pc = ((return_hi as u16) << 8) | (return_lo as u16);

        instruction.cycles()
    }

    #[inline]
    fn push_op(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::PHA | Mnemonic::PHP),
            "Invalid Instruction given to push_op: {instruction:?}"
        );

        match instruction.mnemonic() {
            Mnemonic::PHA => self.push_stack(self.a),
            Mnemonic::PHP => self.push_stack(self.sr | FLAG_BRK | FLAG_IGN),
            _ => unreachable!(),
        }

        instruction.cycles()
    }

    #[inline]
    fn pull_op(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::PLA | Mnemonic::PLP),
            "Invalid Instruction given to pull_op: {instruction:?}"
        );

        match instruction.mnemonic() {
            Mnemonic::PLA => {
                self.a = self.pop_stack();
                self.flag_set_zero(self.a);
                self.flag_set_negative(self.a);
            }
            Mnemonic::PLP => self.sr = (self.pop_stack() & !FLAG_BRK) | FLAG_IGN,
            _ => unreachable!(),
        }

        instruction.cycles()
    }

    #[inline]
    fn alr(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::ALR),
            "Invalid Instruction given to alr: {instruction:?}"
        );

        let operand_1 = self.a;
        let operand_2 = self.fetch(instruction.mode()).0;

        let result_and = operand_1 & operand_2;
        let result_shift = result_and >> 1;
        let carry_out = (result_and & 1) == 1;

        self.a = result_shift;

        self.flag_set_negative(result_shift);
        self.flag_set_zero(result_shift);
        self.flag_set_carry(carry_out);

        instruction.cycles()
    }

    #[inline]
    fn anc(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::ANC | Mnemonic::ANC2),
            "Invalid Instruction given to anc: {instruction:?}"
        );

        let operand_1 = self.a;
        let operand_2 = self.fetch(instruction.mode()).0;

        let result_and = operand_1 & operand_2;
        let carry_out = result_and & 0b1000_0000 != 0;

        self.a = result_and;

        self.flag_set_negative(result_and);
        self.flag_set_zero(result_and);
        self.flag_set_carry(carry_out);

        instruction.cycles()
    }

    #[inline]
    fn arr(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::ARR),
            "Invalid Instruction given to arr: {instruction:?}"
        );

        let operand_1 = self.a;
        let operand_2 = self.fetch(instruction.mode()).0;

        let result_and = operand_1 & operand_2;
        let result_ror = (result_and >> 1)
            | if self.flag_get_carry() {
                0b1000_0000
            } else {
                0
            };

        self.flag_set_negative(result_ror);
        self.flag_set_zero(result_ror);
        self.flag_set_carry((result_ror & 0x40) != 0);

        if (((result_ror >> 6) ^ (result_ror >> 5)) & 1) != 0 {
            self.sr |= FLAG_OVF;
        } else {
            self.sr &= !FLAG_OVF;
        }

        self.a = result_ror;

        instruction.cycles()
    }

    #[inline]
    fn dcp(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::DCP),
            "Invalid Instruction given to dcp: {instruction:?}"
        );

        let byte = {
            let addr = self.resolve_address_old(instruction.mode()).0;
            self.memory.read_byte(addr)
        };

        let result_dec = byte.wrapping_add(255);
        self.write(instruction.mode(), result_dec);

        let operand_1 = self.a;
        let operand_2 = !result_dec;
        let operand_2_carry = operand_2.wrapping_add(1);

        let result = operand_1.wrapping_add(operand_2_carry);

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || operand_2_carry < operand_2);

        instruction.cycles()
    }

    fn push_stack(&mut self, byte: u8) {
        self.memory.write_byte(self.sp_addr(), byte);
        self.sp = self.sp.wrapping_add(255);
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.memory.read_byte(self.sp_addr())
    }

    fn sp_addr(&self) -> u16 {
        (self.sp as u16).wrapping_add(STACK_OFFSET)
    }

    fn flag_get_carry(&self) -> bool {
        (self.sr & FLAG_CRY) != 0
    }

    fn flag_get_zero(&self) -> bool {
        (self.sr & FLAG_ZRO) != 0
    }

    fn flag_get_negative(&self) -> bool {
        (self.sr & FLAG_NEG) != 0
    }

    fn flag_get_overflow(&self) -> bool {
        (self.sr & FLAG_OVF) != 0
    }

    fn flag_get_decimal(&self) -> bool {
        (self.sr & FLAG_DEC) != 0
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

    /// helper for signed arithmetic
    fn flag_set_overflow(&mut self, a: u8, b: u8, result: u8) {
        if (a ^ result) & (b ^ result) & 0x80 != 0 {
            self.sr |= FLAG_OVF
        } else {
            self.sr &= !FLAG_OVF
        }
    }
}
