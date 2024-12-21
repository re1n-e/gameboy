use crate::cart::CartRead;
use crate::ram::RamReadWrite;
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
// 0xFF80 - 0xFFFE : Zero Page
pub fn bus_read(cart: &impl CartRead, ram: &impl RamReadWrite, address: u16) -> u8 {
    if address < 0x8000 {
        //ROM Data
        return cart.cart_read(address);
    } else if address < 0xA000 {
        //Char/Map Data
        //TODO
        panic!("UNSUPPORTED bus_read({:04X})", address);
    } else if address < 0xC000 {
        //Cartridge RAM
        return cart.cart_read(address);
    } else if address < 0xE000 {
        //WRAM (Working RAM)
        return ram.wram_read(&mut address);
    } else if address < 0xFE00 {
        //reserved echo ram...
        return 0;
    } else if address < 0xFEA0 {
        //OAM
        //TODO
        panic!("UNSUPPORTED bus_read({:04X})", address);
    } else if address < 0xFF00 {
        //reserved unusable...
        return 0;
    } else if address < 0xFF80 {
        //IO Registers...
        //TODO
        panic!("UNSUPPORTED bus_read({:04X})", address);
    } else if address == 0xFFFF {
        //CPU ENABLE REGISTER...
        //TODO
        return cpu_get_ie_register();
    }

    //NO_IMPL
    return ram.hram_read(&mut address);
}

pub fn bus_write(cart: &impl CartRead, ram: &impl RamReadWrite, address: u16, value: u8) {
    if address < 0x8000 {
        cart.cart_write(address, value);
    } else if address < 0xA000 {
        //Char/Map Data
        //TODO
        panic!("UNSUPPORTED bus_write({:04X})", address);
    } else if address < 0xC000 {
        //EXT-RAM
        cart.cart_write(address, value);
    } else if address < 0xE000 {
        //WRAM
        ram.wram_write(&mut address, value);
    } else if address < 0xFE00 {
        todo!("reserved echo ram");
    } else if address < 0xFEA0 {
        //OAM
        //TODO
        panic!("UNSUPPORTED bus_write({:04X})", address);
    } else if address < 0xFF00 {
        panic!("unusable reserved");
    } else if address < 0xFF80 {
        //IO Registers...
        //TODO
        panic!("UNSUPPORTED bus_write({:04X})", address);
        //NO_IMPL
    } else if address == 0xFFFF {
        //CPU SET ENABLE REGISTER

        cpu_set_ie_register(value);
    } else {
        ram.hram_write(&mut address, value);
    }
}

pub fn bus_read16(cart: &impl CartRead, ram: &impl RamReadWrite, address: u16) -> u16 {
    let lo: u16 = bus_read(cart, ram, address) as u16;
    let hi: u16 = bus_read(cart, ram, address + 1) as u16;
    lo | (hi << 8)
}

pub fn bus_write16(cart: &impl CartRead, ram: &impl RamReadWrite, address: u16, value: u16) {
    bus_write(cart, ram, address + 1, (value >> 8) as u8 & 0xFF);
    bus_write(cart, ram, address, value as u8 & 0xFF);
}
