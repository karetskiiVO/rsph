//! Backend-independent scalar and three-dimensional vector operations.

use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Neg, Sub};

/// Scalar operations required by shared physics code.
pub trait Real:
    Copy
    + Debug
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const HALF: Self;

    fn is_finite(self) -> bool;
}

macro_rules! impl_real {
    ($($scalar:ty),+ $(,)?) => {
        $(impl Real for $scalar {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const HALF: Self = 0.5;

            fn is_finite(self) -> bool {
                <$scalar>::is_finite(self)
            }
        })+
    };
}

impl_real!(f32, f64);

/// A three-lane vector independent of its storage layout.
pub trait Vector3: Copy {
    type Scalar: Real;

    fn zero() -> Self;
    fn dot(self, rhs: Self) -> Self::Scalar;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn scale(self, factor: Self::Scalar) -> Self;
}

/// Compile-time choice of scalar, vector layout, and square-root backend.
///
/// The vector's scalar must match the selected precision:
///
/// ```compile_fail
/// use rsph_lib::{Math, cpu::{CpuMath, Vec3}};
/// fn require_math<M: Math>() {}
/// require_math::<CpuMath<f64, Vec3<f32>>>();
/// ```
pub trait Math {
    type Real: Real;
    type Vec3: Vector3<Scalar = Self::Real>;

    fn sqrt(x: Self::Real) -> Self::Real;
}

#[cfg(test)]
mod tests {
    use super::Real;

    macro_rules! real_tests {
        ($name:ident, $scalar:ty) => {
            #[test]
            fn $name() {
                type S = $scalar;
                let half = <S as Real>::HALF;
                assert_eq!(half + half, <S as Real>::ONE);
                assert_eq!(half - half, <S as Real>::ZERO);
                assert_eq!(-half * (4.0 as S) / (2.0 as S), -1.0 as S);
                assert!(Real::is_finite(half));
                assert!(!Real::is_finite(S::INFINITY));
                assert!(!Real::is_finite(S::NEG_INFINITY));
                assert!(!Real::is_finite(S::NAN));
            }
        };
    }

    real_tests!(f32_real_arithmetic_and_finiteness, f32);
    real_tests!(f64_real_arithmetic_and_finiteness, f64);
}
