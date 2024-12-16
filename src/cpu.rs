use crate::bus::{self, bus_read};
use crate::cart::CartContext;
use crate::instructions::{self, AddrMode, RegType};
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
        self.regs.pc += 1;
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
        } else {
            panic!("Unknown Instruction");
        }
    }

    fn execute(&mut self) {
        println!("Not executed yet");
    }

    pub fn cpu_step(&mut self, cart: &CartContext) -> bool {
        if !self.halted {
            self.fetch_instruction(cart);
            self.fetch_data(cart);
            self.execute();
        }
        true
    }
}

fn reverse(n: u16) -> u16 {
    ((n & 0xFF00) >> 8) | ((n & 0x00FF) << 8)
}

fn emu_cycle(cpu_cycles: u8) {}
