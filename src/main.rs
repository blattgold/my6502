use anyhow::Result;

use crate::{
    cpu::CPU,
    memory::{Bus, Memory},
};

mod cpu;
mod instruction;
mod memory;

fn main() -> Result<()> {
    let mut memory = Bus::new();
    memory.write_byte(0, 0xe9);
    memory.write_byte(1, 12);
    let mut cpu = CPU::new(memory);
    cpu.pc = 0;
    cpu.a = 15;

    cpu.step()?;
    println!("A: {}", cpu.a);
    Ok(())
}
