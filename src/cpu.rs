use crate::bus::{self, bus_read, bus_read16, bus_write, bus_write16};
use crate::cart::CartContext;
use crate::common::{bit, bit_set};
use crate::instructions::{self, inst_name, AddrMode, CondType, InType, RegType};
use crate::ram::RamContext;
struct CpuRegister {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

impl CpuRegister {
    pub fn new() -> Self {
        CpuRegister {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
        }
    }
}

pub struct CpuContext<'a> {
    regs: CpuRegister,
    fetched_data: u16,
    mem_dest: u16,
    cur_opcode: u8,
    cur_inst: Option<instructions::Instruction>,
    ram: RamContext,
    cart: &'a mut CartContext,
    dest_is_mem: bool,
    halted: bool,
    int_master_enable: bool,
    ie_register: u8,
    stepping: bool,
}

impl<'a> CpuContext<'a> {
    pub fn new(cart: &'a mut CartContext) -> Self {
        CpuContext {
            regs: CpuRegister::new(),
            fetched_data: 0,
            mem_dest: 0,
            cur_opcode: 0,
            cur_inst: None,
            int_master_enable: true,
            ram: RamContext::new(),
            cart,
            dest_is_mem: false,
            ie_register: 0,
            halted: false,
            stepping: false,
        }
    }

    fn cpu_read_reg(&self, rt: &RegType) -> u16 {
        match rt {
            RegType::RtA => self.regs.a as u16,
            RegType::RtF => self.regs.f as u16,
            RegType::RtB => self.regs.b as u16,
            RegType::RtC => self.regs.c as u16,
            RegType::RtD => self.regs.d as u16,
            RegType::RtE => self.regs.e as u16,
            RegType::RtH => self.regs.h as u16,
            RegType::RtL => self.regs.l as u16,

            RegType::RtAf => reverse(self.regs.a as u16),
            RegType::RtBc => reverse(self.regs.b as u16),
            RegType::RtDe => reverse(self.regs.d as u16),
            RegType::RtHl => reverse(self.regs.h as u16),

            RegType::RtPc => self.regs.pc,
            RegType::RtSp => self.regs.sp,
            _ => 0,
        }
    }

    fn cpu_set_reg(&mut self, rt: &RegType, value: u16) {}

