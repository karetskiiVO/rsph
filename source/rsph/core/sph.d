module rsph.core.sph;

import rsph.core.quantities : quantityFrom, squareRoot, zero;
import rsph.core.traits : isModel, isPhysicsDomain, isSphModel, isSphParticle, isViscosityKernel;

/// Maxima reduced over all particles. Kinematic viscosity is dynamic viscosity divided by density.
struct TimeStepState(T)
{
    T max_speed;
    T max_sound_speed;
    T max_acceleration;
    T max_kinematic_viscosity;
}

/// Positive safety factors for the acoustic, acceleration and viscosity limits, plus a time cap.
struct TimeStepConfig(T, Time = T)
{
    T cfl;
    T acceleration_factor;
    T viscosity_factor;
    Time max_dt;
}

/// Returns min(max_dt, C_cfl*h/(c+v), C_a*sqrt(h/a), C_nu*h^2/nu).
/// Inputs must be finite. Smoothing length and configuration values must be positive.
/// State values must be nonnegative. Zero rates add no limit.
D.Time calc_dt(D)(D.Length h, TimeStepState!(D.Length) state, TimeStepConfig!(D.Length, D.Time) config)
if (isPhysicsDomain!D)
{
    auto dt = config.max_dt;
    auto signal_speed = state.max_sound_speed + state.max_speed;
    const no_rate = zero!(D.Length);

    if (signal_speed > no_rate)
    {
        auto acoustic_dt = quantityFrom!(D.Time)(config.cfl * (h / signal_speed));
        if (acoustic_dt < dt)
            dt = acoustic_dt;
    }

    if (state.max_acceleration > no_rate)
    {
        auto acceleration_dt = quantityFrom!(D.Time)(
            config.acceleration_factor * squareRoot(h / state.max_acceleration));
        if (acceleration_dt < dt)
            dt = acceleration_dt;
    }

    if (state.max_kinematic_viscosity > no_rate)
    {
        auto viscous_dt = quantityFrom!(D.Time)(
            config.viscosity_factor * h * (h / state.max_kinematic_viscosity));
        if (viscous_dt < dt)
            dt = viscous_dt;
    }

    return dt;
}

/// Adds m_j * W(r_i - r_j, h) to density using the model's fixed smoothing length.
void update_density(D, P, M)(ref const M model, ref P particle, ref const P neighbour)
if (isSphParticle!(D, P) && isModel!(D, P, M))
{
    auto weight = model.kernel().value(particle.position() - neighbour.position(), model.h());
    auto density = particle.density();
    auto contribution = neighbour.rest_mass() * weight;
    static assert(is(typeof(contribution) == D.Density),
        "Mass times kernel weight must yield the domain's density type.");

    auto updated_density = density + contribution;
    static assert(is(typeof(updated_density) == D.Density),
        "Adding densities must yield the domain's density type.");
    particle.set_density(updated_density);
}

/// Updates pressure using the model's equation of state and the particle's rest-frame properties.
void calc_presure(D, P, M)(ref const M model, ref P particle)
if (isSphModel!(D, P, M))
{
    auto pressure = model.equation_of_state().pressure(
        particle.rest_mass_density(), particle.specific_internal_energy());
    particle.set_pressure(pressure);
}

/// Adds the Newtonian pressure contribution from one neighbour to particle acceleration.
/// Uses the sum of each particle's pressure divided by its squared density. Densities must be positive.
void calc_internal_acceleration(D, P, M)(ref const M model, ref P particle, ref const P neighbour)
if (isSphParticle!(D, P) && isModel!(D, P, M))
{
    auto particle_factor = particle.pressure() / particle.density() / particle.density();
    auto neighbour_factor = neighbour.pressure() / neighbour.density() / neighbour.density();
    auto pressure_factor = particle_factor + neighbour_factor;
    static assert(is(typeof(pressure_factor) == typeof(particle_factor)),
        "Adding pressure factors must yield the same type as each factor.");

    auto gradient = model.kernel().gradient(particle.position() - neighbour.position(), model.h());
    auto acceleration = particle.acceleration();
    auto contribution = gradient * (neighbour.rest_mass() * pressure_factor);
    static assert(is(typeof(contribution) == D.Acceleration),
        "The pressure contribution must yield the domain's acceleration type.");
    particle.set_acceleration(acceleration - contribution);
}

/// Adds mu * m_j/(rho_i*rho_j) * (v_j-v_i) * viscosity_laplacian to acceleration.
/// Dynamic viscosity must be nonnegative. Densities and smoothing length must be positive.
void calc_viscosity_acceleration(D, P, M, V)(ref const M model, ref P particle, ref const P neighbour,
    V dynamic_viscosity)
if (isSphParticle!(D, P) && isModel!(D, P, M) && isViscosityKernel!(D, M.Kernel))
{
    auto laplacian = model.kernel().viscosity_laplacian(
        particle.position() - neighbour.position(), model.h());
    auto viscosity_factor = dynamic_viscosity / particle.density() / neighbour.density();
    auto rate = laplacian * (neighbour.rest_mass() * viscosity_factor);
    auto acceleration = particle.acceleration();
    auto contribution = (neighbour.velocity() - particle.velocity()) * rate;
    static assert(is(typeof(contribution) == D.Acceleration),
        "The viscosity contribution must yield the domain's acceleration type.");
    particle.set_acceleration(acceleration + contribution);
}
