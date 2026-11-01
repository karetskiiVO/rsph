use crate::types::math::Math;

pub struct Math32 {}

impl Math for Math32 {
    type Number = f32;
    type Vector = glam::Vec3;
}

pub struct FastMath32 {}

impl Math for FastMath32 {
    type Number = f64;
    type Vector = glam::Vec3A;
}

pub struct Math64 {}

impl Math for Math64 {
    type Number = f64;
    type Vector = glam::DVec3;
}
