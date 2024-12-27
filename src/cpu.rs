use crate::cart::CartContext;
use crate::instructions::{self, inst_name, InType};
use crate::ram::RamContext;
pub struct CpuRegister {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
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
    pub regs: CpuRegister,
    pub fetched_data: u16,
    pub mem_dest: u16,
    pub cur_opcode: u8,
    pub cur_inst: Option<instructions::Instruction>,
    pub ram: RamContext,
    pub cart: &'a mut CartContext,
    pub dest_is_mem: bool,
    pub halted: bool,
    pub int_master_enable: bool,
    pub ie_register: u8,
    pub stepping: bool,
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

    fn fetch_instruction(&mut self) {
        self.cur_opcode = self.bus_read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.cur_inst = instructions::instruction_by_opcode(self.cur_opcode);
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
                InType::InPop => self.proc_pop(),
                InType::InPush => self.proc_push(),
                InType::InJr => self.proc_jr(),
                InType::InCall => self.proc_call(),
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
                    self.bus_read(self.regs.pc + 1),
                    self.bus_read(self.regs.pc + 2),
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

    pub fn cpu_get_ie_register(&self) -> u8 {
        self.ie_register
    }

    pub fn cpu_set_ie_register(&mut self, n: u8) {
        self.ie_register = n;
    }
}

pub fn emu_cycle(cpu_cycles: u8) {}
