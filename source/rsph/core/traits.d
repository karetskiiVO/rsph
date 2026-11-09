module rsph.core.traits;

import rsph.core.quantities : quantityFrom, squareRoot, zero;
import rsph.core.result : Result;
import std.meta : allSatisfy;
import std.traits : isCopyable, lvalueOf, rvalueOf;

private enum bool isCopyableValue(T) = !is(T == void) && isCopyable!T;
private alias areCopyable(Types...) = allSatisfy!(isCopyableValue, Types);

/// Physical quantities and arithmetic available to a simulation model.
/// Implementations maintain consistent units and valid numerical states.
enum bool isPhysicsDomain(D) = __traits(compiles, {
    alias Time = D.Time;
    alias Length = D.Length;
    alias Force = D.Force;
    alias Position = D.Position;
    alias Displacement = D.Displacement;
    alias Velocity = D.Velocity;
    alias Acceleration = D.Acceleration;

    static assert(areCopyable!(Time, Length, D.Mass, D.Density, Force,
        Position, Displacement, Velocity, Acceleration));

    static assert(is(typeof(lvalueOf!(const Time) < lvalueOf!(const Time)) == bool));
    static assert(is(typeof(lvalueOf!(const Time) == lvalueOf!(const Time)) == bool));
    static assert(is(typeof(quantityFrom!Time(rvalueOf!Length)) == Time));

    static assert(is(typeof(lvalueOf!(const Length) < lvalueOf!(const Length)) == bool));
    static assert(is(typeof(lvalueOf!(const Length) == lvalueOf!(const Length)) == bool));
    static assert(is(typeof(zero!Length) == Length));
    static assert(is(typeof(rvalueOf!Length + rvalueOf!Length) == Length));
    static assert(is(typeof(rvalueOf!Length / rvalueOf!Length) == Length));
    static assert(is(typeof(rvalueOf!Length * rvalueOf!Length) == Length));
    static assert(is(typeof(squareRoot(rvalueOf!Length)) == Length));

    static assert(is(typeof(rvalueOf!Force + rvalueOf!Force) == Force));
    static assert(is(typeof(rvalueOf!Force - rvalueOf!Force) == Force));

    static assert(is(typeof(rvalueOf!Position + rvalueOf!Displacement) == Position));
    static assert(is(typeof(rvalueOf!Position - rvalueOf!Position) == Displacement));

    static assert(is(typeof(zero!Displacement) == Displacement));
    static assert(is(typeof(rvalueOf!Displacement + rvalueOf!Displacement) == Displacement));
    static assert(is(typeof(rvalueOf!Displacement - rvalueOf!Displacement) == Displacement));
    static assert(is(typeof(rvalueOf!Displacement / rvalueOf!Time) == Velocity));

    static assert(is(typeof(rvalueOf!Velocity + rvalueOf!Velocity) == Velocity));
    static assert(is(typeof(rvalueOf!Velocity - rvalueOf!Velocity) == Velocity));
    static assert(is(typeof(rvalueOf!Velocity * rvalueOf!Time) == Displacement));
    static assert(is(typeof(rvalueOf!Velocity / rvalueOf!Time) == Acceleration));

    static assert(is(typeof(rvalueOf!Acceleration + rvalueOf!Acceleration) == Acceleration));
    static assert(is(typeof(rvalueOf!Acceleration - rvalueOf!Acceleration) == Acceleration));
    static assert(is(typeof(rvalueOf!Acceleration * rvalueOf!Time) == Velocity));
});

/// Newtonian dynamics, where dividing force by mass produces acceleration.
enum bool isNewtonianDomain(D) = isPhysicsDomain!D
    && is(typeof(rvalueOf!(D.Force) / rvalueOf!(D.Mass)) == D.Acceleration);

/// Domain with pressure and internal energy per unit rest mass.
enum bool isSphDomain(D) = isPhysicsDomain!D && __traits(compiles, {
    static assert(areCopyable!(D.Pressure, D.SpecificInternalEnergy));
});

/// Domain with the per-baryon quantities used by relativistic SPH.
enum bool isRelativisticDomain(D) = isSphDomain!D && __traits(compiles, {
    static assert(areCopyable!(D.BaryonNumber, D.BaryonDensity,
        D.CanonicalMomentumPerBaryon, D.CanonicalEnergyPerBaryon));
});

