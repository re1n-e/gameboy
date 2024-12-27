use crate::cpu::CpuContext;

impl<'a> CpuContext<'a> {
    pub fn stack_push(&mut self, data: u8) {
        self.regs.sp = self.regs.pc.wrapping_sub(1);
        self.bus_write(self.regs.sp, data);
    }

    pub fn stack_push16(&mut self, data: u16) {
        self.stack_push((data >> 8) as u8 & 0xFF);
        self.stack_push(data as u8 & 0xFF);
    }

    pub fn stack_pop(&mut self) -> u8 {
        let res = self.bus_read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        res
    }

    pub fn stack_pop16(&mut self) -> u16 {
        let lo: u16 = self.stack_pop() as u16;
        let hi: u16 = self.stack_pop() as u16;

        (hi << 8) | lo
    }
}
