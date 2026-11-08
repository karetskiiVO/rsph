use crate::traits::{PhysicsDomain, Sqrt, Zero};

/// Maxima reduced over all particles; kinematic viscosity is nu = mu/rho.
#[derive(Clone, Copy, Debug)]
pub struct TimeStepState<T> {
    pub max_speed: T,
    pub max_sound_speed: T,
    pub max_acceleration: T,
    pub max_kinematic_viscosity: T,
}

/// Positive safety factors for acoustic, acceleration and viscous limits, plus a time cap.
#[derive(Clone, Copy, Debug)]
pub struct TimeStepConfig<T, Time = T> {
    pub cfl: T,
    pub acceleration_factor: T,
    pub viscosity_factor: T,
    pub max_dt: Time,
}

/// Returns min(max_dt, C_cfl*h/(c+v), C_a*sqrt(h/a), C_nu*h^2/nu).
/// Inputs must be finite: h and config positive, state nonnegative; zero rates add no limit.
pub fn calc_dt<D: PhysicsDomain>(
    h: D::Length,
    state: TimeStepState<D::Length>,
    config: TimeStepConfig<D::Length, D::Time>,
) -> D::Time {
    let mut dt = config.max_dt;
    let signal_speed = state.max_sound_speed + state.max_speed;

    if signal_speed > D::Length::ZERO {
        let acoustic_dt = D::Time::from(config.cfl * (h / signal_speed));
        if acoustic_dt < dt { dt = acoustic_dt; }
    }

    if state.max_acceleration > D::Length::ZERO {
        let acceleration_dt = D::Time::from(
            config.acceleration_factor * (h / state.max_acceleration).sqrt()
        );

        if acceleration_dt < dt { dt = acceleration_dt; }
    }

    if state.max_kinematic_viscosity > D::Length::ZERO {
        let viscous_dt = D::Time::from(
            config.viscosity_factor * h * (h / state.max_kinematic_viscosity)
        );

        if viscous_dt < dt { dt = viscous_dt; }
    }

    return dt;
}
