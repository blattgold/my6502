pub trait Memory: Default {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, byte: u8);

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_word(&mut self, addr: u16, word: u16) {
        self.write_byte(addr, word as u8);
        self.write_byte(addr, (word >> 8) as u8);
    }
}

pub struct Bus {
    ram: [u8; 1024],
    rom: [u8; 4096],
}

impl Bus {
    pub fn new() -> Self {
        let ram = [0; 1024];
        let rom = [0; 4096];
        Self { ram, rom }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0..0x8000 => self.ram[(addr & 0x3FF) as usize],
            _ => self.rom[(addr & 0xFFF) as usize],
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0..0x8000 => self.ram[(addr & 0x3FF) as usize] = byte,
            _ => eprintln!("attempted to write '{byte}' to ROM at address: {addr}!"),
        };
    }
}
