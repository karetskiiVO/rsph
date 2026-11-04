use std::ops::{Div, Mul};

use crate::traits::{Model, PhysicsDomain, SphDomain, SphParticle, ViscosityKernel};

type ViscosityFactor<D, V> = <<V as Div<<D as PhysicsDomain>::Density>>::Output as Div<
    <D as PhysicsDomain>::Density,
>>::Output;
type WeightedViscosity<D, V> = <<D as PhysicsDomain>::Mass as Mul<ViscosityFactor<D, V>>>::Output;

/// Adds mu * m_j/(rho_i*rho_j) * (v_j-v_i) * viscosity_laplacian to acceleration.
/// Dynamic viscosity mu must be nonnegative, with positive densities and smoothing length.
pub fn calc_viscosity_acceleration<D, P, M, V>(
    model: &M,
    particle: &mut P,
    neighbour: &P,
    dynamic_viscosity: V,
) where
    D: SphDomain,
    P: SphParticle<D> + ?Sized,
    M: Model<D> + ?Sized,
    M::Kernel: ViscosityKernel<D>,
    V: Div<D::Density>,
    <V as Div<D::Density>>::Output: Div<D::Density>,
    D::Mass: Mul<ViscosityFactor<D, V>>,
    <M::Kernel as ViscosityKernel<D>>::Laplacian: Mul<WeightedViscosity<D, V>>,
    D::Velocity: Mul<
        <<M::Kernel as ViscosityKernel<D>>::Laplacian as Mul<WeightedViscosity<D, V>>>::Output,
        Output = D::Acceleration,
    >,
{
    let laplacian = model.kernel().viscosity_laplacian(
        particle.position() - neighbour.position(),
        model.h()
    );
    let viscosity_factor = dynamic_viscosity / particle.density() / neighbour.density();
    let rate = laplacian * (neighbour.rest_mass() * viscosity_factor);
    let acceleration = particle.acceleration() + (neighbour.velocity() - particle.velocity()) * rate;

    particle.set_acceleration(acceleration);
}
