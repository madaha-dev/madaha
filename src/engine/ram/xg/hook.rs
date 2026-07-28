use std::fmt;

use crate::engine::ram::MemoryAddr;

pub type HookFunc = Box<dyn FnMut() + 'static>;

type LowAddr = [Option<HookFunc>; 129];
type MidAddr = [LowAddr; 129];

pub struct RAMHook([MidAddr; 129]);

impl RAMHook {
    pub fn new() -> Self {
        let n: [[LowAddr; 129]; 129] =
            std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| None)));
        Self(n)
    }

    pub fn insert<F>(&mut self, addr: MemoryAddr, hook: F)
    where
        F: FnMut() + 'static,
    {
        let (mut h, mut m, mut l) = addr.split();
        h = if matches!(h, 0x80..=0xFF) { 0x80 } else { h };
        m = if matches!(m, 0x80..=0xFF) { 0x80 } else { m };
        l = if matches!(l, 0x80..=0xFF) { 0x80 } else { l };

        self.0[h as usize][m as usize][l as usize] = Some(Box::new(hook));
    }

    pub fn call(&mut self, addr: &MemoryAddr) {
        let (h, m, l) = addr.split();
        if matches!(h, 0x80..=0xFF) || matches!(m, 0x80..=0xFF) || matches!(l, 0x80..=0xFF) {
            return;
        }

        if let Some(hook) = self.0[h as usize][m as usize][0x80].as_mut() {
            (hook)();
        } else if let Some(hook) = self.0[h as usize][0x80][l as usize].as_mut() {
            (hook)();
        } else if let Some(hook) = self.0[0x80][m as usize][l as usize].as_mut() {
            (hook)();
        } else if let Some(hook) = self.0[h as usize][m as usize][l as usize].as_mut() {
            (hook)();
        }
    }
}

impl fmt::Debug for RAMHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RAMHook...")
    }
}
