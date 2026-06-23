
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MemoryAddr([u8; 3]);

impl MemoryAddr {
    pub fn inc(&mut self) {
        *self += 1;
    }

    pub const fn new(h: u8, m: u8, l: u8) -> Self {
        Self([h, m, l])
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
