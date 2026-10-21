use anyhow::Result;

use my6502::{
    cpu::CPU,
    memory::{Bus, Memory},
};

fn main() -> Result<()> {
    let mut memory = Bus::new();
    memory.write_byte(0, 0xe9);
    memory.write_byte(1, 12);
    let mut cpu = CPU::new(memory, false);
    cpu.pc = 0;
    cpu.a = 15;

    cpu.step()?;
    cpu.step()?;

    println!("{}", cpu);
    Ok(())
}
