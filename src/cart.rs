use std::fs::File;
use std::io::{self, Read};
use std::mem::MaybeUninit;

#[derive(Debug)]
struct RomHeader {
    entry: [u8; 4],
    logo: [u8; 0x30],
    title: [u8; 16],
    new_lic_code: u16,
    sgb_flag: u8,
    card_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    lic_code: u8,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

impl RomHeader {
    fn title_as_string(&self) -> String {
        self.title
            .iter()
            .take_while(|&&x| x != 0)
            .cloned()
            .map(|b| b as char)
            .collect()
    }
}

pub struct CartContext {
    filename: String,
    rom_size: u32,
    rom_data: Vec<u8>,
    header: Option<RomHeader>,
}

const ROM_TYPES: [&str; 35] = [
    "ROM ONLY",
    "MBC1",
    "MBC1+RAM",
    "MBC1+RAM+BATTERY",
    "0x04 ???",
    "MBC2",
    "MBC2+BATTERY",
    "0x07 ???",
    "ROM+RAM 1",
    "ROM+RAM+BATTERY 1",
    "0x0A ???",
    "MMM01",
    "MMM01+RAM",
    "MMM01+RAM+BATTERY",
    "0x0E ???",
    "MBC3+TIMER+BATTERY",
    "MBC3+TIMER+RAM+BATTERY 2",
    "MBC3",
    "MBC3+RAM 2",
    "MBC3+RAM+BATTERY 2",
    "0x14 ???",
    "0x15 ???",
    "0x16 ???",
    "0x17 ???",
    "0x18 ???",
    "MBC5",
    "MBC5+RAM",
    "MBC5+RAM+BATTERY",
    "MBC5+RUMBLE",
    "MBC5+RUMBLE+RAM",
    "MBC5+RUMBLE+RAM+BATTERY",
    "0x1F ???",
    "MBC6",
    "0x21 ???",
    "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
];

fn lic_code(code: u8) -> String {
    match code {
        0x00 => String::from("None"),
        0x01 => String::from("Nintendo R&D1"),
        0x08 => String::from("Capcom"),
        0x13 => String::from("Electronic Arts"),
        0x18 => String::from("Hudson Soft"),
        0x19 => String::from("b-ai"),
        0x20 => String::from("kss"),
        0x22 => String::from("pow"),
        0x24 => String::from("PCM Complete"),
        0x25 => String::from("san-x"),
        0x28 => String::from("Kemco Japan"),
        0x29 => String::from("seta"),
        0x30 => String::from("Viacom"),
        0x31 => String::from("Nintendo"),
        0x32 => String::from("Bandai"),
        0x33 => String::from("Ocean/Acclaim"),
        0x34 => String::from("Konami"),
        0x35 => String::from("Hector"),
        0x37 => String::from("Taito"),
        0x38 => String::from("Hudson"),
        0x39 => String::from("Banpresto"),
        0x41 => String::from("Ubi Soft"),
        0x42 => String::from("Atlus"),
        0x44 => String::from("Malibu"),
        0x46 => String::from("angel"),
        0x47 => String::from("Bullet-Proof"),
        0x49 => String::from("irem"),
        0x50 => String::from("Absolute"),
        0x51 => String::from("Acclaim"),
        0x52 => String::from("Activision"),
        0x53 => String::from("American sammy"),
        0x54 => String::from("Konami"),
        0x55 => String::from("Hi tech entertainment"),
        0x56 => String::from("LJN"),
        0x57 => String::from("Matchbox"),
        0x58 => String::from("Mattel"),
        0x59 => String::from("Milton Bradley"),
        0x60 => String::from("Titus"),
        0x61 => String::from("Virgin"),
        0x64 => String::from("LucasArts"),
        0x67 => String::from("Ocean"),
        0x69 => String::from("Electronic Arts"),
        0x70 => String::from("Infogrames"),
        0x71 => String::from("Interplay"),
        0x72 => String::from("Broderbund"),
        0x73 => String::from("sculptured"),
        0x75 => String::from("sci"),
        0x78 => String::from("THQ"),
        0x79 => String::from("Accolade"),
        0x80 => String::from("misawa"),
        0x83 => String::from("lozc"),
        0x86 => String::from("Tokuma Shoten Intermedia"),
        0x87 => String::from("Tsukuda Original"),
        0x91 => String::from("Chunsoft"),
        0x92 => String::from("Video system"),
        0x93 => String::from("Ocean/Acclaim"),
        0x95 => String::from("Varie"),
        0x96 => String::from("Yonezawa/s’pal"),
        0x97 => String::from("Kaneko"),
        0x99 => String::from("Pack in soft"),
        0xA4 => String::from("Konami (Yu-Gi-Oh!)"),
        _ => String::from("Unknown"),
    }
}

impl CartContext {
    pub fn new() -> Self {
        CartContext {
            filename: String::new(),
            rom_size: 0,
            rom_data: Vec::new(),
            header: None,
        }
    }

