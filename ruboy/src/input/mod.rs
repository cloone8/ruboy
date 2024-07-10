use core::sync::atomic::{AtomicBool, Ordering};

pub mod keyboard;

#[derive(Debug, Default)]
pub struct Inputs {
    pub up: AtomicBool,
    pub down: AtomicBool,
    pub left: AtomicBool,
    pub right: AtomicBool,
    pub start: AtomicBool,
    pub select: AtomicBool,
    pub a: AtomicBool,
    pub b: AtomicBool,
}

impl Inputs {
    pub fn set_to_none(&self) {
        self.up.store(false, Ordering::Relaxed);
        self.down.store(false, Ordering::Relaxed);
        self.left.store(false, Ordering::Relaxed);
        self.right.store(false, Ordering::Relaxed);
        self.start.store(false, Ordering::Relaxed);
        self.select.store(false, Ordering::Relaxed);
        self.a.store(false, Ordering::Relaxed);
        self.b.store(false, Ordering::Relaxed);
    }
}