/// Read and write position, coordinate velocity and coordinate acceleration.
/// Velocity is dx^i/dt in the domain's coordinates.
enum bool isParticle(D, P) = isPhysicsDomain!D
    && is(typeof(lvalueOf!(const P).position()) == D.Position)
    && is(typeof(lvalueOf!P.set_position(rvalueOf!(D.Position))) == void)
    && is(typeof(lvalueOf!(const P).velocity()) == D.Velocity)
    && is(typeof(lvalueOf!P.set_velocity(rvalueOf!(D.Velocity))) == void)
    && is(typeof(lvalueOf!(const P).acceleration()) == D.Acceleration)
    && is(typeof(lvalueOf!P.set_acceleration(rvalueOf!(D.Acceleration))) == void);

/// Local rest-frame fluid properties shared by classical and relativistic SPH.
/// Rest mass is baryonic mass. Specific internal energy excludes rest energy.
enum bool isSphParticle(D, P) = isSphDomain!D && isParticle!(D, P)
    && is(typeof(lvalueOf!(const P).rest_mass()) == D.Mass)
    && is(typeof(lvalueOf!(const P).rest_mass_density()) == D.Density)
    && is(typeof(lvalueOf!(const P).pressure()) == D.Pressure)
    && is(typeof(lvalueOf!(const P).density()) == D.Density)
    && is(typeof(lvalueOf!P.set_density(rvalueOf!(D.Density))) == void)
    && is(typeof(lvalueOf!P.set_pressure(rvalueOf!(D.Pressure))) == void)
    && is(typeof(lvalueOf!(const P).specific_internal_energy()) == D.SpecificInternalEnergy)
    && is(typeof(lvalueOf!P.set_specific_internal_energy(rvalueOf!(D.SpecificInternalEnergy))) == void);

/// Per-baryon canonical variables and frame-specific densities for relativistic SPH.
/// Computing-frame density includes the metric volume factor and the time component of four-velocity.
enum bool isRelativisticSphParticle(D, P) = isRelativisticDomain!D && isSphParticle!(D, P)
    && is(typeof(lvalueOf!(const P).baryon_number()) == D.BaryonNumber)
    && is(typeof(lvalueOf!P.set_baryon_number(rvalueOf!(D.BaryonNumber))) == void)
    && is(typeof(lvalueOf!(const P).rest_frame_baryon_density()) == D.BaryonDensity)
    && is(typeof(lvalueOf!P.set_rest_frame_baryon_density(rvalueOf!(D.BaryonDensity))) == void)
    && is(typeof(lvalueOf!(const P).computing_frame_baryon_density()) == D.BaryonDensity)
    && is(typeof(lvalueOf!P.set_computing_frame_baryon_density(rvalueOf!(D.BaryonDensity))) == void)
    && is(typeof(lvalueOf!(const P).canonical_momentum_per_baryon()) == D.CanonicalMomentumPerBaryon)
    && is(typeof(lvalueOf!P.set_canonical_momentum_per_baryon(rvalueOf!(D.CanonicalMomentumPerBaryon))) == void)
    && is(typeof(lvalueOf!(const P).canonical_energy_per_baryon()) == D.CanonicalEnergyPerBaryon)
    && is(typeof(lvalueOf!P.set_canonical_energy_per_baryon(rvalueOf!(D.CanonicalEnergyPerBaryon))) == void);

/// Spatial smoothing kernel normalized in the model's chosen volume measure.
/// Finite inputs and a positive smoothing length are required. The weight and
/// gradient vanish outside the support radius.
enum bool isSmoothingKernel(D, K) = isPhysicsDomain!D && __traits(compiles, {
    static assert(areCopyable!(K.Value, K.Gradient));
    static assert(is(typeof(lvalueOf!(const K).support_radius(rvalueOf!(D.Length))) == D.Length));
    static assert(is(typeof(lvalueOf!(const K).value(
        rvalueOf!(D.Displacement), rvalueOf!(D.Length))) == K.Value));
    static assert(is(typeof(lvalueOf!(const K).gradient(
        rvalueOf!(D.Displacement), rvalueOf!(D.Length))) == K.Gradient));
});

