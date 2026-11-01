//! Sequential CPU execution with reusable particle and position buffers.

mod math;
mod neighbors;

pub use math::{AlignedVec3, CpuMath, Vec3};
pub use neighbors::{AllPairs, CpuNeighbors, FixedRadius, RadiusError, RadiusIndex};

use core::{fmt, marker::PhantomData};

use crate::{Config, Execute, Math, Params, Physics, Real, ScalarOf};

/// Selects the sequential CPU implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cpu;

#[derive(Debug, PartialEq, Eq)]
pub enum CpuError<E> {
    InvalidTimeStep,
    Neighbors(E),
}

impl<E: fmt::Display> fmt::Display for CpuError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeStep => f.write_str("time step must be positive and finite"),
            Self::Neighbors(error) => write!(f, "neighbor search: {error}"),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for CpuError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Neighbors(error) => Some(error),
            Self::InvalidTimeStep => None,
        }
    }
}

type ParticleOf<C> = <<C as Config>::Physics as Physics<<C as Config>::Math>>::Particle;
type NeighborIndexOf<C> = <<C as Config>::Neighbors as CpuNeighbors<<C as Config>::Math>>::Index;

/// CPU-owned buffers and parameters for a configuration.
pub struct CpuState<C: Config>
where
    C::Neighbors: CpuNeighbors<C::Math>,
{
    particles: Vec<ParticleOf<C>>,
    output: Vec<ParticleOf<C>>,
    positions: Vec<<C::Math as Math>::Vec3>,
    index: NeighborIndexOf<C>,
    params: Params<C>,
    _config: PhantomData<fn() -> C>,
}

impl<C: Config> CpuState<C>
where
    C::Neighbors: CpuNeighbors<C::Math>,
{
    pub fn particles(&self) -> &[ParticleOf<C>] {
        &self.particles
    }

    /// The next step rebuilds the position buffer and neighbor index.
    pub fn particles_mut(&mut self) -> &mut [ParticleOf<C>] {
        &mut self.particles
    }

    pub fn params(&self) -> &Params<C> {
        &self.params
    }

    /// Changed neighbor parameters are validated on the next step.
    pub fn params_mut(&mut self) -> &mut Params<C> {
        &mut self.params
    }
}

impl<C> Execute<C> for Cpu
where
    C: Config<Backend = Cpu>,
    C::Neighbors: CpuNeighbors<C::Math>,
{
    type State = CpuState<C>;
    type Input = Vec<ParticleOf<C>>;
    type Error = CpuError<<C::Neighbors as CpuNeighbors<C::Math>>::Error>;

    fn init(
        &mut self,
        particles: Self::Input,
        params: Params<C>,
    ) -> Result<Self::State, Self::Error> {
        let positions: Vec<_> = particles.iter().map(C::Physics::position).collect();
        let index =
            C::Neighbors::build(&positions, &params.neighbors).map_err(CpuError::Neighbors)?;
        let output = Vec::with_capacity(particles.len());
        Ok(CpuState {
            particles,
            output,
            positions,
            index,
            params,
            _config: PhantomData,
        })
    }

    fn step(&mut self, state: &mut Self::State, dt: ScalarOf<C>) -> Result<(), Self::Error> {
        if !dt.is_finite() || dt <= ScalarOf::<C>::ZERO {
            return Err(CpuError::InvalidTimeStep);
        }

        state.positions.clear();
        state
            .positions
            .extend(state.particles.iter().map(C::Physics::position));
        C::Neighbors::rebuild(&mut state.index, &state.positions, &state.params.neighbors)
            .map_err(CpuError::Neighbors)?;

        state.output.clear();
        for (i, particle) in state.particles.iter().enumerate() {
            let mut rate = C::Physics::local_rate(&state.params.physics, particle);
            C::Neighbors::for_each(&state.index, &state.positions, i, |j| {
                rate += C::Physics::pair_rate(&state.params.physics, particle, &state.particles[j]);
            });
            state.output.push(C::Physics::advance(
                &state.params.physics,
                particle,
                rate,
                dt,
            ));
        }
        core::mem::swap(&mut state.particles, &mut state.output);
        Ok(())
    }
}
