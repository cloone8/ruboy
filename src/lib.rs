use cpu::Cpu;

mod cpu;
mod memcontroller;

pub struct Gameboy {
    cpu: Cpu,
}
