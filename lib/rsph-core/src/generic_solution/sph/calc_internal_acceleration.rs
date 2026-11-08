use core::ops::{Add, Div, Mul};

use crate::traits::{Model, SmoothingKernel, SphDomain, SphParticle};

type PressureOverDensity<D> = <<D as SphDomain>::Pressure as Div<<D as crate::traits::PhysicsDomain>::Density>>::Output;
type PressureOverDensitySquared<D> = <PressureOverDensity<D> as Div<<D as crate::traits::PhysicsDomain>::Density>>::Output;

// TODO: трение и внешние силы

/// Adds the Newtonian pressure contribution from one neighbour to particle acceleration.
/// Uses p_i/rho_i^2 + p_j/rho_j^2; densities must be positive.
pub fn calc_internal_acceleration<D, P, M>(model: &M, particle: &mut P, neighbour: &P)
where
    D: SphDomain,
    P: SphParticle<D> + ?Sized,
    M: Model<D> + ?Sized,
    D::Pressure: Div<D::Density>,
    PressureOverDensity<D>: Div<D::Density>,
    PressureOverDensitySquared<D>: Add<Output = PressureOverDensitySquared<D>>,
    D::Mass: Mul<PressureOverDensitySquared<D>>,
    <M::Kernel as SmoothingKernel<D>>::Gradient: Mul<<D::Mass as Mul<PressureOverDensitySquared<D>>>::Output, Output = D::Acceleration>,
{
    let pressure_factor =
        particle.pressure() / particle.density() / particle.density() +
        neighbour.pressure() / neighbour.density() / neighbour.density();
    let gradient = model
        .kernel()
        .gradient(particle.position() - neighbour.position(), model.h());

    let acceleration = particle.acceleration() - gradient * (neighbour.rest_mass() * pressure_factor);
    particle.set_acceleration(acceleration);
}
