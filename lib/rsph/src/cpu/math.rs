//! CPU math types with selectable scalar precision and vector layout.

use crate::math::{Math, Real, Vector3};
use core::marker::PhantomData;
use num_traits::Float;

/// A zero-sized CPU backend selected through scalar and vector type parameters.
#[derive(Clone, Copy, Debug)]
pub struct CpuMath<S, V>(PhantomData<(S, V)>);

impl<S, V> Math for CpuMath<S, V>
where
    S: Real + Float,
    V: Vector3<Scalar = S>,
{
    type Real = S;
    type Vec3 = V;

    fn sqrt(x: S) -> S {
        Float::sqrt(x)
    }
}

/// Three contiguous scalar components with C layout and scalar alignment.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3<S: Real> {
    pub x: S,
    pub y: S,
    pub z: S,
}

/// Three scalar components with at least 16-byte alignment.
///
/// The trailing padding is not a fourth component. Alignment alone does not
/// guarantee SIMD instructions; this implementation uses scalar operations.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignedVec3<S: Real> {
    pub x: S,
    pub y: S,
    pub z: S,
}

macro_rules! impl_vector3 {
    ($vector:ident) => {
        impl<S: Real> $vector<S> {
            pub const fn new(x: S, y: S, z: S) -> Self {
                Self { x, y, z }
            }
        }

        impl<S: Real> Vector3 for $vector<S> {
            type Scalar = S;

            fn zero() -> Self {
                Self::new(S::ZERO, S::ZERO, S::ZERO)
            }

            fn dot(self, rhs: Self) -> S {
                self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
            }

            fn add(self, rhs: Self) -> Self {
                Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
            }

            fn sub(self, rhs: Self) -> Self {
                Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
            }

            fn scale(self, factor: S) -> Self {
                Self::new(self.x * factor, self.y * factor, self.z * factor)
            }
        }
    };
}

impl_vector3!(Vec3);
impl_vector3!(AlignedVec3);

#[cfg(test)]
mod tests {
    use super::{AlignedVec3, CpuMath, Vec3};
    use crate::math::{Math, Vector3};
    use core::mem::{align_of, size_of};

    // Mixed signs and three nonzero lanes catch omitted or padded-lane products.
    macro_rules! vector_tests {
        ($name:ident, $scalar:ty, $vector:ident) => {
            #[test]
            fn $name() {
                type S = $scalar;
                type V = $vector<S>;
                type M = CpuMath<S, V>;
                let a = V::new(-2.0, 3.0, -6.0);
                let b = V::new(4.0, -5.0, -2.0);
                assert_eq!(a.dot(b), -11.0);
                assert_eq!(M::sqrt(a.dot(a)), 7.0);
                assert_eq!(a.add(b), V::new(2.0, -2.0, -8.0));
                assert_eq!(a.sub(b), V::new(-6.0, 8.0, -4.0));
                assert_eq!(a.scale(-0.5), V::new(1.0, -1.5, 3.0));
                assert_eq!(a.add(V::zero()), a);
                assert_eq!(M::sqrt(2.25), 1.5);
            }
        };
    }

    vector_tests!(packed_f32_vector_arithmetic, f32, Vec3);
    vector_tests!(packed_f64_vector_arithmetic, f64, Vec3);
    vector_tests!(aligned_f32_vector_arithmetic, f32, AlignedVec3);
    vector_tests!(aligned_f64_vector_arithmetic, f64, AlignedVec3);

    #[test]
    fn packed_vectors_have_three_scalar_layout() {
        assert_eq!(size_of::<Vec3<f32>>(), 12);
        assert_eq!(align_of::<Vec3<f32>>(), 4);
        assert_eq!(size_of::<Vec3<f64>>(), 24);
        assert_eq!(align_of::<Vec3<f64>>(), 8);
    }

    #[test]
    fn aligned_vectors_have_sixteen_byte_alignment_and_padded_stride() {
        assert_eq!(size_of::<AlignedVec3<f32>>(), 16);
        assert_eq!(align_of::<AlignedVec3<f32>>(), 16);
        assert_eq!(size_of::<AlignedVec3<f64>>(), 32);
        assert_eq!(align_of::<AlignedVec3<f64>>(), 16);
    }
}
