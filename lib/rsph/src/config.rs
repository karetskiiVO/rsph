use std::marker::PhantomData;

use rsph_core::traits::{Model, Particle, PhysicsDomain};

use crate::traits::Device;

pub struct Config<PhD: PhysicsDomain, D: Device, M: Model<PhD, P>, P: Particle<PhD>> {
    pub device: D,
    pub model: M,

    _types: PhantomData<(PhD, fn() -> P)>,
}
