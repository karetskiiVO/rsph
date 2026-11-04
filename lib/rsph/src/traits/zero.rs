/// Additive identity for a numeric value.
pub trait Zero {
    const ZERO: Self;
}

impl Zero for f32 {
    const ZERO: Self = 0.0;
}

impl Zero for f64 {
    const ZERO: Self = 0.0;
}

impl Zero for glam::Vec3 {
    const ZERO: Self = glam::Vec3::ZERO;
}

impl Zero for glam::DVec3 {
    const ZERO: Self = glam::DVec3::ZERO;
}
