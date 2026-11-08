use crate::traits::{
    EquationOfState, Particle, PhysicsDomain, PrimitiveRecovery, RelativisticDomain,
    RelativisticSphParticle, SmoothingKernel, SpacetimeGeometry, SphDomain, SphParticle,
};

/// Configures a smoothing kernel for particles of type `P` in domain `D`.
pub trait Model<D: PhysicsDomain, P: Particle<D> + ?Sized> {
    /// Kernel using the same domain and distance convention as the model.
    type Kernel: SmoothingKernel<D>;

    /// Configured kernel, borrowed from or otherwise owned by the model.
    fn kernel(&self) -> &Self::Kernel;

    /// Fixed smoothing length shared by all particles.
    fn h(&self) -> D::Length;
}

/// SPH particle capability and thermal EOS, with the kernel inherited from Model.
pub trait SphModel<D: SphDomain, P: SphParticle<D> + ?Sized>: Model<D, P> {
    /// Thermal EOS using the same rest-mass and energy units as the particles.
    type EquationOfState: EquationOfState<D>;

    /// Configured equation of state.
    fn equation_of_state(&self) -> &Self::EquationOfState;
}

/// Relativistic SPH model with compatible geometry, EOS and primitive recovery.
/// Integration and device execution are separate contracts.
pub trait RelativisticModel<D: RelativisticDomain, P: RelativisticSphParticle<D> + ?Sized>:
    SphModel<D, P>
{
    /// Geometry and coordinate convention used by this model.
    type Geometry: SpacetimeGeometry<D>;
    /// Recovery compatible with this model's exact component types.
    type PrimitiveRecovery: PrimitiveRecovery<D, Self::EquationOfState, Self::Geometry>;

    /// Configured spacetime geometry.
    fn geometry(&self) -> &Self::Geometry;
    /// Configured primitive recovery algorithm.
    fn primitive_recovery(&self) -> &Self::PrimitiveRecovery;
}
