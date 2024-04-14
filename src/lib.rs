use cpu::Cpu;
use memcontroller::{GBRam, MemController};

mod cpu;
pub mod isa;
mod memcontroller;

pub struct Gameboy<R>
where
    R: GBRam,
{
    cpu: Cpu,
    mem: MemController<R>,
}
