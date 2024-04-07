use cpu::Cpu;
use memcontroller::MemController;

mod cpu;
mod memcontroller;

pub struct Gameboy {
    cpu: Cpu,
    mem: MemController
}
