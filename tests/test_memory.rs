use std::collections::HashMap;

use my6502::memory::Memory;

#[derive(Default)]
pub struct TestMemory {
    pub memory: HashMap<u16, u8>,
}

impl Memory for TestMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        if let Some(n) = self.memory.get(&addr) {
            *n
        } else {
            0
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory.insert(addr, byte);
    }

    fn clear(&mut self) {
        self.memory.clear();
    }
}
