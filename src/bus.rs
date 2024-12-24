use crate::cart::CartRead;
use crate::ram::RamReadWrite;
use crate::cpu::CpuContext;
// 0x0000 - 0x3FFF : ROM Bank 0
// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
// 0x8000 - 0x97FF : CHR RAM
// 0x9800 - 0x9BFF : BG Map 1
// 0x9C00 - 0x9FFF : BG Map 2
// 0xA000 - 0xBFFF : Cartridge RAM
// 0xC000 - 0xCFFF : RAM Bank 0
// 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
// 0xE000 - 0xFDFF : Reserved - Echo RAM
// 0xFE00 - 0xFE9F : Object Attribute Memory
// 0xFEA0 - 0xFEFF : Reserved - Unusable
// 0xFF00 - 0xFF7F : I/O Registers
// 0xFF80 - 0xFFFE : Zero Pagezz
impl<'a> CpuContext<'a> {
    pub fn bus_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.cart_read(address), // ROM Data
            0x8000..=0x9FFF => panic!("UNSUPPORTED bus_read({:04X})", address), // Char/Map Data
            0xA000..=0xBFFF => self.cart.cart_read(address), // Cartridge RAM
            0xC000..=0xDFFF => self.ram.wram_read(address), // Working RAM
            0xE000..=0xFDFF => 0, // Echo RAM
            0xFE00..=0xFE9F => panic!("UNSUPPORTED bus_read({:04X})", address), // OAM
            0xFEA0..=0xFEFF => 0, // Unusable
            0xFF00..=0xFF7F => panic!("UNSUPPORTED bus_read({:04X})", address), // IO Registers
            0xFFFF => self.cpu_get_ie_register(), // IE Register
            _ => self.ram.hram_read(address), // HRAM
        }
    }

    pub fn bus_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.cart_write(address, value),
            0x8000..=0x9FFF => panic!("UNSUPPORTED bus_write({:04X})", address),
            0xA000..=0xBFFF => self.cart.cart_write(address, value),
            0xC000..=0xDFFF => self.ram.wram_write(address, value),
            0xE000..=0xFDFF => todo!("reserved echo ram"),
            0xFE00..=0xFE9F => panic!("UNSUPPORTED bus_write({:04X})", address),
            0xFEA0..=0xFEFF => panic!("unusable reserved"),
            0xFF00..=0xFF7F => panic!("UNSUPPORTED bus_write({:04X})", address),
            0xFFFF => self.cpu_set_ie_register(value),
            _ => self.ram.hram_write(address, value),
        }
    }

    pub fn bus_read16(&self, address: u16) -> u16 {
        let lo = self.bus_read(address) as u16;
        let hi = self.bus_read(address + 1) as u16;
        lo | (hi << 8)
    }

    pub fn bus_write16(&mut self, address: u16, value: u16) {
        self.bus_write(address + 1, ((value >> 8) & 0xFF) as u8);
        self.bus_write(address, (value & 0xFF) as u8);
    }
}