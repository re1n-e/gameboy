pub fn cpu_init() {}
pub fn cpu_step() -> bool {
    println!("Cpu not yet implemented.");
    false
}

struct CpuRegister {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l :u8,
    pc: u16,
    sp: u16,
}

struct CpuContext {
    regs: CpuRegister,
    fetch_data: u16,
    mem_dest: u16,
    cur_opcode: u8,
    halted: bool,
    stepping: bool,
}

