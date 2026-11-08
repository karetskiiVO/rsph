use core::ops::{Add, Mul};

use crate::traits::{Model, SmoothingKernel, SphDomain, SphParticle};

// First & second step
/// rho[i] += m[j] * W(dist(i, j), h) using the model's fixed smoothing length.
pub fn update_density<D, P, M>(model: &M, particle: &mut P, neightbour: &P)
where
    D: SphDomain,
    P: SphParticle<D> + ?Sized,
    M: Model<D> + ?Sized,
    D::Mass: Mul<<M::Kernel as SmoothingKernel<D>>::Value, Output = D::Density>,
    D::Density: Add<Output = D::Density>,
{
    let weight = model.kernel().value(
        particle.position() - neightbour.position(),
        model.h()
    );
    let density = particle.density() + neightbour.rest_mass() * weight;

    particle.set_density(density);
}
