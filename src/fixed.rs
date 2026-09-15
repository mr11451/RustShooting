use core::ops::{Add, AddAssign, Sub};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Q12_4(pub i16);

impl Q12_4 {
    pub const ZERO: Self = Self(0);
    #[allow(dead_code)]
    pub const ONE: Self = Self(16);

    #[allow(dead_code)]
    pub const fn from_int(value: i16) -> Self {
        Self(value.saturating_mul(16))
    }

    pub const fn raw(self) -> i16 {
        self.0
    }
}

impl Add for Q12_4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign for Q12_4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Q12_4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}
