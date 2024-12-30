use crate::cpu::CpuContext;
use crate::instructions::RegType;

impl<'a> CpuContext<'a> {
    pub fn cpu_read_reg(&self, rt: &RegType) -> u16 {
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

    pub fn cpu_set_reg(&mut self, rt: &RegType, val: u16) {
        match rt {
            RegType::RtA => self.regs.a = (val & 0xFF) as u8,
            RegType::RtF => self.regs.f = (val & 0xFF) as u8,
            RegType::RtB => self.regs.b = (val & 0xFF) as u8,
            RegType::RtC => self.regs.c = (val & 0xFF) as u8,
            RegType::RtD => self.regs.d = (val & 0xFF) as u8,
            RegType::RtE => self.regs.e = (val & 0xFF) as u8,
            RegType::RtH => self.regs.h = (val & 0xFF) as u8,
            RegType::RtL => self.regs.l = (val & 0xFF) as u8,
            RegType::RtAf => self.regs.a = reverse(val) as u8,
            RegType::RtBc => self.regs.b = reverse(val) as u8,
            RegType::RtDe => self.regs.d = reverse(val) as u8,
            RegType::RtHl => self.regs.h = reverse(val) as u8,
            RegType::RtPc => self.regs.pc = val,
            RegType::RtSp => self.regs.sp = val,
            RegType::RtNone => {}
        }
    }

    pub fn cpu_read_reg8(&mut self, rt: &RegType) -> u8 {
        match rt {
            RegType::RtA => self.regs.a,
            RegType::RtF => self.regs.f,
            RegType::RtB => self.regs.b,
            RegType::RtC => self.regs.c,
            RegType::RtD => self.regs.d,
            RegType::RtE => self.regs.e,
            RegType::RtH => self.regs.h,
            RegType::RtL => self.regs.l,
            RegType::RtHl => self.bus_read(self.cpu_read_reg(&RegType::RtHl)),
            _ => panic!("No Impl"),
        }
    }

    pub fn cpu_set_reg8(&mut self, rt: &RegType, val: u8) {
        match rt {
            RegType::RtA => self.regs.a = (val & 0xFF) as u8,
            RegType::RtF => self.regs.f = (val & 0xFF) as u8,
            RegType::RtB => self.regs.b = (val & 0xFF) as u8,
            RegType::RtC => self.regs.c = (val & 0xFF) as u8,
            RegType::RtD => self.regs.d = (val & 0xFF) as u8,
            RegType::RtE => self.regs.e = (val & 0xFF) as u8,
            RegType::RtH => self.regs.h = (val & 0xFF) as u8,
            RegType::RtL => self.regs.l = (val & 0xFF) as u8,
            RegType::RtHl => self.bus_write(self.cpu_read_reg(&RegType::RtHl), val),
            _ => panic!("No impl"),
        }
    }

}

fn reverse(n: u16) -> u16 {
    ((n & 0xFF00) >> 8) | ((n & 0x00FF) << 8)
}
