use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use test_case::test_case;

mod json_data;
mod test_memory;
use json_data::*;
use test_memory::*;

use my6502::{cpu::CPU, memory::Memory};

#[test_case(0 ; "opcode 0x00")]
#[test_case(1 ; "opcode 0x01")]
#[test_case(2 ; "opcode 0x02")]
#[test_case(3 ; "opcode 0x03")]
#[test_case(4 ; "opcode 0x04")]
#[test_case(5 ; "opcode 0x05")]
#[test_case(6 ; "opcode 0x06")]
#[test_case(7 ; "opcode 0x07")]
#[test_case(8 ; "opcode 0x08")]
#[test_case(9 ; "opcode 0x09")]
#[test_case(10 ; "opcode 0x0a")]
#[test_case(11 ; "opcode 0x0b")]
#[test_case(12 ; "opcode 0x0c")]
#[test_case(13 ; "opcode 0x0d")]
#[test_case(14 ; "opcode 0x0e")]
#[test_case(15 ; "opcode 0x0f")]
#[test_case(16 ; "opcode 0x10")]
#[test_case(17 ; "opcode 0x11")]
#[test_case(18 ; "opcode 0x12")]
#[test_case(19 ; "opcode 0x13")]
#[test_case(20 ; "opcode 0x14")]
#[test_case(21 ; "opcode 0x15")]
#[test_case(22 ; "opcode 0x16")]
#[test_case(23 ; "opcode 0x17")]
#[test_case(24 ; "opcode 0x18")]
#[test_case(25 ; "opcode 0x19")]
#[test_case(26 ; "opcode 0x1a")]
#[test_case(27 ; "opcode 0x1b")]
#[test_case(28 ; "opcode 0x1c")]
#[test_case(29 ; "opcode 0x1d")]
#[test_case(30 ; "opcode 0x1e")]
#[test_case(31 ; "opcode 0x1f")]
#[test_case(32 ; "opcode 0x20")]
#[test_case(33 ; "opcode 0x21")]
#[test_case(34 ; "opcode 0x22")]
#[test_case(35 ; "opcode 0x23")]
#[test_case(36 ; "opcode 0x24")]
#[test_case(37 ; "opcode 0x25")]
#[test_case(38 ; "opcode 0x26")]
#[test_case(39 ; "opcode 0x27")]
#[test_case(40 ; "opcode 0x28")]
#[test_case(41 ; "opcode 0x29")]
#[test_case(42 ; "opcode 0x2a")]
#[test_case(43 ; "opcode 0x2b")]
#[test_case(44 ; "opcode 0x2c")]
#[test_case(45 ; "opcode 0x2d")]
#[test_case(46 ; "opcode 0x2e")]
#[test_case(47 ; "opcode 0x2f")]
#[test_case(48 ; "opcode 0x30")]
#[test_case(49 ; "opcode 0x31")]
#[test_case(50 ; "opcode 0x32")]
#[test_case(51 ; "opcode 0x33")]
#[test_case(52 ; "opcode 0x34")]
#[test_case(53 ; "opcode 0x35")]
#[test_case(54 ; "opcode 0x36")]
#[test_case(55 ; "opcode 0x37")]
#[test_case(56 ; "opcode 0x38")]
#[test_case(57 ; "opcode 0x39")]
#[test_case(58 ; "opcode 0x3a")]
#[test_case(59 ; "opcode 0x3b")]
#[test_case(60 ; "opcode 0x3c")]
#[test_case(61 ; "opcode 0x3d")]
#[test_case(62 ; "opcode 0x3e")]
#[test_case(63 ; "opcode 0x3f")]
#[test_case(64 ; "opcode 0x40")]
#[test_case(65 ; "opcode 0x41")]
#[test_case(66 ; "opcode 0x42")]
#[test_case(67 ; "opcode 0x43")]
#[test_case(68 ; "opcode 0x44")]
#[test_case(69 ; "opcode 0x45")]
#[test_case(70 ; "opcode 0x46")]
#[test_case(71 ; "opcode 0x47")]
#[test_case(72 ; "opcode 0x48")]
#[test_case(73 ; "opcode 0x49")]
#[test_case(74 ; "opcode 0x4a")]
#[test_case(75 ; "opcode 0x4b")]
#[test_case(76 ; "opcode 0x4c")]
#[test_case(77 ; "opcode 0x4d")]
#[test_case(78 ; "opcode 0x4e")]
#[test_case(79 ; "opcode 0x4f")]
#[test_case(80 ; "opcode 0x50")]
#[test_case(81 ; "opcode 0x51")]
#[test_case(82 ; "opcode 0x52")]
#[test_case(83 ; "opcode 0x53")]
#[test_case(84 ; "opcode 0x54")]
#[test_case(85 ; "opcode 0x55")]
#[test_case(86 ; "opcode 0x56")]
#[test_case(87 ; "opcode 0x57")]
#[test_case(88 ; "opcode 0x58")]
#[test_case(89 ; "opcode 0x59")]
#[test_case(90 ; "opcode 0x5a")]
#[test_case(91 ; "opcode 0x5b")]
#[test_case(92 ; "opcode 0x5c")]
#[test_case(93 ; "opcode 0x5d")]
#[test_case(94 ; "opcode 0x5e")]
#[test_case(95 ; "opcode 0x5f")]
#[test_case(96 ; "opcode 0x60")]
#[test_case(97 ; "opcode 0x61")]
#[test_case(98 ; "opcode 0x62")]
#[test_case(99 ; "opcode 0x63")]
#[test_case(100 ; "opcode 0x64")]
#[test_case(101 ; "opcode 0x65")]
#[test_case(102 ; "opcode 0x66")]
#[test_case(103 ; "opcode 0x67")]
#[test_case(104 ; "opcode 0x68")]
#[test_case(105 ; "opcode 0x69")]
#[test_case(106 ; "opcode 0x6a")]
#[test_case(107 ; "opcode 0x6b")]
#[test_case(108 ; "opcode 0x6c")]
#[test_case(109 ; "opcode 0x6d")]
#[test_case(110 ; "opcode 0x6e")]
#[test_case(111 ; "opcode 0x6f")]
#[test_case(112 ; "opcode 0x70")]
#[test_case(113 ; "opcode 0x71")]
#[test_case(114 ; "opcode 0x72")]
#[test_case(115 ; "opcode 0x73")]
#[test_case(116 ; "opcode 0x74")]
#[test_case(117 ; "opcode 0x75")]
#[test_case(118 ; "opcode 0x76")]
#[test_case(119 ; "opcode 0x77")]
#[test_case(120 ; "opcode 0x78")]
#[test_case(121 ; "opcode 0x79")]
#[test_case(122 ; "opcode 0x7a")]
#[test_case(123 ; "opcode 0x7b")]
#[test_case(124 ; "opcode 0x7c")]
#[test_case(125 ; "opcode 0x7d")]
#[test_case(126 ; "opcode 0x7e")]
#[test_case(127 ; "opcode 0x7f")]
#[test_case(128 ; "opcode 0x80")]
#[test_case(129 ; "opcode 0x81")]
#[test_case(130 ; "opcode 0x82")]
#[test_case(131 ; "opcode 0x83")]
#[test_case(132 ; "opcode 0x84")]
#[test_case(133 ; "opcode 0x85")]
#[test_case(134 ; "opcode 0x86")]
#[test_case(135 ; "opcode 0x87")]
#[test_case(136 ; "opcode 0x88")]
#[test_case(137 ; "opcode 0x89")]
#[test_case(138 ; "opcode 0x8a")]
#[test_case(139 ; "opcode 0x8b")]
#[test_case(140 ; "opcode 0x8c")]
#[test_case(141 ; "opcode 0x8d")]
#[test_case(142 ; "opcode 0x8e")]
#[test_case(143 ; "opcode 0x8f")]
#[test_case(144 ; "opcode 0x90")]
#[test_case(145 ; "opcode 0x91")]
#[test_case(146 ; "opcode 0x92")]
#[test_case(147 ; "opcode 0x93")]
#[test_case(148 ; "opcode 0x94")]
#[test_case(149 ; "opcode 0x95")]
#[test_case(150 ; "opcode 0x96")]
#[test_case(151 ; "opcode 0x97")]
#[test_case(152 ; "opcode 0x98")]
#[test_case(153 ; "opcode 0x99")]
#[test_case(154 ; "opcode 0x9a")]
#[test_case(155 ; "opcode 0x9b")]
#[test_case(156 ; "opcode 0x9c")]
#[test_case(157 ; "opcode 0x9d")]
#[test_case(158 ; "opcode 0x9e")]
#[test_case(159 ; "opcode 0x9f")]
#[test_case(160 ; "opcode 0xa0")]
#[test_case(161 ; "opcode 0xa1")]
#[test_case(162 ; "opcode 0xa2")]
#[test_case(163 ; "opcode 0xa3")]
#[test_case(164 ; "opcode 0xa4")]
#[test_case(165 ; "opcode 0xa5")]
#[test_case(166 ; "opcode 0xa6")]
#[test_case(167 ; "opcode 0xa7")]
#[test_case(168 ; "opcode 0xa8")]
#[test_case(169 ; "opcode 0xa9")]
#[test_case(170 ; "opcode 0xaa")]
#[test_case(171 ; "opcode 0xab")]
#[test_case(172 ; "opcode 0xac")]
#[test_case(173 ; "opcode 0xad")]
#[test_case(174 ; "opcode 0xae")]
#[test_case(175 ; "opcode 0xaf")]
#[test_case(176 ; "opcode 0xb0")]
#[test_case(177 ; "opcode 0xb1")]
#[test_case(178 ; "opcode 0xb2")]
#[test_case(179 ; "opcode 0xb3")]
#[test_case(180 ; "opcode 0xb4")]
#[test_case(181 ; "opcode 0xb5")]
#[test_case(182 ; "opcode 0xb6")]
#[test_case(183 ; "opcode 0xb7")]
#[test_case(184 ; "opcode 0xb8")]
#[test_case(185 ; "opcode 0xb9")]
#[test_case(186 ; "opcode 0xba")]
#[test_case(187 ; "opcode 0xbb")]
#[test_case(188 ; "opcode 0xbc")]
#[test_case(189 ; "opcode 0xbd")]
#[test_case(190 ; "opcode 0xbe")]
#[test_case(191 ; "opcode 0xbf")]
#[test_case(192 ; "opcode 0xc0")]
#[test_case(193 ; "opcode 0xc1")]
#[test_case(194 ; "opcode 0xc2")]
#[test_case(195 ; "opcode 0xc3")]
#[test_case(196 ; "opcode 0xc4")]
#[test_case(197 ; "opcode 0xc5")]
#[test_case(198 ; "opcode 0xc6")]
#[test_case(199 ; "opcode 0xc7")]
#[test_case(200 ; "opcode 0xc8")]
#[test_case(201 ; "opcode 0xc9")]
#[test_case(202 ; "opcode 0xca")]
#[test_case(203 ; "opcode 0xcb")]
#[test_case(204 ; "opcode 0xcc")]
#[test_case(205 ; "opcode 0xcd")]
#[test_case(206 ; "opcode 0xce")]
#[test_case(207 ; "opcode 0xcf")]
#[test_case(208 ; "opcode 0xd0")]
#[test_case(209 ; "opcode 0xd1")]
#[test_case(210 ; "opcode 0xd2")]
#[test_case(211 ; "opcode 0xd3")]
#[test_case(212 ; "opcode 0xd4")]
#[test_case(213 ; "opcode 0xd5")]
#[test_case(214 ; "opcode 0xd6")]
#[test_case(215 ; "opcode 0xd7")]
#[test_case(216 ; "opcode 0xd8")]
#[test_case(217 ; "opcode 0xd9")]
#[test_case(218 ; "opcode 0xda")]
#[test_case(219 ; "opcode 0xdb")]
#[test_case(220 ; "opcode 0xdc")]
#[test_case(221 ; "opcode 0xdd")]
#[test_case(222 ; "opcode 0xde")]
#[test_case(223 ; "opcode 0xdf")]
#[test_case(224 ; "opcode 0xe0")]
#[test_case(225 ; "opcode 0xe1")]
#[test_case(226 ; "opcode 0xe2")]
#[test_case(227 ; "opcode 0xe3")]
#[test_case(228 ; "opcode 0xe4")]
#[test_case(229 ; "opcode 0xe5")]
#[test_case(230 ; "opcode 0xe6")]
#[test_case(231 ; "opcode 0xe7")]
#[test_case(232 ; "opcode 0xe8")]
#[test_case(233 ; "opcode 0xe9")]
#[test_case(234 ; "opcode 0xea")]
#[test_case(235 ; "opcode 0xeb")]
#[test_case(236 ; "opcode 0xec")]
#[test_case(237 ; "opcode 0xed")]
#[test_case(238 ; "opcode 0xee")]
#[test_case(239 ; "opcode 0xef")]
#[test_case(240 ; "opcode 0xf0")]
#[test_case(241 ; "opcode 0xf1")]
#[test_case(242 ; "opcode 0xf2")]
#[test_case(243 ; "opcode 0xf3")]
#[test_case(244 ; "opcode 0xf4")]
#[test_case(245 ; "opcode 0xf5")]
#[test_case(246 ; "opcode 0xf6")]
#[test_case(247 ; "opcode 0xf7")]
#[test_case(248 ; "opcode 0xf8")]
#[test_case(249 ; "opcode 0xf9")]
#[test_case(250 ; "opcode 0xfa")]
#[test_case(251 ; "opcode 0xfb")]
#[test_case(252 ; "opcode 0xfc")]
#[test_case(253 ; "opcode 0xfd")]
#[test_case(254 ; "opcode 0xfe")]
#[test_case(255 ; "opcode 0xff")]
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
