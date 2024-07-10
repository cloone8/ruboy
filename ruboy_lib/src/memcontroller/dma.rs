use crate::{GBAllocator, RomReader};

use super::{MemController, ReadError, WriteErrType};

#[derive(Debug)]
pub struct DMAController {
    oam: Option<DMACommand>,
}

#[derive(Debug, Clone)]
pub struct DMACommand {
    pub cycles: usize,
    pub target_address: u16,
    pub data: Vec<u8>,
}

impl DMAController {
    pub fn new() -> Self {
        Self { oam: None }
    }

    pub fn push_oam(&mut self, command: DMACommand) {
        if self.oam.is_none() {
            self.oam = Some(command);
        }
    }

    pub fn run_cycle(&mut self) -> Vec<DMACommand> {
        let mut ret = Vec::new();

        self.oam = self.oam.take().and_then(|mut oam_dma| {
            oam_dma.cycles -= 1;

            if oam_dma.cycles == 0 {
                ret.push(oam_dma);
                None
            } else {
                Some(oam_dma)
            }
        });

        ret
    }
}

pub fn oam_dma_command(
    val: u8,
    mem: &MemController<impl GBAllocator, impl RomReader>,
) -> Result<DMACommand, ReadError> {
    let source_addr = (val as u16) * 0x100;
    log::info!(
        "Creating new DMA command reading from 0x{:x} (0x{:x})",
        source_addr,
        val
    );

    let mut source_data: Vec<u8> = Vec::with_capacity(0x100);

    for addr in source_addr..(source_addr + 0x100) {
        source_data.push(mem.read8(addr)?);
    }

    Ok(DMACommand {
        cycles: 640,
        target_address: 0xFE00,
        data: source_data,
    })
}
