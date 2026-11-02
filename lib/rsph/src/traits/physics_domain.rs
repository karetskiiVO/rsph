use std::ops::{Add, Div, Mul, Sub};

/// Physical quantities and the arithmetic available to a simulation model.
///
/// A domain chooses its numeric representation, spatial dimension and units.
/// Positions locate points relative to an origin and may be represented by
/// vectors such as `glam::Vec3`. Displacements, velocities, accelerations and
/// forces may use the same vector type. The required operations preserve these
/// relationships:
///
/// | Operation | Result |
/// | --- | --- |
/// | `Position + Displacement` | `Position` |
/// | `Position - Position` | `Displacement` |
/// | `Displacement / Time` | `Velocity` |
/// | `Velocity * Time` | `Displacement` |
/// | `Velocity / Time` | `Acceleration` |
/// | `Acceleration * Time` | `Velocity` |
/// | `Force / Mass` | `Acceleration` |
///
/// Each vector quantity also supports addition and subtraction of its own kind.
/// Separate wrapper types allow the compiler to distinguish physical units.
/// Associated types may coincide when the implementation uses a shared scalar
/// or vector representation; in that case the implementer maintains the
/// distinction between quantities.
///
/// # Semantic requirements
///
/// All quantities must use compatible units and coordinate systems. Arithmetic
/// must respect the physical relationships above and the usual vector-space
/// laws, within the precision of the chosen numeric representation.
///
/// The bounds check operand and result types, not numerical validity or these
/// laws. Implementers and callers must ensure finite values and positive mass,
/// density and time steps wherever an algorithm requires them, and must avoid
/// division by zero. The trait performs no runtime validation.
///
/// All quantities implement `Copy` (and therefore `Clone`), so arithmetic can
/// reuse values without explicit cloning. Consumers add `Send` or `Sync` bounds
/// only when their execution strategy needs them.
///
/// # Example
///
/// A semi-implicit Euler step uses the domain's arithmetic and `Copy` guarantees
/// to reuse the time step without additional bounds.
/// Here a 3D domain uses `glam::Vec3` for positions and all vector quantities,
/// and `f32` for time, mass and density.
///
/// ```
/// use glam::Vec3;
/// use rsph_lib::traits::PhysicsDomain;
///
/// struct Glam3D;
///
/// impl PhysicsDomain for Glam3D {
///     type Time = f32;
///     type Mass = f32;
///     type Density = f32;
///     type Force = Vec3;
///     type Position = Vec3;
///     type Displacement = Vec3;
///     type Velocity = Vec3;
///     type Acceleration = Vec3;
/// }
///
/// fn advance<D: PhysicsDomain>(
///     position: D::Position,
///     velocity: D::Velocity,
///     force: D::Force,
///     mass: D::Mass,
///     dt: D::Time,
/// ) -> D::Position {
///     let acceleration = force / mass;
///     let velocity = velocity + acceleration * dt;
///     position + velocity * dt
/// }
///
/// let position = advance::<Glam3D>(
///     Vec3::ZERO, Vec3::X, Vec3::Y * 2.0, 2.0, 0.5,
/// );
/// assert_eq!(position, Vec3::new(0.5, 0.25, 0.0));
/// ```
///
/// A domain is rejected if dividing force by mass produces a type other than
/// its acceleration. Here `DVec2 / f64` produces `DVec2`, but the domain declares
/// `Acceleration = f64`:
///
/// ```compile_fail,E0271
/// use rsph_lib::traits::PhysicsDomain;
///
/// struct InvalidDomain;
///
/// impl PhysicsDomain for InvalidDomain {
///     type Time = f64;
///     type Mass = f64;
///     type Density = f64;
///     type Force = glam::DVec2;
///     type Position = f64;
///     type Displacement = f64;
///     type Velocity = f64;
///     type Acceleration = f64;
/// }
/// ```
///
/// Every quantity must be `Copy`, including density, which has no required
/// arithmetic operations. A heap-owned density therefore cannot be used:
///
/// ```compile_fail,E0277
/// use rsph_lib::traits::PhysicsDomain;
///
/// struct NonCopyDomain;
///
/// impl PhysicsDomain for NonCopyDomain {
///     type Time = f32;
///     type Mass = f32;
///     type Density = Box<f32>;
///     type Force = f32;
///     type Position = f32;
///     type Displacement = f32;
///     type Velocity = f32;
///     type Acceleration = f32;
/// }
/// ```
pub trait PhysicsDomain {
    /// A duration used for time steps and rates of change.
    type Time: Copy;

    /// The mass of a particle or body, used to convert force to acceleration.
    type Mass: Copy;

    /// Mass density in the domain's chosen spatial measure.
    type Density: Copy;

    /// A force vector; division by mass obeys Newton's second law.
    type Force: Copy
        + Add<Output = Self::Force>
        + Sub<Output = Self::Force>
        + Div<Self::Mass, Output = Self::Acceleration>;

    /// A position vector relative to the chosen origin, for example `glam::Vec3`.
    /// Subtracting positions yields a displacement, which may use the same type.
    type Position: Copy
        + Add<Self::Displacement, Output = Self::Position>
        + Sub<Self::Position, Output = Self::Displacement>;

    /// A displacement vector, with units of length.
    type Displacement: Copy
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
