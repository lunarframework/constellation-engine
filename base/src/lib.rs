pub mod project;

pub mod math {
    use num_traits::Zero;
    use std::ops::*;

    pub fn align<T>(value: T, alignment: T) -> T
    where
        T: Add<Output = T> + Rem<Output = T> + Sub<Output = T> + PartialEq + Zero + Copy,
    {
        if value % alignment == T::zero() {
            value
        } else {
            value + alignment - (value % alignment)
        }
    }
}
