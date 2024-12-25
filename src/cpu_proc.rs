use crate::cpu::{CpuContext, emu_cycle};
use crate::common::{bit, bit_set};
use crate::instructions::{AddrMode, CondType,RegType};

impl<'a> CpuContext<'a> {
    pub fn cpu_set_flags(&mut self, z: u8, n: u8, h: u8, c: u8) {
        self.regs.f = bit_set!(self.regs.f, 7, z);
        self.regs.f = bit_set!(self.regs.f, 6, n);

        self.regs.f = bit_set!(self.regs.f, 5, h);
        self.regs.f = bit_set!(self.regs.f, 4, c);
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
                RegType::RtA => self.cpu_set_reg(
                    &inst.reg_1,
                    self.bus_read16(0xFF00 | self.fetched_data),
                ),
                _ => self.bus_write(
                    0xFF00 | self.fetched_data,
                    self.regs.a,
                ),
            }
        }
    }

    pub fn proc_jp(&mut self) {
        if self.check_condition() {
            self.regs.pc = self.fetched_data;
            emu_cycle(1);
        }
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