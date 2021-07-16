use std::ops;

pub struct ClampedValue<T: PartialOrd + Copy> {
    value: T,
    min: T,
    max: T,
}

impl<T: PartialOrd + Copy> ClampedValue<T> {
    pub fn new(value: T, min: T, max: T) -> Self {
        Self { value, min, max }
    }

    pub fn value(&self) -> T {
        self.value
    }

    pub fn assign(&mut self, value: T) {
        self.value = if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        };
    }
}

impl<T: PartialOrd + Copy + ops::Add<Output = T>> ops::AddAssign<T> for ClampedValue<T> {
    fn add_assign(&mut self, rhs: T) {
        self.assign(self.value + rhs);
    }
}

impl<T: PartialOrd + Copy + ops::Sub<Output = T>> ops::SubAssign<T> for ClampedValue<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.assign(self.value - rhs);
    }
}
