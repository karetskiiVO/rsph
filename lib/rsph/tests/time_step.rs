use rsph_lib::generic_solution::sph::{TimeStepConfig, TimeStepState, calc_dt};
use rsph_lib::quantities::Time;
use rsph_lib::traits::PhysicsDomain;

macro_rules! domain {
    ($name:ident, $scalar:ty, $time:ty, $vector:ty) => {
        struct $name;
        impl PhysicsDomain for $name {
            type Time = $time;
            type Length = $scalar;
            type Mass = $scalar;
            type Density = $scalar;
            type Force = $vector;
            type Position = $vector;
            type Displacement = $vector;
            type Velocity = $vector;
            type Acceleration = $vector;
        }
    };
}

domain!(F64Domain, f64, f64, glam::DVec3);
domain!(WrappedDomain, f32, Time<f32>, glam::DVec3);
domain!(WrappedF64Domain, f64, Time<f64>, glam::DVec3);

fn config() -> TimeStepConfig<f64> {
    TimeStepConfig {
        cfl: 0.25,
        acceleration_factor: 0.25,
        viscosity_factor: 0.125,
        max_dt: 1.0,
    }
}

fn resting() -> TimeStepState<f64> {
    TimeStepState {
        max_speed: 0.0,
        max_sound_speed: 0.0,
        max_acceleration: 0.0,
        max_kinematic_viscosity: 0.0,
    }
}

#[test]
fn acoustic_limit_uses_sound_and_flow_speed() {
    let state = TimeStepState {
        max_speed: 1.0,
        max_sound_speed: 3.0,
        ..resting()
    };
    assert_eq!(calc_dt::<F64Domain>(2.0, state, config()), 0.125);
}

#[test]
fn acceleration_and_viscosity_each_limit_the_step() {
    let acceleration = TimeStepState {
        max_acceleration: 4.0,
        ..resting()
    };
    assert_eq!(calc_dt::<F64Domain>(4.0, acceleration, config()), 0.25);
    let viscosity = TimeStepState {
        max_kinematic_viscosity: 4.0,
        ..resting()
    };
    assert_eq!(calc_dt::<F64Domain>(2.0, viscosity, config()), 0.125);
}

#[test]
fn selects_strictest_constraint_and_respects_max_dt() {
    let state = TimeStepState {
        max_speed: 1.0,
        max_sound_speed: 1.0,
        max_acceleration: 16.0,
        max_kinematic_viscosity: 2.0,
    };
    assert_eq!(
        calc_dt::<F64Domain>(2.0, state, config()),
        0.08838834764831845
    );
    let capped = TimeStepConfig {
        max_dt: 0.01,
        ..config()
    };
    assert_eq!(calc_dt::<F64Domain>(2.0, state, capped), 0.01);
}

#[test]
fn zero_constraints_use_max_dt_without_an_epsilon() {
    assert_eq!(calc_dt::<F64Domain>(2.0, resting(), config()), 1.0);
    let tiny = TimeStepState {
        max_sound_speed: 1e-20,
        ..resting()
    };
    let uncapped = TimeStepConfig {
        max_dt: 1e30,
        ..config()
    };
    assert_eq!(calc_dt::<F64Domain>(2.0, tiny, uncapped), 5e19);
}

#[test]
fn works_with_f32_and_time_wrapper() {
    let step: Time<f32> = calc_dt::<WrappedDomain>(
        2.0_f32,
        TimeStepState {
            max_speed: 1.0,
            max_sound_speed: 3.0,
            max_acceleration: 0.0,
            max_kinematic_viscosity: 0.0,
        },
        TimeStepConfig {
            cfl: 0.5,
            acceleration_factor: 0.25,
            viscosity_factor: 0.125,
            max_dt: Time(1.0),
        },
    );
    assert_eq!(step, Time(0.25));
}

#[test]
fn wrapped_domain_time_is_returned_and_respects_its_cap() {
    let config = TimeStepConfig {
        max_dt: Time(0.01),
        cfl: 0.25,
        acceleration_factor: 0.25,
        viscosity_factor: 0.125,
    };
    let step: Time<f64> = calc_dt::<WrappedF64Domain>(2.0, resting(), config);
    assert_eq!(step, Time(0.01));
}
