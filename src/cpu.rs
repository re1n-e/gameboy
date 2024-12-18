use crate::bus::{self, bus_read};
use crate::cart::CartContext;
use crate::common::{bit, bit_set};
use crate::instructions::{self, inst_name, AddrMode, CondType, InType, RegType};
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

pub struct CpuContext {
    regs: CpuRegister,
    fetched_data: u16,
    mem_dest: u16,
    cur_opcode: u8,
    cur_inst: Option<instructions::Instruction>,
    dest_is_mem: bool,
    halted: bool,
    int_master_enable: bool,
    stepping: bool,
}

impl CpuContext {
    pub fn new() -> Self {
        CpuContext {
            regs: CpuRegister::new(),
            fetched_data: 0,
            mem_dest: 0,
            cur_opcode: 0,
            cur_inst: None,
            int_master_enable: true,
            dest_is_mem: false,
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

    fn fetch_instruction(&mut self, cart: &CartContext) {
        self.cur_opcode = bus::bus_read(cart, self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.cur_inst = instructions::instruction_by_opcode(self.cur_opcode);
    }

    fn fetch_data(&mut self, cart: &CartContext) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        if let Some(inst) = &self.cur_inst {
            match inst.mode {
                AddrMode::AmImp => return,
                AddrMode::AmR => self.fetched_data = self.cpu_read_reg(&inst.reg_1),
                AddrMode::AmRD8 => {
                    self.fetched_data = bus::bus_read(cart, self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                }
                AddrMode::AmD16 => {
                    let lo = bus::bus_read(cart, self.regs.pc) as u16;
                    emu_cycle(1);
                    let hi = bus::bus_read(cart, self.regs.pc + 1) as u16;
                    emu_cycle(1);
                    self.fetched_data = lo | (hi << 8);
                    self.regs.pc += 2;
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
                InType::InJp => self.proc_jp(),
                InType::InDi => self.proc_di(),
                InType::InXor => self.proc_xor(),
                InType::InNop => (),
                _ => (),
            }
        } 
    }

    pub fn cpu_step(&mut self, cart: &CartContext) -> bool {
        if !self.halted {
            self.fetch_instruction(cart);
            self.fetch_data(cart);
            if let Some(inst) = &self.cur_inst {
                println!(
                    "PC: {:04X}  INST: {}  ({:02X} {:02X} {:02X}) A: {:02X} B: {:02X} C: {:02X}",
                    self.regs.pc,
                    inst_name(&inst.type_in),
                    self.cur_opcode,
                    bus_read(cart, self.regs.pc + 1),
                    bus_read(cart, self.regs.pc + 2),
                    self.regs.a,
                    self.regs.b,
                    self.regs.c
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

    fn cpu_set_flags(&mut self, z: i8, n: i8, h: i8, c: i8) {
        if z != -1 {
            self.regs.f = bit_set!(self.regs.f, 7, z);
        }

        if n != -1 {
            self.regs.f = bit_set!(self.regs.f, 6, n);
        }

        if h != -1 {
            self.regs.f = bit_set!(self.regs.f, 5, h);
        }

        if c != -1 {
            self.regs.f = bit_set!(self.regs.f, 4, c);
        }
    }

    fn proc_xor(&mut self) {
        self.regs.a ^= self.fetched_data as u8 & 0xFF;
        self.cpu_set_flags(if self.regs.a == 0 {1} else {0}, 0, 0, 0);
    }

    fn proc_ld(&self) {
        // TODO: Implement load instruction
        todo!("Load instruction not implemented");
    }

    fn proc_jp(&mut self) {
        if self.check_condition() {
            self.regs.pc = self.fetched_data;
            emu_cycle(1);
        }
    }

    // Condition checking method
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

    // Flag checking methods
    fn get_flag_z(&self) -> bool {
        bit!(self.regs.f, 7) == 1
    }

    fn get_flag_c(&self) -> bool {
        bit!(self.regs.f, 4) == 1
    }
}

// Utility function to reverse 16-bit value
fn reverse(n: u16) -> u16 {
    ((n & 0xFF00) >> 8) | ((n & 0x00FF) << 8)
}

// Emulation cycle function (placeholder)
fn emu_cycle(cpu_cycles: u8) {}
