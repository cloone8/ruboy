use std::fmt::Display;
use std::time::Instant;

use cpu::Cpu;
use cpu::CpuErr;
use input::apply_input_to;
use memcontroller::MemController;

use memcontroller::MemControllerInitErr;
use memcontroller::WriteError;
use ppu::PpuErr;
use ppu::{Ppu, FRAME_CYCLES};
use thiserror::Error;

mod boot;
mod cpu;
mod extern_traits;
mod input;
pub mod isa;
mod memcontroller;
mod ppu;
pub mod rom;

pub use extern_traits::*;

pub const CLOCK_SPEED_HZ: usize = 1 << 22;
pub const CLOCK_SPEED_HZ_F64: f64 = CLOCK_SPEED_HZ as f64;
pub const DESIRED_FRAMERATE: f64 = CLOCK_SPEED_HZ_F64 / (FRAME_CYCLES as f64);

pub struct Ruboy<A, R, V, I>
where
    A: GBAllocator,
    R: RomReader,
    V: GBGraphicsDrawer,
    I: InputHandler,
{
    cycle_accumulator: f64,
    cpu: Cpu,
    ppu: Ppu<V>,
    mem: MemController<A, R>,
    input: I,
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

    #[error("Error during DMA cycle")]
    Dma(#[source] WriteError),
}

impl<A: GBAllocator, R: RomReader, V: GBGraphicsDrawer, I: InputHandler> Ruboy<A, R, V, I> {
    pub fn new(rom: R, output: V, input: I) -> Result<Self, RuboyStartErr<R>> {
        Ok(Self {
            cycle_accumulator: 0.0,
            cpu: Cpu::new(),
            ppu: Ppu::new(output),
            mem: MemController::new(rom)?,
            input,
        })
    }

    pub fn step(&mut self, dt: f64) -> Result<usize, RuboyErr<V>> {
        log::debug!("Stepping emulator {} seconds", dt);

        let cycles_dt = dt * CLOCK_SPEED_HZ_F64;
        let (mut cycles_to_run, accumulated) = split_f64(cycles_dt);

        self.cycle_accumulator += accumulated;
        let (extra_cycles, new_accumulator) = split_f64(self.cycle_accumulator);

        cycles_to_run += extra_cycles;
        self.cycle_accumulator = new_accumulator;

        debug_assert!(cycles_to_run >= 0);

        log::trace!("Running {} cycles", cycles_to_run as usize);

        for _ in 0..(cycles_to_run as usize) {
            let (new_joypad_reg_value, can_raise_joypad_interrupt) =
                apply_input_to(self.mem.io_registers.joypad, self.input.get_new_inputs());

            self.mem.io_registers.joypad = new_joypad_reg_value;
            if can_raise_joypad_interrupt {
                self.mem.io_registers.interrupts_requested.set_joypad(true);
            }

            self.cpu.run_cycle(&mut self.mem)?;
            self.ppu.run_cycle(&mut self.mem)?;
            self.mem.dma_cycle().map_err(|e| RuboyErr::Dma(e))?;
        }

        Ok(cycles_to_run as usize)
    }
}

fn split_f64(f: f64) -> (i64, f64) {
    let truncated = f.trunc();

    (truncated as i64, f - truncated)
}
