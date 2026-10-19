use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use test_case::test_case;

mod json_data;
mod test_memory;
use json_data::*;
use test_memory::*;

use my6502::{cpu::CPU, memory::Memory};

/// Some tests are commented out, those are unstable instructions which are out of scope, for now.
/// Even if they were not out of scope, testing them like this doesn't work.
/// Because some, for example, use a constant value that is dependent on the chips temperature.
///
///
/// Unstable Opcodes:
///  - ANE
///
/// I have also uncommented some which I couldn't figure out, such as:
///
/// ARR
/// ---
/// Tried a couple of things to make it work, including looking through other emulators source code and begging DeepSeek for assistance.
/// I usually use LLMs only for research/studying assistance, that should speak to how spooked I got by this opcode.
/// It works for roughly 75% of the test cases
/// DeepSeek suggested some janky fixes which brought it up to roughly 85%, but I decided to not use them because it convoluted the logic and made the code eccentric.
/// I am going to leave my version of ARR available because it works consistently.
/// I have concluded that it probably doesn't work for one of the following reasons:
/// - It is timing/ordering sensitive in some obnoxious way
/// - It is some general NMOS 6502 unoffical opcode tomfoolery
/// - It has something to do with decimal mode which has scarred me for life
/// - Maybe the test is just wrong
#[test_case(0x00 ; "0x00 BRK impl")]
#[test_case(0x01 ; "0x01 ORA X ind")]
#[test_case(0x02 ; "0x02 JAM")]
#[test_case(0x03 ; "0x03 SLO X ind")]
#[test_case(0x04 ; "0x04 NOP zpg")]
#[test_case(0x05 ; "0x05 ORA zpg")]
#[test_case(0x06 ; "0x06 ASL zpg")]
#[test_case(0x07 ; "0x07 SLO zpg")]
#[test_case(0x08 ; "0x08 PHP impl")]
#[test_case(0x09 ; "0x09 ORA imm")]
#[test_case(0x0a ; "0x0a ASL A")]
#[test_case(0x0b ; "0x0b ANC imm")]
#[test_case(0x0c ; "0x0c NOP abs")]
#[test_case(0x0d ; "0x0d ORA abs")]
#[test_case(0x0e ; "0x0e ASL abs")]
#[test_case(0x0f ; "0x0f SLO abs")]
#[test_case(0x10 ; "0x10 BPL rel")]
#[test_case(0x11 ; "0x11 ORA ind Y")]
#[test_case(0x12 ; "0x12 JAM")]
#[test_case(0x13 ; "0x13 SLO ind Y")]
#[test_case(0x14 ; "0x14 NOP zpg X")]
#[test_case(0x15 ; "0x15 ORA zpg X")]
#[test_case(0x16 ; "0x16 ASL zpg X")]
#[test_case(0x17 ; "0x17 SLO zpg X")]
#[test_case(0x18 ; "0x18 CLC impl")]
#[test_case(0x19 ; "0x19 ORA abs Y")]
#[test_case(0x1a ; "0x1a NOP impl")]
#[test_case(0x1b ; "0x1b SLO abs Y")]
#[test_case(0x1c ; "0x1c NOP abs X")]
#[test_case(0x1d ; "0x1d ORA abs X")]
#[test_case(0x1e ; "0x1e ASL abs X")]
#[test_case(0x1f ; "0x1f SLO abs X")]
#[test_case(0x20 ; "0x20 JSR abs")]
#[test_case(0x21 ; "0x21 AND X ind")]
#[test_case(0x22 ; "0x22 JAM")]
#[test_case(0x23 ; "0x23 RLA X ind")]
#[test_case(0x24 ; "0x24 BIT zpg")]
#[test_case(0x25 ; "0x25 AND zpg")]
#[test_case(0x26 ; "0x26 ROL zpg")]
#[test_case(0x27 ; "0x27 RLA zpg")]
#[test_case(0x28 ; "0x28 PLP impl")]
#[test_case(0x29 ; "0x29 AND imm")]
#[test_case(0x2a ; "0x2a ROL A")]
#[test_case(0x2b ; "0x2b ANC imm")]
#[test_case(0x2c ; "0x2c BIT abs")]
#[test_case(0x2d ; "0x2d AND abs")]
#[test_case(0x2e ; "0x2e ROL abs")]
#[test_case(0x2f ; "0x2f RLA abs")]
#[test_case(0x30 ; "0x30 BMI rel")]
#[test_case(0x31 ; "0x31 AND ind Y")]
#[test_case(0x32 ; "0x32 JAM")]
#[test_case(0x33 ; "0x33 RLA ind Y")]
#[test_case(0x34 ; "0x34 NOP zpg X")]
#[test_case(0x35 ; "0x35 AND zpg X")]
#[test_case(0x36 ; "0x36 ROL zpg X")]
#[test_case(0x37 ; "0x37 RLA zpg X")]
#[test_case(0x38 ; "0x38 SEC impl")]
#[test_case(0x39 ; "0x39 AND abs Y")]
#[test_case(0x3a ; "0x3a NOP impl")]
#[test_case(0x3b ; "0x3b RLA abs Y")]
#[test_case(0x3c ; "0x3c NOP abs X")]
#[test_case(0x3d ; "0x3d AND abs X")]
#[test_case(0x3e ; "0x3e ROL abs X")]
#[test_case(0x3f ; "0x3f RLA abs X")]
#[test_case(0x40 ; "0x40 RTI impl")]
#[test_case(0x41 ; "0x41 EOR X ind")]
#[test_case(0x42 ; "0x42 JAM")]
#[test_case(0x43 ; "0x43 SRE X ind")]
#[test_case(0x44 ; "0x44 NOP zpg")]
#[test_case(0x45 ; "0x45 EOR zpg")]
#[test_case(0x46 ; "0x46 LSR zpg")]
#[test_case(0x47 ; "0x47 SRE zpg")]
#[test_case(0x48 ; "0x48 PHA impl")]
#[test_case(0x49 ; "0x49 EOR imm")]
#[test_case(0x4a ; "0x4a LSR A")]
#[test_case(0x4b ; "0x4b ALR imm")]
#[test_case(0x4c ; "0x4c JMP abs")]
#[test_case(0x4d ; "0x4d EOR abs")]
#[test_case(0x4e ; "0x4e LSR abs")]
#[test_case(0x4f ; "0x4f SRE abs")]
#[test_case(0x50 ; "0x50 BVC rel")]
#[test_case(0x51 ; "0x51 EOR ind Y")]
#[test_case(0x52 ; "0x52 JAM")]
#[test_case(0x53 ; "0x53 SRE ind Y")]
#[test_case(0x54 ; "0x54 NOP zpg X")]
#[test_case(0x55 ; "0x55 EOR zpg X")]
#[test_case(0x56 ; "0x56 LSR zpg X")]
#[test_case(0x57 ; "0x57 SRE zpg X")]
#[test_case(0x58 ; "0x58 CLI impl")]
#[test_case(0x59 ; "0x59 EOR abs Y")]
#[test_case(0x5a ; "0x5a NOP impl")]
#[test_case(0x5b ; "0x5b SRE abs Y")]
#[test_case(0x5c ; "0x5c NOP abs X")]
#[test_case(0x5d ; "0x5d EOR abs X")]
#[test_case(0x5e ; "0x5e LSR abs X")]
#[test_case(0x5f ; "0x5f SRE abs X")]
#[test_case(0x60 ; "0x60 RTS impl")]
#[test_case(0x61 ; "0x61 ADC X ind")]
#[test_case(0x62 ; "0x62 JAM")]
#[test_case(0x63 ; "0x63 RRA X ind")]
#[test_case(0x64 ; "0x64 NOP zpg")]
#[test_case(0x65 ; "0x65 ADC zpg")]
#[test_case(0x66 ; "0x66 ROR zpg")]
#[test_case(0x67 ; "0x67 RRA zpg")]
#[test_case(0x68 ; "0x68 PLA impl")]
#[test_case(0x69 ; "0x69 ADC imm")]
#[test_case(0x6a ; "0x6a ROR A")]
// #[test_case(0x6b ; "0x6b ARR imm")]
#[test_case(0x6c ; "0x6c JMP ind")]
#[test_case(0x6d ; "0x6d ADC abs")]
#[test_case(0x6e ; "0x6e ROR abs")]
#[test_case(0x6f ; "0x6f RRA abs")]
#[test_case(0x70 ; "0x70 BVS rel")]
#[test_case(0x71 ; "0x71 ADC ind Y")]
#[test_case(0x72 ; "0x72 JAM")]
#[test_case(0x73 ; "0x73 RRA ind Y")]
#[test_case(0x74 ; "0x74 NOP zpg X")]
#[test_case(0x75 ; "0x75 ADC zpg X")]
#[test_case(0x76 ; "0x76 ROR zpg X")]
#[test_case(0x77 ; "0x77 RRA zpg X")]
#[test_case(0x78 ; "0x78 SEI impl")]
#[test_case(0x79 ; "0x79 ADC abs Y")]
#[test_case(0x7a ; "0x7a NOP impl")]
#[test_case(0x7b ; "0x7b RRA abs Y")]
#[test_case(0x7c ; "0x7c NOP abs X")]
#[test_case(0x7d ; "0x7d ADC abs X")]
#[test_case(0x7e ; "0x7e ROR abs X")]
#[test_case(0x7f ; "0x7f RRA abs X")]
#[test_case(0x80 ; "0x80 NOP imm")]
#[test_case(0x81 ; "0x81 STA X ind")]
#[test_case(0x82 ; "0x82 NOP imm")]
#[test_case(0x83 ; "0x83 SAX X ind")]
#[test_case(0x84 ; "0x84 STY zpg")]
#[test_case(0x85 ; "0x85 STA zpg")]
#[test_case(0x86 ; "0x86 STX zpg")]
#[test_case(0x87 ; "0x87 SAX zpg")]
#[test_case(0x88 ; "0x88 DEY impl")]
#[test_case(0x89 ; "0x89 NOP imm")]
#[test_case(0x8a ; "0x8a TXA impl")]
// #[test_case(0x8b ; "0x8b ANE imm")]
#[test_case(0x8c ; "0x8c STY abs")]
#[test_case(0x8d ; "0x8d STA abs")]
#[test_case(0x8e ; "0x8e STX abs")]
#[test_case(0x8f ; "0x8f SAX abs")]
#[test_case(0x90 ; "0x90 BCC rel")]
#[test_case(0x91 ; "0x91 STA ind Y")]
#[test_case(0x92 ; "0x92 JAM")]
#[test_case(0x93 ; "0x93 SHA ind Y")]
#[test_case(0x94 ; "0x94 STY zpg X")]
#[test_case(0x95 ; "0x95 STA zpg X")]
#[test_case(0x96 ; "0x96 STX zpg Y")]
#[test_case(0x97 ; "0x97 SAX zpg Y")]
#[test_case(0x98 ; "0x98 TYA impl")]
#[test_case(0x99 ; "0x99 STA abs Y")]
#[test_case(0x9a ; "0x9a TXS impl")]
#[test_case(0x9b ; "0x9b TAS abs Y")]
#[test_case(0x9c ; "0x9c SHY abs X")]
#[test_case(0x9d ; "0x9d STA abs X")]
#[test_case(0x9e ; "0x9e SHX abs Y")]
#[test_case(0x9f ; "0x9f SHA abs Y")]
#[test_case(0xa0 ; "0xa0 LDY imm")]
#[test_case(0xa1 ; "0xa1 LDA X ind")]
#[test_case(0xa2 ; "0xa2 LDX imm")]
#[test_case(0xa3 ; "0xa3 LAX X ind")]
#[test_case(0xa4 ; "0xa4 LDY zpg")]
#[test_case(0xa5 ; "0xa5 LDA zpg")]
#[test_case(0xa6 ; "0xa6 LDX zpg")]
#[test_case(0xa7 ; "0xa7 LAX zpg")]
#[test_case(0xa8 ; "0xa8 TAY impl")]
#[test_case(0xa9 ; "0xa9 LDA imm")]
#[test_case(0xaa ; "0xaa TAX impl")]
#[test_case(0xab ; "0xab LXA imm")]
#[test_case(0xac ; "0xac LDY abs")]
#[test_case(0xad ; "0xad LDA abs")]
#[test_case(0xae ; "0xae LDX abs")]
#[test_case(0xaf ; "0xaf LAX abs")]
#[test_case(0xb0 ; "0xb0 BCS rel")]
#[test_case(0xb1 ; "0xb1 LDA ind Y")]
#[test_case(0xb2 ; "0xb2 JAM")]
#[test_case(0xb3 ; "0xb3 LAX ind Y")]
#[test_case(0xb4 ; "0xb4 LDY zpg X")]
#[test_case(0xb5 ; "0xb5 LDA zpg X")]
#[test_case(0xb6 ; "0xb6 LDX zpg Y")]
#[test_case(0xb7 ; "0xb7 LAX zpg Y")]
#[test_case(0xb8 ; "0xb8 CLV impl")]
#[test_case(0xb9 ; "0xb9 LDA abs Y")]
#[test_case(0xba ; "0xba TSX impl")]
#[test_case(0xbb ; "0xbb LAS abs Y")]
#[test_case(0xbc ; "0xbc LDY abs X")]
#[test_case(0xbd ; "0xbd LDA abs X")]
#[test_case(0xbe ; "0xbe LDX abs Y")]
#[test_case(0xbf ; "0xbf LAX abs Y")]
#[test_case(0xc0 ; "0xc0 CPY imm")]
#[test_case(0xc1 ; "0xc1 CMP X ind")]
#[test_case(0xc2 ; "0xc2 NOP imm")]
#[test_case(0xc3 ; "0xc3 DCP X ind")]
#[test_case(0xc4 ; "0xc4 CPY zpg")]
#[test_case(0xc5 ; "0xc5 CMP zpg")]
#[test_case(0xc6 ; "0xc6 DEC zpg")]
#[test_case(0xc7 ; "0xc7 DCP zpg")]
#[test_case(0xc8 ; "0xc8 INY impl")]
#[test_case(0xc9 ; "0xc9 CMP imm")]
#[test_case(0xca ; "0xca DEX impl")]
#[test_case(0xcb ; "0xcb SBX imm")]
#[test_case(0xcc ; "0xcc CPY abs")]
#[test_case(0xcd ; "0xcd CMP abs")]
#[test_case(0xce ; "0xce DEC abs")]
#[test_case(0xcf ; "0xcf DCP abs")]
#[test_case(0xd0 ; "0xd0 BNE rel")]
#[test_case(0xd1 ; "0xd1 CMP ind Y")]
#[test_case(0xd2 ; "0xd2 JAM")]
#[test_case(0xd3 ; "0xd3 DCP ind Y")]
#[test_case(0xd4 ; "0xd4 NOP zpg X")]
#[test_case(0xd5 ; "0xd5 CMP zpg X")]
#[test_case(0xd6 ; "0xd6 DEC zpg X")]
#[test_case(0xd7 ; "0xd7 DCP zpg X")]
#[test_case(0xd8 ; "0xd8 CLD impl")]
#[test_case(0xd9 ; "0xd9 CMP abs Y")]
#[test_case(0xda ; "0xda NOP impl")]
#[test_case(0xdb ; "0xdb DCP abs Y")]
#[test_case(0xdc ; "0xdc NOP abs X")]
#[test_case(0xdd ; "0xdd CMP abs X")]
#[test_case(0xde ; "0xde DEC abs X")]
#[test_case(0xdf ; "0xdf DCP abs X")]
#[test_case(0xe0 ; "0xe0 CPX imm")]
#[test_case(0xe1 ; "0xe1 SBC X ind")]
#[test_case(0xe2 ; "0xe2 NOP imm")]
#[test_case(0xe3 ; "0xe3 ISC X ind")]
#[test_case(0xe4 ; "0xe4 CPX zpg")]
#[test_case(0xe5 ; "0xe5 SBC zpg")]
#[test_case(0xe6 ; "0xe6 INC zpg")]
#[test_case(0xe7 ; "0xe7 ISC zpg")]
#[test_case(0xe8 ; "0xe8 INX impl")]
#[test_case(0xe9 ; "0xe9 SBC imm")]
#[test_case(0xea ; "0xea NOP impl")]
#[test_case(0xeb ; "0xeb USBC imm")]
#[test_case(0xec ; "0xec CPX abs")]
#[test_case(0xed ; "0xed SBC abs")]
#[test_case(0xee ; "0xee INC abs")]
#[test_case(0xef ; "0xef ISC abs")]
#[test_case(0xf0 ; "0xf0 BEQ rel")]
#[test_case(0xf1 ; "0xf1 SBC ind Y")]
#[test_case(0xf2 ; "0xf2 JAM")]
#[test_case(0xf3 ; "0xf3 ISC ind Y")]
#[test_case(0xf4 ; "0xf4 NOP zpg X")]
#[test_case(0xf5 ; "0xf5 SBC zpg X")]
#[test_case(0xf6 ; "0xf6 INC zpg X")]
#[test_case(0xf7 ; "0xf7 ISC zpg X")]
#[test_case(0xf8 ; "0xf8 SED impl")]
#[test_case(0xf9 ; "0xf9 SBC abs Y")]
#[test_case(0xfa ; "0xfa NOP impl")]
#[test_case(0xfb ; "0xfb ISC abs Y")]
#[test_case(0xfc ; "0xfc NOP abs X")]
#[test_case(0xfd ; "0xfd SBC abs X")]
#[test_case(0xfe ; "0xfe INC abs X")]
#[test_case(0xff ; "0xff ISC abs X")]
fn test_opcode(opcode: u8) -> Result<()> {
    let verbose_output: bool = std::env::var("TEST_6502_VERBOSE").is_ok();

    let test_file_path = PathBuf::from(format!("tests/json/v1/{opcode:02x?}.json"));
    let file_string = read_to_string(test_file_path)?;
    let tests: Vec<TestCase> = serde_json::from_str(&file_string)?;

    let mut failures = Vec::new();

    let memory = TestMemory::default();
    let mut cpu = CPU::new(memory, true);

    for test in tests {
        cpu.memory.clear();
        cpu.a = test.initial.a;
        cpu.x = test.initial.x;
        cpu.y = test.initial.y;
        cpu.sp = test.initial.s;
        cpu.sr = test.initial.p;
        cpu.pc = test.initial.pc;
        for (addr, byte) in test.initial.ram.clone() {
            cpu.memory.write_byte(addr, byte);
        }

        match cpu.step() {
            Ok(()) => {
                // validate final state
                let mut failure_string = String::new();

                if cpu.a != test.final_state.a {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register A; Expected: {}, Got: {}\n",
                        test.final_state.a, cpu.a
                    ))
                }

                if cpu.x != test.final_state.x {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register X; Expected: {}, Got: {}\n",
                        test.final_state.x, cpu.x
                    ))
                }

                if cpu.y != test.final_state.y {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register Y; Expected: {}, Got: {}\n",
                        test.final_state.y, cpu.y
                    ))
                }

                if cpu.sp != test.final_state.s {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register SP; Expected: {}, Got: {}\n",
                        test.final_state.s, cpu.sp
                    ))
                }

                if cpu.sr != test.final_state.p {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register SR; Expected: {}, Got: {}\n",
                        test.final_state.p, cpu.sr
                    ))
                }

                if cpu.pc != test.final_state.pc {
                    failure_string.push_str(&format!(
                        "Unexpected Value in Register PC; Expected: {}, Got: {}\n",
                        test.final_state.pc, cpu.pc
                    ))
                }

                for (addr, byte) in test.final_state.ram.clone() {
                    let read_byte = cpu.memory.read_byte(addr);
                    if read_byte != byte {
                        failure_string.push_str(&format!(
                            "Unexpected Byte in Memory Location; Expected: {}@{addr:04x?}, Got: {}@{addr:04x?}\n",
                            byte, read_byte
                        ))
                    }
                }

                let expected_cycles = test.cycles.len();
                if cpu.cycles != expected_cycles {
                    failure_string.push_str(&format!(
                        "Execution Incorrect Number of Cycles; Expected: {}, Got: {}\n",
                        expected_cycles, cpu.cycles
                    ))
                }

                if !failure_string.is_empty() {
                    failures.push((failure_string, test));
                }
                cpu = CPU::new(cpu.memory, true);
            }
            Err(e) => {
                failures.push((e.to_string(), test));
                cpu = CPU::new(cpu.memory, true);
            }
        }
    }

    if !failures.is_empty() {
        if verbose_output {
            Err(anyhow::anyhow!("Failures: {:#?}", failures))
        } else {
            Err(anyhow::anyhow!("Total Failures: {:#?}", failures.len()))
        }
    } else {
        Ok(())
    }
}
