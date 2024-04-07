use cpu::Cpu;
use memcontroller::MemController;

mod cpu;
mod memcontroller;
pub mod isa;

pub struct Gameboy<M>
    where M: MemController
{
    cpu: Cpu,
    mem: M
}
