use rsph_lib::{
    config::Config,
    traits::{BuildRuntime, Model, Particle, PhysicsDomain},
};

use crate::device::CPU;
use crate::runtime::Runtime;

impl<PhD, M, P> BuildRuntime<CPU> for Config<PhD, CPU, M, P>
where
    PhD: PhysicsDomain,
    M: Model<PhD, P>,
    P: Particle<PhD>,
{
    type Error = (); // TODO:
    type Runtime = Runtime<PhD, M, P>;

    fn build_runtime() -> Result<Self::Runtime, Self::Error> {
        todo!()
    }
}