    fn fetch_instruction(&mut self) {
        self.cur_opcode = bus::bus_read(self.cart, &self.ram, self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.cur_inst = instructions::instruction_by_opcode(self.cur_opcode);
    }

    fn fetch_data(&mut self) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        if let Some(inst) = &self.cur_inst {
            match inst.mode {
                AddrMode::AmImp => return,
                AddrMode::AmR => self.fetched_data = self.cpu_read_reg(&inst.reg_1),
                AddrMode::AmRr => self.fetched_data = self.cpu_read_reg(&inst.reg_2),
                AddrMode::AmRD8 => {
                    self.fetched_data = bus::bus_read(self.cart, &self.ram, self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                }
                AddrMode::AmRD16 => (),
                AddrMode::AmD16 => {
                    let lo = bus::bus_read(self.cart, &self.ram, self.regs.pc) as u16;
                    emu_cycle(1);
                    let hi = bus::bus_read(self.cart, &self.ram, self.regs.pc + 1) as u16;
                    emu_cycle(1);
                    self.fetched_data = lo | (hi << 8);
                    self.regs.pc += 2;
                }
                AddrMode::AmMrr => {
                    self.fetched_data = self.cpu_read_reg(&inst.reg_2);
                    self.mem_dest = self.cpu_read_reg(&inst.reg_1);
                    self.dest_is_mem = true;

                    match inst.reg_1 {
                        RegType::RtC => self.mem_dest |= 0xFF00,
                        _ => (),
                    }
                }
                AddrMode::AmRMr => {
                    let mut addr: u16 = self.cpu_read_reg(&inst.reg_2);

                    match inst.reg_1 {
                        RegType::RtC => addr |= 0xFF00,
                        _ => (),
                    }

                    self.fetched_data = bus_read(self.cart, &self.ram, addr) as u16;
                }
                AddrMode::AmRhli => {
                    self.fetched_data =
                        bus_read(self.cart, &self.ram, self.cpu_read_reg(&inst.reg_2)) as u16;
                    emu_cycle(1);
                    self.cpu_set_reg(&RegType::RtHl, self.cpu_read_reg(&RegType::RtHl) + 1);
                }
                _ => panic!("Unknown addressing mode"),
            }
        }
    }

    fn execute(&mut self) {
        if let Some(inst) = &self.cur_inst {
            match inst.type_in {
                InType::InNone => self.proc_none(),
                InType::InLd => self.proc_ld(),
                InType::InLdh => self.proc_ldh(),
                InType::InJp => self.proc_jp(),
                InType::InDi => self.proc_di(),
                InType::InXor => self.proc_xor(),
                InType::InNop => (),
                _ => (),
            }
        }
    }

    pub fn cpu_step(&mut self) -> bool {
        if !self.halted {
            self.fetch_instruction();
            self.fetch_data();
            if let Some(inst) = &self.cur_inst {
                println!(
                    "PC: {:04X}  INST: {}  ({:02X} {:02X} {:02X}) A: {:02X} BC: {:02X}{:02X} DE: {:02X}{:02X} HL: {:02X}{:02X}",
                    self.regs.pc,
                    inst_name(&inst.type_in),
                    self.cur_opcode,
                    bus_read(self.cart, &self.ram, self.regs.pc + 1),
                    bus_read(self.cart, &self.ram, self.regs.pc + 2),
                    self.regs.a,
                    self.regs.b,
                    self.regs.c,
                    self.regs.d,
                    self.regs.e,
                    self.regs.h,
                    self.regs.l,
                );
            }
            self.execute();
        }
        true
    }

    fn proc_di(&mut self) {
        self.int_master_enable = false;
    }

    // Instruction processing methods
    fn proc_none(&self) {
        panic!("INVALID INSTRUCTION");
    }

    fn cpu_set_flags(&mut self, z: u8, n: u8, h: u8, c: u8) {
        self.regs.f = bit_set!(self.regs.f, 7, z);
        self.regs.f = bit_set!(self.regs.f, 6, n);

        self.regs.f = bit_set!(self.regs.f, 5, h);
        self.regs.f = bit_set!(self.regs.f, 4, c);
    }

    fn proc_xor(&mut self) {
        self.regs.a ^= self.fetched_data as u8 & 0xFF;
        self.cpu_set_flags(if self.regs.a == 0 { 1 } else { 0 }, 0, 0, 0);
    }

    fn proc_ld(&mut self) {
        if self.dest_is_mem {
            if let Some(inst) = &self.cur_inst {
                match inst.reg_2 {
                    RegType::RtAf => {
                        emu_cycle(1);
                        bus_write16(self.cart, &self.ram, self.mem_dest, self.fetched_data);
                    }
                    _ => bus_write(self.cart, &self.ram, self.mem_dest, self.fetched_data as u8),
                }
            }

            emu_cycle(1);

            return;
        }

        if let Some(inst) = self.cur_inst.clone() {
            match inst.mode {
                AddrMode::AmHlspr => {
                    let hflag: u8 = (self.cpu_read_reg(&inst.reg_2) & 0xF) as u8
                        + if (self.fetched_data & 0xF) >= 0x10 {
                            1
                        } else {
                            0
                        };

                    let cflag: u8 = (self.cpu_read_reg(&inst.reg_2) & 0xFF) as u8
                        + if (self.fetched_data & 0xFF) >= 0x100 {
                            1
                        } else {
                            0
                        };
                    let reg2_value = self.cpu_read_reg(&inst.reg_2);
                    self.cpu_set_flags(0, 0, hflag, cflag);
                    self.cpu_set_reg(&inst.reg_1, reg2_value + self.fetched_data);
                }
                _ => self.cpu_set_reg(&inst.reg_1, self.fetched_data),
            }
        }
    }

    fn proc_ldh(&mut self) {
        if let Some(inst) = self.cur_inst.clone() {
            match inst.reg_1 {
                RegType::RtA => self.cpu_set_reg(
                    &inst.reg_1,
                    bus_read16(self.cart, &self.ram, 0xFF00 | self.fetched_data),
                ),
                _ => bus_write(
                    self.cart,
                    &self.ram,
                    0xFF00 | self.fetched_data,
                    self.regs.a,
                ),
            }
        }
    }

    fn proc_jp(&mut self) {
        if self.check_condition() {
            self.regs.pc = self.fetched_data;
            emu_cycle(1);
        }
    }

    fn check_condition(&self) -> bool {
        let z: bool = self.get_flag_z();
        let c: bool = self.get_flag_c();

        if let Some(inst) = &self.cur_inst {
            match inst.cond {
                CondType::CtNone => true,
                CondType::CtC => c,
                CondType::CtNc => !c,
                CondType::CtZ => z,
                CondType::CtNz => !z,
            }
        } else {
            false
        }
    }

    fn get_flag_z(&self) -> bool {
        bit!(self.regs.f, 7) == 1
    }

    fn get_flag_c(&self) -> bool {
        bit!(self.regs.f, 4) == 1
    }
}

fn reverse(n: u16) -> u16 {
    ((n & 0xFF00) >> 8) | ((n & 0x00FF) << 8)
}

fn emu_cycle(cpu_cycles: u8) {}
