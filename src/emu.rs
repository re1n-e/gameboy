use crate::{cart, common, cpu::{self, CpuContext}};

use sdl2::{
    self,
    sys::{ttf::TTF_Init, SDL_Delay, SDL_Init, SDL_INIT_VIDEO},
};

pub struct emu_context {
    paused: bool,
    running: bool,
    ticks: u64,
}

impl emu_context {
    pub fn new() -> Self {
        emu_context {
            paused: false,
            running: false,
            ticks: 0,
        }
    }

    fn delay(ms: u32) {
        unsafe {
            SDL_Delay(ms);
        }
    }

    pub fn emu_run(&mut self, argv: Vec<String>) {
        if argv.len() < 2 {
            panic!("Usage: emu <rom_file>\n");
        }

        let mut cart: cart::CartContext = cart::CartContext::new();
        if let Ok(val) = cart.cart_load(&argv[1]) {
            if !val {
                panic!("Failed to load ROM file: {}", &argv[1]);
            }
        }

        println!("Cart loaded..");

        unsafe {
            SDL_Init(SDL_INIT_VIDEO);
            println!("SDL INIT");
            TTF_Init();
            println!("TTF INIT");
        }

        let mut cpu: CpuContext = CpuContext::new(&mut cart);

        self.running = true;
        self.paused = false;
        self.ticks = 0;

        while self.running {
            if self.paused {
                common::delay(10);
                continue;
            }

            if !cpu.cpu_step() {
                panic!("CPU Stopped");
            }

            self.ticks += 1;
        }
    }
}
