use core::sync::atomic::Ordering;
use std::sync::Arc;

use ruboy_lib::{GbInputs, InputHandler};

use super::Inputs;

#[derive(Debug, Clone)]
pub struct KeyboardInput {
    inputs: Arc<Inputs>,
}

impl KeyboardInput {
    pub fn new(inputs: Arc<Inputs>) -> Self {
        Self { inputs }
    }
}

impl InputHandler for KeyboardInput {
    fn get_new_inputs(&mut self) -> ruboy_lib::GbInputs {
        let inputs = self.inputs.as_ref();

        GbInputs {
            up: inputs.up.load(Ordering::Relaxed),
            down: inputs.down.load(Ordering::Relaxed),
            left: inputs.left.load(Ordering::Relaxed),
            right: inputs.right.load(Ordering::Relaxed),
            start: inputs.start.load(Ordering::Relaxed),
            select: inputs.select.load(Ordering::Relaxed),
            b: inputs.b.load(Ordering::Relaxed),
            a: inputs.a.load(Ordering::Relaxed),
        }
    }
}
