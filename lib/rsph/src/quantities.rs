use std::ops::{Div, Mul};

/// Time value with an explicit scalar representation.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Time<T>(pub T);

impl<T> From<T> for Time<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Mass value with an explicit scalar representation.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Mass<T>(pub T);

macro_rules! quantity_ops {
    ($value:ty, $scalar:ty) => {
        impl<T: Into<$scalar>> Mul<Time<T>> for $value {
            type Output = Self;

            fn mul(self, rhs: Time<T>) -> Self {
                let dt: $scalar = rhs.0.into();
                self * dt
            }
        }

        impl<T: Into<$scalar>> Div<Time<T>> for $value {
            type Output = Self;

            fn div(self, rhs: Time<T>) -> Self {
                let dt: $scalar = rhs.0.into();
                self / dt
            }
        }

        impl<T: Into<$scalar>> Div<Mass<T>> for $value {
            type Output = Self;

            fn div(self, rhs: Mass<T>) -> Self {
                let mass: $scalar = rhs.0.into();
                self / mass
            }
        }
    };
}

quantity_ops!(f32, f32);
quantity_ops!(f64, f64);
quantity_ops!(glam::Vec3, f32);
quantity_ops!(glam::DVec3, f64);

macro_rules! mass_weight {
    ($scalar:ty) => {
        impl<T: Into<$scalar>> Mul<$scalar> for Mass<T> {
            type Output = $scalar;

            fn mul(self, weight: $scalar) -> $scalar {
                let mass: $scalar = self.0.into();
                mass * weight
            }
        }
    };
}

mass_weight!(f32);
mass_weight!(f64);
