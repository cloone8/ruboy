use core::cell::RefCell;
use std::rc::Rc;

use ruboy_lib::{GbInputs, InputHandler};

#[derive(Debug, Default)]
pub struct Inputs {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
}

impl Inputs {
    pub fn set_to_none(&mut self) {
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
        self.start = false;
        self.select = false;
        self.a = false;
        self.b = false;
    }
}

#[derive(Debug, Clone)]
pub struct SharedInputs {
    pub inputs: Rc<RefCell<Inputs>>,
}

impl SharedInputs {
    pub fn new() -> Self {
        Self {
            inputs: Rc::new(RefCell::new(Inputs::default())),
        }
    }
}

impl InputHandler for SharedInputs {
    fn get_new_inputs(&mut self) -> ruboy_lib::GbInputs {
        let inputs_borrowed = self.inputs.borrow();

        GbInputs {
            up: inputs_borrowed.up,
            start: inputs_borrowed.start,
            select: inputs_borrowed.select,
            b: inputs_borrowed.b,
            a: inputs_borrowed.a,
            down: inputs_borrowed.down,
            left: inputs_borrowed.left,
            right: inputs_borrowed.right,
        }
    }
}
