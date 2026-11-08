use crate::traits::SphDomain;

/// Thermal EOS using proper rest-mass density and internal energy per unit rest mass.
/// Specific internal energy excludes rest energy.
pub trait EquationOfState<D: SphDomain> {
    /// Evaluates rest-frame pressure; the caller supplies a state supported by the EOS.
    fn pressure(
        &self,
        rest_mass_density: D::Density,
        specific_internal_energy: D::SpecificInternalEnergy,
    ) -> D::Pressure;
}
