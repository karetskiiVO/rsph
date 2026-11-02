use crate::traits::{EquationOfState, RelativisticDomain, SpacetimeGeometry};

/// Conservative fluid values at one event, independent of particle storage.
/// Canonical momentum and energy use per-baryon normalization.
pub struct ConservativeVariables<D: RelativisticDomain> {
    /// Baryons per computing-frame coordinate volume, including the metric measure.
    pub computing_frame_baryon_density: D::BaryonDensity,
    /// Spatial covariant canonical momentum per baryon, including enthalpy.
    pub canonical_momentum_per_baryon: D::CanonicalMomentumPerBaryon,
    /// Canonical energy per baryon, including rest energy.
    pub canonical_energy_per_baryon: D::CanonicalEnergyPerBaryon,
}

impl<D: RelativisticDomain> Copy for ConservativeVariables<D> {}

impl<D: RelativisticDomain> Clone for ConservativeVariables<D> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Recovered primitive values for the same event and conservative state.
/// The caller validates and commits them to particle storage.
pub struct PrimitiveVariables<D: RelativisticDomain> {
    /// Coordinate velocity `dx^i/dt` in the model's chart.
    pub velocity: D::Velocity,
    /// Proper baryonic rest-mass density.
    pub rest_mass_density: D::Density,
    /// Proper baryon number density in the local fluid rest frame.
    pub rest_frame_baryon_density: D::BaryonDensity,
    /// Internal energy per unit rest mass, excluding rest energy.
    pub specific_internal_energy: D::SpecificInternalEnergy,
    /// Local rest-frame pressure consistent with the selected EOS.
    pub pressure: D::Pressure,
}

impl<D: RelativisticDomain> Copy for PrimitiveVariables<D> {}

impl<D: RelativisticDomain> Clone for PrimitiveVariables<D> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Recovers primitive values consistent with the input, metric and EOS.
/// Metric and conservative values must refer to the same event.
pub trait PrimitiveRecovery<D: RelativisticDomain, E: EquationOfState<D>, G: SpacetimeGeometry<D>> {
    /// Invalid input, EOS/metric incompatibility, or failure to converge.
    type Error;

    /// Recovers a complete set of primitive values without updating a particle.
    fn recover(
        &self,
        conservative: ConservativeVariables<D>,
        metric: G::Metric,
        equation_of_state: &E,
    ) -> Result<PrimitiveVariables<D>, Self::Error>;
}
