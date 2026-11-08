/// Square root with an explicit output quantity.
pub trait Sqrt {
    type Output;
    fn sqrt(self) -> Self::Output;
}

impl Sqrt for f32 {
    type Output = Self;
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }
}

impl Sqrt for f64 {
    type Output = Self;
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }
}
