use anyhow::Result;

use crate::{
    cpu::CPU,
    memory::{Bus, Memory},
};

pub mod cpu;
pub mod instruction;
pub mod memory;

fn main() -> Result<()> {
    let mut memory = Bus::new();
    memory.write_byte(0, 0xe9);
    memory.write_byte(1, 12);
    let mut cpu = CPU::new(memory, false);
    cpu.pc = 0;
    cpu.a = 15;

    println!("A: {}", cpu.a);
    cpu.step()?;
    println!("A: {}", cpu.a);
    cpu.step()?;
    println!("A: {}", cpu.a);
    Ok(())
}
