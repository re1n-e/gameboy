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

    pub fn proc_ld(&mut self) {
        if self.dest_is_mem {
            if let Some(inst) = &self.cur_inst {
                match inst.reg_2 {
                    RegType::RtAf => {
                        emu_cycle(1);
                        self.bus_write16(self.mem_dest, self.fetched_data);
                    }
                    _ => self.bus_write(self.mem_dest, self.fetched_data as u8),
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

    pub fn proc_ldh(&mut self) {
        if let Some(inst) = self.cur_inst.clone() {
            match inst.reg_1 {
                RegType::RtA => {
                    self.cpu_set_reg(&inst.reg_1, self.bus_read16(0xFF00 | self.fetched_data))
                }
                _ => self.bus_write(0xFF00 | self.fetched_data, self.regs.a),
            }
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

        if let Some(inst) = &self.cur_inst.clone() {
            self.cpu_set_reg(&inst.reg_1, n);
            match inst.reg_1 {
                RegType::RtAf => {
                    self.cpu_set_reg(&inst.reg_1, n & 0xFFF0);
                }
                _ => (),
            }
        }
    }

    pub fn proc_push(&mut self) {
        if let Some(inst) = &self.cur_inst.clone() {
            let hi: u16 = (self.cpu_read_reg(&inst.reg_1) >> 8) & 0xFF;
            emu_cycle(1);
            self.stack_push(hi as u8);

            let lo: u16 = self.cpu_read_reg(&inst.reg_1) & 0xFF;
            emu_cycle(1);
            self.stack_push(lo as u8);

            emu_cycle(1);
        }
    }

    pub fn proc_rst(&mut self) {
        if let Some(inst) = &self.cur_inst {
            self.goto_addr(inst.param as u16, true);
        }
    }

    pub fn proc_ret(&mut self) {
        if let Some(inst) = &self.cur_inst {
            match inst.cond {
                CondType::CtNone => (),
                _ => emu_cycle(1),
            }
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

    pub fn get_flag_z(&self) -> bool {
        bit!(self.regs.f, 7) == 1
    }

    pub fn get_flag_c(&self) -> bool {
        bit!(self.regs.f, 4) == 1
    }
}
