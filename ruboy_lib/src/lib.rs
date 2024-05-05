use cpu::Cpu;
use memcontroller::MemController;

pub use memcontroller::GBRam;
pub use memcontroller::BoxedGBRam;
pub use memcontroller::InlineGBRam;

mod boot;
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

impl<R: GBRam> Gameboy<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        log::info!("Starting Ruboy Emulator");

        self.load_boot_rom();

        loop {
            self.cpu.run_instruction(&mut self.mem).unwrap();
        }
    }

    #[cfg(feature = "boot_img_enabled")]
    fn load_boot_rom(&mut self) {
        log::info!("Boot ROM enabled, using {} image", boot::IMAGE_NAME);
        log::trace!("Boot ROM has length {}", boot::IMAGE.len());

        for (i, &byte) in boot::IMAGE.iter().enumerate() {
            self.mem.write8(i as u16, byte);
        }
    }

    #[cfg(not(feature = "boot_img_enabled"))]
    fn load_boot_rom(&mut self) {}
}

impl<R: GBRam> Default for Gameboy<R> {
    fn default() -> Self {
        Gameboy {
            cpu: Cpu::new(),
            mem: MemController::<R>::new()
        }
    }
}
