use cpu::Cpu;
use memcontroller::MemController;

mod cpu;
pub mod isa;
mod memcontroller;

pub struct Gameboy<M>
where
    M: MemController,
{
    cpu: Cpu,
    mem: M,
}
