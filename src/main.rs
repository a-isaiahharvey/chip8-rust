#![cfg(not(target_arch = "wasm32"))]

use std::{
    fs,
    sync::{
        atomic::{self, AtomicU64},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use chip8_rust::{
    app::App,
    cpu::{Chip8, Chip8IO, StepResult},
};
use clap::Parser;

/// Call this in a loop to limit how many times per second the loop runs
pub fn rate_limit(ticks_per_sec: u64, ticker: &mut Instant) -> (Duration, Duration) {
    let last_tick = *ticker;
    let task_end = Instant::now();
    let busy_elapsed = task_end - *ticker;
    let target = Duration::from_nanos(1_000_000_000 / ticks_per_sec);

    if target > busy_elapsed {
        thread::sleep(target - busy_elapsed);
    }

    let loop_end = Instant::now();
    let full_elapsed = loop_end - last_tick;

    *ticker = loop_end;

    (busy_elapsed, full_elapsed)
}

#[derive(Parser, Debug)]
enum Args {
    /// Run the ROM
    Run {
        /// Instructions per second
        #[clap(long, default_value_t = 1000)]
        ips: u64,

        /// Path to the rom file to load
        rom: String,
    },
}

impl Args {
    fn rom_bytes(&self) -> Vec<u8> {
        let rom = match self {
            Args::Run { rom, .. } => rom,
        };

        println!("Reading file {}", rom);
        fs::read(&rom).expect("open input file")
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let instruction_set = args.rom_bytes();
    match args {
        Args::Run { ips, .. } => {
            let io = Arc::new(Mutex::new(Chip8IO::new()));
            let cpu = Arc::new(Mutex::new(Chip8::new(&instruction_set, io.clone(), false)));
            let target_ips = Arc::new(AtomicU64::new(ips));
            let gui = App::new(cpu.clone(), io, target_ips.clone());

            // Creates thread for running the Chip8 emulator
            thread::spawn(move || {
                let mut ticker = Instant::now();
                loop {
                    let step = cpu.lock().unwrap().step();
                    match step {
                        Ok(StepResult::Continue(_)) => {}
                        _ => break,
                    };

                    rate_limit(target_ips.load(atomic::Ordering::Relaxed), &mut ticker);
                }
                println!("CPU Stopped");
            });

            gui.run();
        }
    };
}
