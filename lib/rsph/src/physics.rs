use core::ops::AddAssign;

use crate::Math;

/// Pure particle calculations, independent of storage and execution backend.
///
/// A model can have its own classical or relativistic particle representation.
/// Rates include all evolved quantities and must be additive. CPU execution
/// calls `local_rate`, adds each directed `pair_rate`, then calls `advance`.
/// The input particles remain the old state throughout that process.
///
/// Keep implementations free of host-only resources if they will also be
/// compiled for a GPU. Device compatibility still depends on the GPU toolchain.
pub trait Physics<M: Math> {
    type Params;
    type Particle: Copy;
    type Rate: AddAssign;

    /// Position used to construct the neighbor index.
    fn position(particle: &Self::Particle) -> M::Vec3;

    /// Local source terms, or a zero rate when there are none.
    fn local_rate(params: &Self::Params, particle: &Self::Particle) -> Self::Rate;

    /// Contribution of `neighbor` to `particle`; each directed pair is separate.
    ///
    /// This must not update the neighbor or any external state. Particle masses,
    /// smoothing lengths and other model-specific inputs belong to `Particle`
    /// or `Params`.
    fn pair_rate(
        params: &Self::Params,
        particle: &Self::Particle,
        neighbor: &Self::Particle,
    ) -> Self::Rate;

    /// Produce a new particle from the old state and accumulated rate.
    ///
    /// The update rule is part of the supplied model. This execution protocol
    /// is one stage; a multistage integrator requiring additional neighbor
    /// evaluations needs a corresponding execution implementation.
    fn advance(
        params: &Self::Params,
        particle: &Self::Particle,
        rate: Self::Rate,
        dt: M::Real,
    ) -> Self::Particle;
}
