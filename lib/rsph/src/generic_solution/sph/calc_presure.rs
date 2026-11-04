use crate::traits::{EquationOfState, SphDomain, SphModel, SphParticle};

/// Third step: updates pressure from the model's EOS.
/// Don't forget setup equation_of_state
pub fn calc_presure<D, P, M>(model: &M, particle: &mut P)
where
    D: SphDomain,
    P: SphParticle<D> + ?Sized,
    M: SphModel<D> + ?Sized,
{
    let pressure = model.equation_of_state().pressure(
        particle.rest_mass_density(),
        particle.specific_internal_energy(),
    );
    particle.set_pressure(pressure);
}
