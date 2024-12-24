pub struct RamContext {
    wram: [u8; 0x2000],
    hram: [u8; 0x80],
}

impl RamContext {
    pub fn new() -> Self {
        RamContext {
            wram: [0; 0x2000],
            hram: [0; 0x80],
        }
    }
}

pub trait RamReadWrite {
    fn wram_read(&self, address: u16) -> u8;
    fn wram_write(&mut self, address: u16, value: u8);
    fn hram_read(&self, address: u16) -> u8;
    fn hram_write(&mut self, address: u16, value: u8);
}

impl RamReadWrite for RamContext {
    fn wram_read(&self, addr: u16) -> u8 {
        let address = addr - 0xC000;
        if address >= 0x2000 {
            panic!("INVALID WRAM ADDR {:08X}", address + 0xC000)
        }
        self.wram[address as usize]
    }

    fn wram_write(&mut self, addr: u16, value: u8) {
        let address = addr - 0xC000;
        self.wram[address as usize] = value;
    }

    fn hram_read(&self, addr: u16) -> u8 {
        let address = addr -  0xFF80;
        self.hram[address as usize]
    }

    fn hram_write(&mut self, addr: u16, value: u8) {
        let address = addr - 0xFF80;
        self.hram[address as usize] = value;
    }
}
