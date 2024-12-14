use gameboy::emu;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut emu_context: emu::emu_context = emu::emu_context::new();
    emu_context.emu_run(args);
}
