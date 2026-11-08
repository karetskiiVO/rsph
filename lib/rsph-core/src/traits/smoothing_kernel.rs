use crate::traits::PhysicsDomain;

/// Spatial smoothing kernel normalized in the model's chosen volume measure.
/// Inputs must be finite, with a positive smoothing length.
pub trait SmoothingKernel<D: PhysicsDomain> {
    /// Kernel weight, with units inverse to the chosen spatial volume measure.
    type Value: Copy;

    /// Spatial gradient of the kernel weight, with one additional inverse length.
    type Gradient: Copy;

    /// Radius outside which both the weight and its gradient vanish.
    fn support_radius(&self, smoothing_length: D::Length) -> D::Length;

    /// Kernel weight at the given displacement and smoothing length.
    fn value(&self, displacement: D::Displacement, smoothing_length: D::Length) -> Self::Value;

    /// Gradient with respect to displacement, holding smoothing length fixed.
    fn gradient(
        &self,
        displacement: D::Displacement,
        smoothing_length: D::Length,
    ) -> Self::Gradient;
}

/// Separate viscosity kernel support, using a dissipative Laplacian weight.
pub trait ViscosityKernel<D: PhysicsDomain>: SmoothingKernel<D> {
    type Laplacian: Copy;

    /// Finite, nonnegative weight; zero outside support_radius(h).
    fn viscosity_laplacian(
        &self,
        displacement: D::Displacement,
        smoothing_length: D::Length,
    ) -> Self::Laplacian;
}
