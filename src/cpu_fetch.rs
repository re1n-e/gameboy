use crate::cpu::{CpuContext, emu_cycle};
use crate::instructions::{AddrMode, RegType};

impl<'a> CpuContext<'a> {
    pub fn fetch_data(&mut self) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        if let Some(inst) = &self.cur_inst {
            match inst.mode {
                AddrMode::AmImp => return,
                AddrMode::AmR => self.fetched_data = self.cpu_read_reg(&inst.reg_1),
                AddrMode::AmRr => self.fetched_data = self.cpu_read_reg(&inst.reg_2),
                AddrMode::AmRD8 => {
                    self.fetched_data = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                }
                AddrMode::AmRD16 => (),
                AddrMode::AmD16 => {
                    let lo = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    let hi = self.bus_read(self.regs.pc.wrapping_add(1)) as u16;
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

                    self.fetched_data = self.bus_read(addr) as u16;
                }
                AddrMode::AmRhli => {
                    self.fetched_data =
                        self.bus_read(self.cpu_read_reg(&inst.reg_2)) as u16;
                    emu_cycle(1);
                    self.cpu_set_reg(&RegType::RtHl, self.cpu_read_reg(&RegType::RtHl).wrapping_add(1));
                }
                AddrMode::AmRhld => {
                    self.fetched_data = self.bus_read(self.cpu_read_reg(&inst.reg_2)) as u16;
                    emu_cycle(1);
                    self.cpu_set_reg(&RegType::RtHl, self.cpu_read_reg(&RegType::RtHl).wrapping_sub(1));
                }
                AddrMode::AmHlir => {
                    self.fetched_data = self.cpu_read_reg(&inst.reg_2);
                    self.mem_dest = self.cpu_read_reg(&inst.reg_1);
                    self.dest_is_mem = true;
                    self.cpu_set_reg(&RegType::RtHl, self.cpu_read_reg(&RegType::RtHl).wrapping_add(1));
                },
                AddrMode::AmHldr => {
                    self.fetched_data = self.cpu_read_reg(&inst.reg_2);
                    self.mem_dest = self.cpu_read_reg(&inst.reg_1);
                    self.dest_is_mem = true;
                    self.cpu_set_reg(&RegType::RtHl, self.cpu_read_reg(&RegType::RtHl).wrapping_sub(1));
                },
                AddrMode::AmRa8 => {
                    self.fetched_data = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                },
                AddrMode::AmA8r => {
                    self.mem_dest = self.bus_read(self.regs.pc) as u16 | 0xFF00;
                    self.dest_is_mem = true;
                    emu_cycle(1);
                    self.regs.pc += 1;
                },
                AddrMode::AmHlspr => {
                    self.fetched_data = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                },
                AddrMode::AmD8 => {
                    self.fetched_data = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                },
                AddrMode::AmA16r => {},  
                AddrMode::AmD16r => {
                    let lo = self.bus_read(self.regs.pc);
                    emu_cycle(1);
                    let hi = self.bus_read(self.regs.pc.wrapping_add(1));
                    emu_cycle(1);
                    self.mem_dest = (lo as u16) | ((hi as u16) << 8);
                    self.dest_is_mem = true;
                    self.regs.pc += 2;
                    self.fetched_data = self.cpu_read_reg(&inst.reg_2);
                },
                AddrMode::AmMrd8 => {
                    self.fetched_data = self.bus_read(self.regs.pc) as u16;
                    emu_cycle(1);
                    self.regs.pc += 1;
                    self.mem_dest = self.cpu_read_reg(&inst.reg_1);
                    self.dest_is_mem = true;
                },
                AddrMode::AmMr => {
                    self.mem_dest = self.cpu_read_reg(&inst.reg_1);
                    self.dest_is_mem = true;
                    self.fetched_data = self.bus_read(self.cpu_read_reg(&inst.reg_1)) as u16;
                    emu_cycle(1);
                },
                AddrMode::AmRa16 => {
                    let lo = self.bus_read(self.regs.pc);
                    emu_cycle(1);
                    let hi = self.bus_read(self.regs.pc.wrapping_add(1));
                    emu_cycle(1);
                    let addr = (lo as u16) | ((hi as u16) << 8);
                    self.regs.pc += 2;
                    self.fetched_data = self.bus_read(addr) as u16;
                    emu_cycle(1);
                }
            }
        }
    }
}