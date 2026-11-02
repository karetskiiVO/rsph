use crate::traits::{PhysicsDomain, RelativisticDomain, SphDomain};

/// Particle getters and setters; quantities are returned by Copy.
/// Implementations and models keep physical state consistent.
pub trait Particle<D: PhysicsDomain> {
    /// Position relative to the domain's chosen origin, e.g. `glam::Vec3`.
    fn position(&self) -> D::Position;

    /// Sets the position in the domain's coordinates.
    fn set_position(&mut self, position: D::Position);

    /// Coordinate velocity dx^i/dt; distinct from four-velocity and local observer velocity.
    fn velocity(&self) -> D::Velocity;

    /// Sets coordinate velocity dx^i/dt.
    fn set_velocity(&mut self, velocity: D::Velocity);

    /// Accumulated coordinate acceleration d^2 x^i / dt^2.
    fn acceleration(&self) -> D::Acceleration;

    fn set_acceleration(&mut self, acceleration: D::Acceleration);
}

/// Local rest-frame fluid properties shared by classical and relativistic SPH.
pub trait SphParticle<D: SphDomain>: Particle<D> {
    /// Baryonic rest mass represented by this particle, excluding thermal and
    /// kinetic energy. Positive for a material SPH particle.
    fn rest_mass(&self) -> D::Mass;

    /// Baryonic rest mass per proper volume in the local fluid rest frame.
    fn rest_mass_density(&self) -> D::Density;

    /// Pressure in the local fluid rest frame, consistent with the model's EOS.
    fn pressure(&self) -> D::Pressure;

    fn density(&self) -> D::Density;
    fn set_density(&mut self, density: D::Density);

    /// Sets the local rest-frame pressure.
    fn set_pressure(&mut self, pressure: D::Pressure);

    /// Internal energy per unit baryonic rest mass, excluding rest energy.
    fn specific_internal_energy(&self) -> D::SpecificInternalEnergy;

    /// Sets internal energy per unit rest mass, excluding rest energy.
    fn set_specific_internal_energy(&mut self, energy: D::SpecificInternalEnergy);
}

/// Per-baryon canonical variables and frame-specific densities for SR/GR SPH.
/// The model supplies geometry, EOS and consistent state updates.
pub trait RelativisticSphParticle<D: RelativisticDomain>: SphParticle<D> {
    /// Baryon number carried by the particle, positive for material particles.
    fn baryon_number(&self) -> D::BaryonNumber;

    /// Sets the baryon number carried by the particle.
    fn set_baryon_number(&mut self, number: D::BaryonNumber);

    /// Baryons per proper volume measured in the local fluid rest frame.
    fn rest_frame_baryon_density(&self) -> D::BaryonDensity;

    /// Sets the proper baryon density in the local rest frame.
    fn set_rest_frame_baryon_density(&mut self, density: D::BaryonDensity);

    /// Baryons per coordinate volume; N = sqrt(-g) U^0 n in units with c = 1.
    fn computing_frame_baryon_density(&self) -> D::BaryonDensity;

    /// Sets the baryon density per coordinate volume.
    fn set_computing_frame_baryon_density(&mut self, density: D::BaryonDensity);

    /// Spatial covariant canonical momentum per baryon, including enthalpy.
    fn canonical_momentum_per_baryon(&self) -> D::CanonicalMomentumPerBaryon;

    /// Sets the spatial covariant canonical momentum per baryon.
    fn set_canonical_momentum_per_baryon(&mut self, momentum: D::CanonicalMomentumPerBaryon);

    /// Canonical energy per baryon, including rest energy in the model's units.
    fn canonical_energy_per_baryon(&self) -> D::CanonicalEnergyPerBaryon;

    /// Sets canonical energy per baryon, including rest energy.
    fn set_canonical_energy_per_baryon(&mut self, energy: D::CanonicalEnergyPerBaryon);
}
