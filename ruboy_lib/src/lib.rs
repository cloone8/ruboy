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

const TARGET_CLOCK_SPEED_HZ: u64 = 1 << 22;
const SPEED_CHECK_INTERVAL_MS: u64 = 10;
const SPEED_CHECK_INTERVAL_DURATION: Duration = Duration::from_millis(SPEED_CHECK_INTERVAL_MS);
const CYCLES_PER_INTERVAL: u64 = (TARGET_CLOCK_SPEED_HZ * SPEED_CHECK_INTERVAL_MS) / 1000;

const SPEED_REPORT_INTERVAL_MS: u64 = 1000;
const SPEED_REPORT_INTERVAL_DURATION: Duration = Duration::from_millis(SPEED_REPORT_INTERVAL_MS);

pub struct Ruboy<A, R, V>
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

    pub fn val_raw(self) -> f64 {
        match self {
            Frequency::None(x) => x,
            Frequency::Kilo(x) => x * 1000.0,
            Frequency::Mega(x) => x * (1000.0 * 1000.0),
            Frequency::Giga(x) => x * (1000.0 * 1000.0 * 1000.0),
        }
    }

    pub fn val_unit(self) -> f64 {
        match self {
            Frequency::None(x) => x,
            Frequency::Kilo(x) => x,
            Frequency::Mega(x) => x,
            Frequency::Giga(x) => x,
        }
    }
    pub fn upcast(self) -> Self {
        if self.val_unit() > 1000.0 {
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
            Frequency::None(x) => write!(f, "{:.6}Hz", x),
            Frequency::Kilo(x) => write!(f, "{:.6}KHz", x),
            Frequency::Mega(x) => write!(f, "{:.6}MHz", x),
            Frequency::Giga(x) => write!(f, "{:.6}GHz", x),
        }
    }
}

#[derive(Debug, Error)]
pub enum RuboyStartErr<R: RomReader> {
    #[error("Could not initialize memory controller: {0}")]
    MemController(#[from] MemControllerInitErr<R>),
}

#[derive(Debug, Error)]
// #[derive(Debug)]
pub enum RuboyErr<V: GBGraphicsDrawer> {
    #[error("Error during CPU cycle")]
    Cpu(#[from] CpuErr),

    #[error("Error during PPU cycle")]
    Ppu(#[from] PpuErr<V>),
}

// impl<V: GBGraphicsDrawer> std::error::Error for RuboyErr<V> {}
// impl<V: GBGraphicsDrawer> Display for RuboyErr<V> {}

impl<A: GBAllocator, R: RomReader, V: GBGraphicsDrawer> Ruboy<A, R, V> {
    pub fn new(rom: R, output: V) -> Result<Self, RuboyStartErr<R>> {
        Ok(Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(output),
            mem: MemController::new(rom)?,
        })
    }

    pub fn start(mut self) -> Result<(), RuboyErr<V>> {
        log::info!("Starting Ruboy Emulator");

        let mut cycles_since_last_check = 0_usize;
        let mut cycles_since_last_report = 0_usize;

        let mut last_speed_check = Instant::now();
        let mut last_report = Instant::now();

        loop {
            self.cpu.run_cycle(&mut self.mem)?;
            self.ppu.run_cycle(&mut self.mem)?;

            cycles_since_last_check += 1;
            cycles_since_last_report += 1;

            // Report clock speed to the user
            let last_report_elapsed = last_report.elapsed();
            if last_report_elapsed >= SPEED_REPORT_INTERVAL_DURATION {
                let cycles_per_second = Frequency::new(
                    cycles_since_last_report as f64 / last_report_elapsed.as_secs_f64(),
                );

                log::info!("Current speed: {}", cycles_per_second);
                cycles_since_last_report = 0;
                last_report = Instant::now();
            }

            // Make sure we're keeping roughly in sync with the original gameboy
            // clockspeed
            let last_check_elapsed = last_speed_check.elapsed();

            if last_check_elapsed >= SPEED_CHECK_INTERVAL_DURATION {
                cycles_since_last_check = 0;
                last_speed_check = Instant::now();
            } else if cycles_since_last_check >= CYCLES_PER_INTERVAL as usize {
                let wake_time = last_speed_check + SPEED_CHECK_INTERVAL_DURATION;

                spin_sleep::sleep(wake_time.duration_since(Instant::now()));
            }
        }
    }
}
