#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct MemoryAddr([u8; 3]);

impl MemoryAddr {
    pub fn inc(&mut self) {
        *self += 1;
    }

    pub const fn new(h: u8, m: u8, l: u8) -> Self {
        Self([h, m, l])
    }

    pub fn split(&self) -> (u8, u8, u8) {
        (self.0[0], self.0[1], self.0[2])
    }

    fn to_usize(&self) -> usize {
        (self.0[2] as usize) | (self.0[1] as usize) << 8 | (self.0[0] as usize) << 16
    }
}

impl std::ops::Add<u8> for MemoryAddr {
    type Output = Self;
    fn add(self, rhs: u8) -> Self::Output {
        let [h, m, l] = self.0;
        let (l, carry) = l.overflowing_add(rhs);
        let (m, carry) = m.overflowing_add(carry as u8);
        let h = h.wrapping_add(carry as u8);
        Self([h, m, l])
    }
}

impl std::ops::AddAssign<u8> for MemoryAddr {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

impl From<[u8; 3]> for MemoryAddr {
    fn from(value: [u8; 3]) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for MemoryAddr {
    fn from(value: &[u8]) -> Self {
        Self(value.try_into().unwrap_or([0, 0, 0]))
    }
}

impl From<usize> for MemoryAddr {
    fn from(value: usize) -> Self {
        let h = ((value >> 16) & 0xFF) as u8;
        let m = ((value >> 8) & 0xFF) as u8;
        let l = (value & 0xFF) as u8;

        Self([h, m, l])
    }
}

impl std::ops::Index<usize> for MemoryAddr {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Into<[u8; 3]> for MemoryAddr {
    fn into(self) -> [u8; 3] {
        self.0
    }
}

impl Into<Box<[u8]>> for MemoryAddr {
    fn into(self) -> Box<[u8]> {
        self.0.into()
    }
}

impl Into<usize> for MemoryAddr {
    fn into(self) -> usize {
        self.to_usize()
    }
}

impl Into<usize> for &MemoryAddr {
    fn into(self) -> usize {
        self.to_usize()
    }
}
