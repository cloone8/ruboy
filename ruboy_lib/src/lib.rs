use std::error::Error;
use std::fmt::Display;
use std::time::Duration;
use std::time::Instant;

use cpu::Cpu;
use memcontroller::allocator::GBAllocator;
use memcontroller::MemController;

pub use memcontroller::allocator;
use memcontroller::MemControllerInitErr;
use rom::RomReader;
use thiserror::Error;

mod boot;
mod cpu;
pub mod isa;
mod memcontroller;
pub mod rom;

pub struct Gameboy<A, R>
where
    A: GBAllocator,
    R: RomReader,
{
    cpu: Cpu,
    mem: MemController<A, R>,
}

#[derive(Debug, Clone, Copy)]
enum Frequency {
    None(f64),
    Kilo(f64),
    Mega(f64),
    Giga(f64),
}

impl Frequency {
    pub fn new(val: f64) -> Self {
        Self::None(val).upcast()
    }

    pub fn val(self) -> f64 {
        match self {
            Frequency::None(x) => x,
            Frequency::Kilo(x) => x,
            Frequency::Mega(x) => x,
            Frequency::Giga(x) => x,
        }
    }
    pub fn upcast(self) -> Self {
        if self.val() > 1000.0 {
            match self {
                Frequency::None(x) => Frequency::Kilo(x / 1000.0).upcast(),
                Frequency::Kilo(x) => Frequency::Mega(x / 1000.0).upcast(),
                Frequency::Mega(x) => Frequency::Giga(x / 1000.0).upcast(),
                Frequency::Giga(_) => self,
            }
        } else {
            self
        }
    }
}

impl Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Frequency::None(x) => write!(f, "{:.2}Hz", x),
            Frequency::Kilo(x) => write!(f, "{:.2}KHz", x),
            Frequency::Mega(x) => write!(f, "{:.2}MHz", x),
            Frequency::Giga(x) => write!(f, "{:.2}GHz", x),
        }
    }
}

#[derive(Debug, Error)]
pub enum GameboyStartErr<R: RomReader> {
    #[error("Could not initialize memory controller: {0}")]
    MemController(#[from] MemControllerInitErr<R>),
}

impl<A: GBAllocator, R: RomReader> Gameboy<A, R> {
    pub fn new(rom: R) -> Result<Self, GameboyStartErr<R>> {
        Ok(Self {
            cpu: Cpu::new(),
            mem: MemController::new(rom)?,
        })
    }

    pub fn start(mut self) -> Result<(), Box<dyn Error>> {
        log::info!("Starting Ruboy Emulator");

        let mut cycles = 0_usize;
        let mut last_second = Instant::now();

        loop {
            self.cpu.run_cycle(&mut self.mem)?;
            cycles += 1;

            let elapsed_time = last_second.elapsed();
            if elapsed_time > Duration::from_secs(1) {
                let cycles_per_second = Frequency::new(cycles as f64 / elapsed_time.as_secs_f64());

                log::info!("Current speed: {}", cycles_per_second);
                cycles = 0;
                last_second = Instant::now();
            }
        }
    }
}