/// Smoothing kernel with a separate dissipative Laplacian weight.
/// The weight is finite, nonnegative and zero outside the support radius.
enum bool isViscosityKernel(D, K) = isSmoothingKernel!(D, K) && __traits(compiles, {
    static assert(isCopyableValue!(K.Laplacian));
    static assert(is(typeof(lvalueOf!(const K).viscosity_laplacian(
        rvalueOf!(D.Displacement), rvalueOf!(D.Length))) == K.Laplacian));
});

/// Rest-frame thermal pressure from proper density and specific internal energy.
enum bool isEquationOfState(D, E) = isSphDomain!D
    && is(typeof(lvalueOf!(const E).pressure(
        rvalueOf!(D.Density), rvalueOf!(D.SpecificInternalEnergy))) == D.Pressure);

private enum bool isComponentType(Expected, Actual) =
    is(Actual == Expected) || is(Actual == const(Expected));

/// Configured smoothing kernel and fixed smoothing length for a particle type.
enum bool isModel(D, P, M) = isParticle!(D, P) && __traits(compiles, {
    static assert(isSmoothingKernel!(D, M.Kernel));
    static assert(isComponentType!(M.Kernel, typeof(lvalueOf!(const M).kernel())));
    static assert(is(typeof(lvalueOf!(const M).h()) == D.Length));
});

/// SPH model with a thermal equation of state compatible with its domain.
enum bool isSphModel(D, P, M) = isSphParticle!(D, P) && isModel!(D, P, M)
    && __traits(compiles, {
        static assert(isEquationOfState!(D, M.EquationOfState));
        static assert(isComponentType!(M.EquationOfState, typeof(lvalueOf!(const M).equation_of_state())));
    });

/// Metric sampling in the model's coordinate chart, with typed sampling errors.
/// The metric representation, signature and units must match the recovery component.
enum bool isSpacetimeGeometry(D, G) = isRelativisticDomain!D && __traits(compiles, {
    static assert(isCopyableValue!(G.Metric));
    static assert(is(typeof(lvalueOf!(const G).metric_at(
        rvalueOf!(D.Time), rvalueOf!(D.Position))) == Result!(G.Metric, G.Error)));
});

/// Conservative fluid values at one event, with per-baryon normalization.
struct ConservativeVariables(D)
if (isRelativisticDomain!D)
{
    /// Baryons per computing-frame coordinate volume, including the metric measure.
    D.BaryonDensity computing_frame_baryon_density;
    /// Spatial covariant canonical momentum per baryon, including enthalpy.
    D.CanonicalMomentumPerBaryon canonical_momentum_per_baryon;
    /// Canonical energy per baryon, including rest energy.
    D.CanonicalEnergyPerBaryon canonical_energy_per_baryon;
}

/// Recovered primitive values for the same event and conservative state.
struct PrimitiveVariables(D)
if (isRelativisticDomain!D)
{
    /// Coordinate velocity dx^i/dt in the model's chart.
    D.Velocity velocity;
    /// Proper baryonic rest-mass density.
    D.Density rest_mass_density;
    /// Proper baryon number density in the local fluid rest frame.
    D.BaryonDensity rest_frame_baryon_density;
    /// Internal energy per unit rest mass, excluding rest energy.
    D.SpecificInternalEnergy specific_internal_energy;
    /// Local rest-frame pressure consistent with the equation of state.
    D.Pressure pressure;
}

/// Recovery using a compatible metric and equation of state at the same event.
enum bool isPrimitiveRecovery(D, E, G, R) = isEquationOfState!(D, E) && isSpacetimeGeometry!(D, G)
    && __traits(compiles, {
        static assert(is(typeof(lvalueOf!(const R).recover(
            rvalueOf!(ConservativeVariables!D), rvalueOf!(G.Metric), lvalueOf!(const E)))
            == Result!(PrimitiveVariables!D, R.Error)));
    });

/// Relativistic SPH model with geometry and recovery matched to its exact component types.
enum bool isRelativisticModel(D, P, M) = isRelativisticSphParticle!(D, P) && isSphModel!(D, P, M)
    && __traits(compiles, {
        static assert(isSpacetimeGeometry!(D, M.Geometry));
        static assert(isPrimitiveRecovery!(D, M.EquationOfState, M.Geometry, M.PrimitiveRecovery));
        static assert(isComponentType!(M.Geometry, typeof(lvalueOf!(const M).geometry())));
        static assert(isComponentType!(M.PrimitiveRecovery, typeof(lvalueOf!(const M).primitive_recovery())));
    });
