use std::fmt::Display;
use std::time::Duration;
use std::time::Instant;

use cpu::Cpu;
use cpu::CpuErr;
use memcontroller::MemController;

use memcontroller::MemControllerInitErr;
use ppu::Ppu;
use ppu::PpuErr;
use thiserror::Error;

mod boot;
mod cpu;
mod extern_traits;
pub mod isa;
mod memcontroller;
mod ppu;
pub mod rom;

pub use extern_traits::*;

pub struct Gameboy<A, R, V>
where
    A: GBAllocator,
    R: RomReader,
    V: GBGraphicsDrawer,
{
    cpu: Cpu,
    ppu: Ppu<V>,
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

#[derive(Debug, Error)]
pub enum GameboyErr {
    #[error("Error during CPU cycle: {0}")]
    Cpu(#[from] CpuErr),

    #[error("Error during PPU cycle: {0}")]
    Ppu(#[from] PpuErr),
}

impl<A: GBAllocator, R: RomReader, V: GBGraphicsDrawer> Gameboy<A, R, V> {
    pub fn new(rom: R, output: V) -> Result<Self, GameboyStartErr<R>> {
        Ok(Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(output),
            mem: MemController::new(rom)?,
        })
    }

    pub fn start(mut self) -> Result<(), GameboyErr> {
        log::info!("Starting Ruboy Emulator");

        let mut cycles = 0_usize;
        let mut last_second = Instant::now();

        loop {
            self.cpu.run_cycle(&mut self.mem)?;
            self.ppu.run_cycle(&mut self.mem)?;

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
