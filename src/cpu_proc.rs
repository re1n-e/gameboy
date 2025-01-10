use crate::common::{bit, bit_set};
use crate::cpu::{emu_cycle, CpuContext};
use crate::instructions::{AddrMode, CondType, RegType};

impl<'a> CpuContext<'a> {
    pub fn cpu_set_flags(&mut self, z: u8, n: u8, h: u8, c: u8) {
        if z as i8 != -1 {
            self.regs.f = bit_set!(self.regs.f, 7, z);
        }
        if n as i8 != -1 {
            self.regs.f = bit_set!(self.regs.f, 6, n);
        }
        if h as i8 != -1 {
            self.regs.f = bit_set!(self.regs.f, 5, h);
        }
        if c as i8 != -1 {
            self.regs.f = bit_set!(self.regs.f, 4, c);
        }
    }

    pub fn proc_di(&mut self) {
        self.int_master_enable = false;
    }

    // Instruction processing methods
    pub fn proc_none(&self) {
        panic!("INVALID INSTRUCTION");
    }

    pub fn proc_xor(&mut self) {
        self.regs.a ^= self.fetched_data as u8 & 0xFF;
        self.cpu_set_flags(if self.regs.a == 0 { 1 } else { 0 }, 0, 0, 0);
    }

    fn is_16_bit(&self, rt: &RegType) -> bool {
        rt >= &RegType::RtAf
    }

    pub fn proc_ld(&mut self) {
        // Handle memory destination case first
        if self.dest_is_mem {
            // Extract needed values before any mutable operations
            let (is_16_bit, fetched_data, mem_dest) = if let Some(inst) = &self.cur_inst {
                (
                    self.is_16_bit(&inst.reg_2),
                    self.fetched_data,
                    self.mem_dest,
                )
            } else {
                return;
            };

            if is_16_bit {
                emu_cycle(1);
                self.bus_write16(mem_dest, fetched_data);
            } else {
                self.bus_write(mem_dest, fetched_data as u8);
            }
            emu_cycle(1);
            return;
        }

        // Extract all needed values before any mutations
        let (reg1, reg2, mode, fetched_data) = if let Some(inst) = &self.cur_inst {
            (
                inst.reg_1.clone(),
                inst.reg_2.clone(),
                inst.mode.clone(),
                self.fetched_data,
            )
        } else {
            return;
        };

        match mode {
            AddrMode::AmHlspr => {
                let reg2_val = self.cpu_read_reg(&reg2);
                let hflag =
                    (reg2_val & 0xF) as u8 + if (fetched_data & 0xF) >= 0x10 { 1 } else { 0 };
                let cflag =
                    (reg2_val & 0xFF) as u8 + if (fetched_data & 0xFF) >= 0x100 { 1 } else { 0 };

                self.cpu_set_flags(0, 0, hflag, cflag);
                self.cpu_set_reg(&reg1, reg2_val + fetched_data);
            }
            _ => self.cpu_set_reg(&reg1, fetched_data),
        }
    }

    pub fn proc_ldh(&mut self) {
        let (reg1, fetched_data) = if let Some(inst) = &self.cur_inst {
            (
                inst.reg_1.clone(),
                self.fetched_data,
            )
        } else {
            return;
        };

        match reg1 {
            RegType::RtA => self.cpu_set_reg(&reg1, self.bus_read16(0xFF00 | fetched_data)),
            _ => self.bus_write(0xFF00 | fetched_data, self.regs.a),
        }
    }

    fn goto_addr(&mut self, addr: u16, pushpc: bool) {
        if self.check_condition() {
            if pushpc {
                emu_cycle(2);
                self.stack_push16(self.regs.pc);
            }

            self.regs.pc = addr;
            emu_cycle(1);
        }
    }

    pub fn proc_jp(&mut self) {
        self.goto_addr(self.fetched_data, false);
    }

    pub fn proc_call(&mut self) {
        self.goto_addr(self.fetched_data, true);
    }

    pub fn proc_jr(&mut self) {
        let rel = (self.fetched_data & 0xFF) as i16;
        let addr: i16 = self.regs.pc as i16 + rel;
        self.goto_addr(addr as u16, false);
    }

    pub fn proc_pop(&mut self) {
        let lo: u16 = self.stack_pop() as u16;
        emu_cycle(1);
        let hi: u16 = self.stack_pop() as u16;
        emu_cycle(1);

        let n: u16 = (hi << 8) | lo;

        let (reg1, _) = if let Some(inst) = &self.cur_inst {
            (
                inst.reg_1.clone(),
                0,
            )
        } else {
            return;
        };

        match reg1 {
            RegType::RtAf => self.cpu_set_reg(&reg1, n & 0xFFF0),
            _ => self.cpu_set_reg(&reg1, n),
        }
    }

    pub fn proc_push(&mut self) {
        if let Some(inst) = &self.cur_inst {
            let reg_value = self.cpu_read_reg(&inst.reg_1);
            let hi: u16 = (reg_value >> 8) & 0xFF;
            emu_cycle(1);
            self.stack_push(hi as u8);

            let lo: u16 = reg_value & 0xFF;
            emu_cycle(1);
            self.stack_push(lo as u8);

            emu_cycle(1);
        }
    }

    pub fn proc_inc(&mut self) {
        let (reg1, mode) = if let Some(inst) = &self.cur_inst {
            (
                inst.reg_1.clone(),
                inst.mode.clone(),
            )
        } else {
            return;
        };

        if self.is_16_bit(&reg1) {
            emu_cycle(1);
        }

        if reg1 == RegType::RtHl && mode == AddrMode::AmMr {
            let addr = self.cpu_read_reg(&RegType::RtHl);
            let val = (self.bus_read(addr) as u16 + 1) & 0xFF;
            self.bus_write(addr, val as u8);
        } else {
            let val = self.cpu_read_reg(&reg1) + 1;
            self.cpu_set_reg(&reg1, val);
        }
    }

    pub fn proc_rst(&mut self) {
        if let Some(inst) = &self.cur_inst {
            self.goto_addr(inst.param as u16, true);
        }
    }

    pub fn proc_ret(&mut self) {
        let needs_extra_cycle = if let Some(inst) = &self.cur_inst {
            match inst.cond {
                CondType::CtNone => false,
                _ => true,
            }
        } else {
            false
        };

        if needs_extra_cycle {
            emu_cycle(1);
        }

        if self.check_condition() {
            let lo: u16 = self.stack_pop() as u16;
            emu_cycle(1);
            let hi: u16 = (self.stack_pop() as u16) << 8;
            emu_cycle(1);

            let n = hi | lo;
            self.regs.pc = n;

            emu_cycle(1);
        }
    }

    pub fn proc_reti(&mut self) {
        self.int_master_enable = true;
        self.proc_ret();
    }

    pub fn check_condition(&self) -> bool {
        if let Some(inst) = &self.cur_inst {
            let z: bool = self.get_flag_z();
            let c: bool = self.get_flag_c();

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

    pub fn get_flag_z(&self) -> bool {
        bit!(self.regs.f, 7) == 1
    }

    pub fn get_flag_c(&self) -> bool {
        bit!(self.regs.f, 4) == 1
    }
}
