use crate::traits::{Particle, PhysicsDomain};

pub trait Model<PhD: PhysicsDomain> {
    type Particle: Particle<PhD>;
    // TODO: ядро интегрирования
}
