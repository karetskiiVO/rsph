use std::marker::PhantomData;

use rsph_lib::traits::{Model, Particle, PhysicsDomain};

use crate::{device::CPU, scene::Scene};

pub struct Runtime<PhD: PhysicsDomain, M: Model<PhD, P>, P: Particle<PhD>> {
    device: CPU,
    model: M,

    scenes: (), // Container of scenes

    _types: PhantomData<(PhD, fn() -> P)>,
}

impl<PhD: PhysicsDomain, M: Model<PhD, P>, P: Particle<PhD>> Runtime<PhD, M, P> {
    fn create_scene(&mut self) -> &Scene {
        todo!()
    }
}
