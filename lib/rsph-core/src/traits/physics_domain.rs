use core::ops::{Add, Div, Mul, Sub};

use crate::traits::{Sqrt, Zero};

/// Physical quantities and arithmetic available to a simulation model.
/// Implementations maintain consistent units and valid numerical states.
pub trait PhysicsDomain {
    /// A duration used for time steps and rates of change.
    type Time: Copy + PartialOrd + From<Self::Length>;

    /// A scalar length for spatial scales, distances and smoothing kernels.
    type Length: Copy
        + PartialOrd
        + Zero
        + Add<Output = Self::Length>
        + Div<Output = Self::Length>
        + Mul<Output = Self::Length>
        + Sqrt<Output = Self::Length>;

    /// The mass representation; SPH particles use it for baryonic rest mass.
    type Mass: Copy;

    /// Mass density in the domain's chosen spatial measure.
    type Density: Copy;

    /// A force vector; the model or a domain extension specifies its dynamics.
    type Force: Copy + Add<Output = Self::Force> + Sub<Output = Self::Force>;

    /// A position vector relative to the chosen origin, for example `glam::Vec3`.
    /// Subtracting positions yields a displacement, which may use the same type.
    type Position: Copy
        + Add<Self::Displacement, Output = Self::Position>
        + Sub<Self::Position, Output = Self::Displacement>;

    /// A displacement vector, with units of length.
    type Displacement: Copy
        + Zero
        + Add<Output = Self::Displacement>
        + Sub<Output = Self::Displacement>
        + Div<Self::Time, Output = Self::Velocity>;

    /// A velocity vector, with units of length per time.
    type Velocity: Copy
        + Add<Output = Self::Velocity>
        + Sub<Output = Self::Velocity>
        + Mul<Self::Time, Output = Self::Displacement>
        + Div<Self::Time, Output = Self::Acceleration>;

    /// An acceleration vector, with units of length per time squared.
    type Acceleration: Copy
        + Add<Output = Self::Acceleration>
        + Sub<Output = Self::Acceleration>
        + Mul<Self::Time, Output = Self::Velocity>;
}

/// Newtonian dynamics: dividing force by mass produces acceleration.
pub trait NewtonianDomain:
    PhysicsDomain<Force: Div<Self::Mass, Output = Self::Acceleration>>
{
}

/// Additional quantity types for smoothed particle hydrodynamics.
pub trait SphDomain: PhysicsDomain {
    /// Pressure measured in the local fluid rest frame.
    type Pressure: Copy;

    /// Internal energy per unit baryonic rest mass, excluding rest energy.
    type SpecificInternalEnergy: Copy;
}

/// Additional quantity types shared by special- and general-relativistic SPH.
pub trait RelativisticDomain: SphDomain {
    /// The baryon number carried by a particle.
    type BaryonNumber: Copy;

    /// Baryon number density; the getter specifies its frame and volume measure.
    type BaryonDensity: Copy;

    /// Covariant spatial canonical momentum per baryon, including enthalpy.
    type CanonicalMomentumPerBaryon: Copy;

    /// Canonical energy per baryon, including rest energy.
    type CanonicalEnergyPerBaryon: Copy;
}
