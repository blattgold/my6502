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

pub const STACK_OFFSET: u16 = 256 * 1;

pub enum CPUState {
    Reset,
    Running,
}

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

    /// return: (address, page cross penalty value)
    #[inline]
    fn resolve_address(&mut self, addressing_mode: AddressingMode) -> (u16, usize) {
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
                let addr_lo = self.memory.read_byte(self.pc);
                let addr_hi = self
                    .memory
                    .read_byte((self.pc.wrapping_add(1) & 0x00ff) | (self.pc & 0xff00));

                ((addr_hi as u16) << 8 | addr_lo as u16, 0)
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

        let (addr, penalty) = self.resolve_address(addressing_mode);
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

        let (addr, _) = self.resolve_address(addressing_mode);
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
        let opcode = self.memory.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

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
                if let Some(instruction) = Instruction::from_opcode(opcode) {
                    self.cycles += self.exec(instruction);
                    Ok(())
                } else {
                    Err(anyhow!("Invalid or unimplemented opcode: {opcode}"))
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.state = CPUState::Reset;
    }

    /// return: cycles this instruction took
    #[inline]
    fn exec(&mut self, instruction: Instruction) -> usize {
        match instruction.mnemonic() {
            Mnemonic::ADC | Mnemonic::SBC => self.arithmetic(instruction),
            Mnemonic::CPX | Mnemonic::CPY | Mnemonic::CMP => self.compare(instruction),
            Mnemonic::AND | Mnemonic::EOR | Mnemonic::ORA => self.logic(instruction),
            Mnemonic::BIT => self.bit(instruction),
            Mnemonic::STA | Mnemonic::STX | Mnemonic::STY => self.store(instruction),
            Mnemonic::LDA | Mnemonic::LDX | Mnemonic::LDY => self.load(instruction),
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
            Mnemonic::NOP => self.nop(instruction),
            Mnemonic::INC => self.inc(instruction),
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
        }
    }

    /// return: cycles taken
    #[inline]
    fn arithmetic(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::ADC | Mnemonic::SBC),
            "Invalid Instruction given to adc: {instruction:?}",
        );

        if self.flag_get_decimal() {
            if instruction.mnemonic() == Mnemonic::ADC {
                self.adc_decimal(instruction)
            } else {
                self.sbc_decimal(instruction)
            }
        } else {
            self.arithmetic_binary(instruction)
        }
    }

    #[inline]
    fn arithmetic_binary(&mut self, instruction: Instruction) -> usize {
        let subtract = instruction.mnemonic() == Mnemonic::SBC;

        let a = self.a;
        let (b, penalty) = self.fetch(instruction.mode());
        let b = if subtract { !b } else { b };
        let carry_in = self.flag_get_carry() as u8;

        let sum = a.wrapping_add(b).wrapping_add(carry_in);
        let carry_out = sum < a || b.wrapping_add(carry_in) < b;

        self.a = sum;

        self.flag_set_zero(sum);
        self.flag_set_negative(sum);
        self.flag_set_carry(carry_out);
        self.flag_set_overflow(a, b, sum);

        penalty + instruction.cycles()
    }

    /// BCD MODE
    /// ---
    /// On the original NMOS 6502, the flag behaviour is quirky:
    /// - *Z* is set based on the binary sum.
    /// - *C* is set based on the BCD carry.
    /// - *N* and *V* are set based on an intermediate sum, where the high nibble has not been decimal adjusted.
    #[inline]
    fn adc_decimal(&mut self, instruction: Instruction) -> usize {
        let a = self.a;
        let (b, penalty) = self.fetch(instruction.mode());

        let carry_in = self.flag_get_carry() as u8;

        let binary_sum = a.wrapping_add(b).wrapping_add(carry_in);

        let lo = (a & 0x0f).wrapping_add(b & 0x0f).wrapping_add(carry_in);

        let (lo_adjusted, lo_carry) = if lo > 0x09 {
            let adjusted = lo.wrapping_add(6);
            ((adjusted & 0x0f) | 0x10, 1)
        } else {
            (lo, 0)
        };

        // overflow and negative are calculated using this intermediate sum on the NMOS 6502.
        // this is a quirk of the original hardware.
        let decimal_intermediate_sum = (a & 0xf0).wrapping_add(b & 0xf0).wrapping_add(lo_adjusted);

        let hi = ((a >> 4) & 0x0f)
            .wrapping_add(b >> 4)
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
        self.flag_set_overflow(a, b, decimal_intermediate_sum);
        self.flag_set_negative(decimal_intermediate_sum);

        penalty + instruction.cycles()
    }

    /// BCD MODE
    /// ---
    /// - *Z*, *N* and *V* are set based on the binary sum.
    /// - *C* is set based on the BCD borrow.
    #[inline]
    fn sbc_decimal(&mut self, instruction: Instruction) -> usize {
        let a = self.a;
        let (b, penalty) = self.fetch(instruction.mode());

        let carry_in = self.flag_get_carry() as u8;

        let binary_sum = a.wrapping_add(!b).wrapping_add(carry_in);

        let lo = (a & 0x0f).wrapping_add(!b & 0x0f).wrapping_add(carry_in);

        let borrow_in = (carry_in == 0) as u8;
        let lo_borrow = (a & 0x0f) < (b & 0x0f).wrapping_add(borrow_in);

        let (lo_adjusted, lo_carry) = if lo_borrow {
            let adjusted = lo.wrapping_add(10);
            (adjusted & 0x0f, 1)
        } else {
            (lo, 0)
        };

        let hi = binary_sum >> 4;

        let hi_borrow = ((a >> 4) & 0x0f) < ((b >> 4) & 0x0f).wrapping_add(lo_carry);

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
        self.flag_set_overflow(a, !b, binary_sum);
        self.flag_set_negative(binary_sum);

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
        let operand_2 = !operand_2;

        let operand_2_carry = operand_2.wrapping_add(1);
        let result = operand_1.wrapping_add(operand_2_carry);

        self.flag_set_zero(result);
        self.flag_set_negative(result);
        self.flag_set_carry(result < operand_1 || operand_2_carry < operand_2);

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

    #[inline]
    fn transfer(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::TAX
                    | Mnemonic::TSX
                    | Mnemonic::TAY
                    | Mnemonic::TXA
                    | Mnemonic::TXS
                    | Mnemonic::TYA
            ),
            "Invalid Instruction given to transfer: {instruction:?}"
        );

        let byte = match instruction.mnemonic() {
            Mnemonic::TAX | Mnemonic::TAY => self.a,
            Mnemonic::TSX => self.sp,
            Mnemonic::TXA | Mnemonic::TXS => self.x,
            Mnemonic::TYA => self.y,
            _ => unreachable!(),
        };

        match instruction.mnemonic() {
            Mnemonic::TXA | Mnemonic::TAY | Mnemonic::TSX | Mnemonic::TAX | Mnemonic::TYA => {
                self.flag_set_negative(byte);
                self.flag_set_zero(byte);
            }
            Mnemonic::TXS => {}
            _ => unreachable!(),
        }

        match instruction.mnemonic() {
            Mnemonic::TAX | Mnemonic::TSX => self.x = byte,
            Mnemonic::TAY => self.y = byte,
            Mnemonic::TXA | Mnemonic::TYA => self.a = byte,
            Mnemonic::TXS => self.sp = byte,
            _ => unreachable!(),
        }

        instruction.cycles()
    }

    #[inline]
    fn shift_rotate(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(
                instruction.mnemonic(),
                Mnemonic::ASL | Mnemonic::LSR | Mnemonic::ROL | Mnemonic::ROR
            ),
            "Invalid Instruction given to shift_rotate: {instruction:?}"
        );

        let operand = match instruction.mode() {
            AddressingMode::Implied => self.a,
            AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::Absolute
            | AddressingMode::AbsoluteX => {
                let addr = self.resolve_address(instruction.mode()).0;
                self.memory.read_byte(addr)
            }
            _ => unreachable!(),
        };

        let (carry, result) = match instruction.mnemonic() {
            Mnemonic::ASL => (operand & 0b1000_0000 != 0, operand << 1),
            Mnemonic::LSR => (operand & 0b0000_0001 != 0, operand >> 1),
            Mnemonic::ROL => (
                operand & 0b1000_0000 != 0,
                (operand << 1) | if self.flag_get_carry() { 1 } else { 0 },
            ),
            Mnemonic::ROR => (
                operand & 0b0000_0001 != 0,
                (operand >> 1)
                    | if self.flag_get_carry() {
                        0b1000_0000
                    } else {
                        0
                    },
            ),
            _ => unreachable!(),
        };

        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.flag_set_carry(carry);

        match instruction.mode() {
            AddressingMode::Implied => self.a = result,
            AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::Absolute
            | AddressingMode::AbsoluteX => _ = self.write(instruction.mode(), result),
            _ => unreachable!(),
        }

        instruction.cycles()
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

        self.pc = self.resolve_address(instruction.mode()).0;
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

    #[inline]
    fn nop(&mut self, _instruction: Instruction) -> usize {
        2
    }

    #[inline]
    fn inc(&mut self, instruction: Instruction) -> usize {
        debug_assert!(
            matches!(instruction.mnemonic(), Mnemonic::INC),
            "Invalid Instruction given to inc: {instruction:?}"
        );

        let addr = self.resolve_address(instruction.mode()).0;
        let result = self.memory.read_byte(addr).wrapping_add(1);
        self.flag_set_negative(result);
        self.flag_set_zero(result);
        self.write(instruction.mode(), result);

        instruction.cycles()
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

        let addr = self.resolve_address(instruction.mode()).0;
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