    pub fn cart_load(&mut self, cart: &str) -> io::Result<bool> {
        self.filename = cart.to_string();

        let mut file = File::open(cart)?;
        println!("Opened: {}", self.filename);

        // Get file size (ROM size)
        self.rom_size = file.metadata()?.len() as u32;

        // Read the ROM data
        self.rom_data.clear();
        file.read_to_end(&mut self.rom_data)?;

        let header_offset = 0x100;
        if self.rom_data.len() > header_offset + std::mem::size_of::<RomHeader>() {
            let mut header: RomHeader = unsafe { MaybeUninit::zeroed().assume_init() };

            header
                .entry
                .copy_from_slice(&self.rom_data[header_offset..header_offset + 4]);
            header
                .logo
                .copy_from_slice(&self.rom_data[header_offset + 4..header_offset + 0x34]);
            header
                .title
                .copy_from_slice(&self.rom_data[header_offset + 0x34..header_offset + 0x44]);

            // Manually copy the rest of the header fields
            header.new_lic_code = u16::from_le_bytes([
                self.rom_data[header_offset + 0x44],
                self.rom_data[header_offset + 0x45],
            ]);
            header.sgb_flag = self.rom_data[header_offset + 0x46];
            header.card_type = self.rom_data[header_offset + 0x47];
            header.rom_size = self.rom_data[header_offset + 0x48];
            header.ram_size = self.rom_data[header_offset + 0x49];
            header.dest_code = self.rom_data[header_offset + 0x4A];
            header.lic_code = self.rom_data[header_offset + 0x4B];
            header.version = self.rom_data[header_offset + 0x4C];
            header.checksum = self.rom_data[header_offset + 0x4D];
            header.global_checksum = u16::from_le_bytes([
                self.rom_data[header_offset + 0x4E],
                self.rom_data[header_offset + 0x4F],
            ]);

            self.header = Some(header);
        }

        if let Some(header) = &self.header {
            println!("Cartridge Loaded:");
            println!("\t Title    : {}", header.title_as_string());
            println!(
                "\t Type     : {:02X} ({})",
                header.card_type,
                self.cart_type_name()
            );
            println!("\t ROM Size : {} KB", 32 << header.rom_size);
            println!("\t RAM Size : {:02X}", header.ram_size);
            println!(
                "\t LIC Code : {:02X} ({})",
                header.lic_code,
                self.cart_lic_name()
            );
            println!("\t ROM Vers : {:02X}", header.version);

            // Checksum calculation
            let mut checksum = 0u16;
            for i in 0x0134..=0x014C {
                checksum = checksum
                    .wrapping_sub(self.rom_data[i as usize] as u16)
                    .wrapping_sub(1);
            }

            println!(
                "\t Checksum : {:02X} ({})",
                header.checksum,
                if (checksum & 0xFF) == 0 {
                    "FAILED"
                } else {
                    "PASSED"
                }
            );
        }

        Ok(true)
    }

    pub fn cart_lic_name(&self) -> String {
        if let Some(header) = &self.header {
            if header.new_lic_code <= 0xA4 {
                return lic_code(header.lic_code);
            }
        }
        String::from("Unknown")
    }

    pub fn cart_type_name(&self) -> String {
        if let Some(header) = &self.header {
            if header.card_type <= 0x22 {
                return ROM_TYPES[header.card_type as usize].to_string();
            }
        }
        String::from("Unknown")
    }
}

pub trait CartRead {
    fn cart_read(&self, address: u16) -> u8;
    fn cart_write(&self, address: u16, value: u8);
}
//
impl CartRead for CartContext {
    fn cart_read(&self, address: u16) -> u8 {
        self.rom_data[address as usize]
    }

    fn cart_write(&self, address: u16, value: u8) {
        panic!("Not implemented cart write!");
    }
}
