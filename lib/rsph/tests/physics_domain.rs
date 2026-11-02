use std::ops::{Add, Div, Mul, Sub};

use rsph_lib::traits::PhysicsDomain;

#[derive(Clone, Copy)]
struct Time(f64);

#[derive(Clone, Copy)]
struct Mass(f64);

#[derive(Clone, Copy)]
struct Density;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(f64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Displacement(f64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity(f64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Acceleration(f64);

#[derive(Clone, Copy)]
struct Force(f64);

macro_rules! arithmetic {
    ($lhs:ty, $trait:ident, $method:ident, $op:tt, $rhs:ty => $out:ident) => {
        impl $trait<$rhs> for $lhs {
            type Output = $out;

            fn $method(self, rhs: $rhs) -> Self::Output {
                $out(self.0 $op rhs.0)
            }
        }
    };
}

macro_rules! additive {
    ($quantity:ident) => {
        arithmetic!($quantity, Add, add, +, $quantity => $quantity);
        arithmetic!($quantity, Sub, sub, -, $quantity => $quantity);
    };
}

additive!(Displacement);
additive!(Velocity);
additive!(Acceleration);
additive!(Force);

arithmetic!(Position, Add, add, +, Displacement => Position);
arithmetic!(Position, Sub, sub, -, Position => Displacement);
arithmetic!(Displacement, Div, div, /, Time => Velocity);
arithmetic!(Velocity, Mul, mul, *, Time => Displacement);
arithmetic!(Velocity, Div, div, /, Time => Acceleration);
arithmetic!(Acceleration, Mul, mul, *, Time => Velocity);

impl Div<Mass> for Force {
    type Output = Acceleration;

    fn div(self, mass: Mass) -> Self::Output {
        Acceleration(self.0 / mass.0)
    }
}

struct UnitDomain;

impl PhysicsDomain for UnitDomain {
    type Time = Time;
    type Mass = Mass;
    type Density = Density;
    type Force = Force;
    type Position = Position;
    type Displacement = Displacement;
    type Velocity = Velocity;
    type Acceleration = Acceleration;
}

struct ScalarDomain;

impl PhysicsDomain for ScalarDomain {
    type Time = f64;
    type Mass = f64;
    type Density = f64;
    type Force = f64;
    type Position = f64;
    type Displacement = f64;
    type Velocity = f64;
    type Acceleration = f64;
}

struct Glam3D;

impl PhysicsDomain for Glam3D {
    type Time = f32;
    type Mass = f32;
    type Density = f32;
    type Force = glam::Vec3;
    type Position = glam::Vec3;
    type Displacement = glam::Vec3;
    type Velocity = glam::Vec3;
    type Acceleration = glam::Vec3;
}

// The integrator reuses the timestep using only PhysicsDomain's guarantees.
fn advance<D: PhysicsDomain>(
    position: D::Position,
    velocity: D::Velocity,
    force: D::Force,
    mass: D::Mass,
    dt: D::Time,
) -> D::Position {
    let acceleration = force / mass;
    let velocity = velocity + acceleration * dt;
    position + velocity * dt
}

fn infer_acceleration<D: PhysicsDomain>(
    start: D::Position,
    end: D::Position,
    elapsed: D::Time,
    velocity_interval: D::Time,
) -> D::Acceleration {
    ((end - start) / elapsed) / velocity_interval
}

#[test]
fn integrates_distinct_copy_quantities() {
    let position = advance::<UnitDomain>(
        Position(2.0),
        Velocity(3.0),
        Force(8.0),
        Mass(2.0),
        Time(0.5),
    );

    assert_eq!(position, Position(4.5));
}

#[test]
fn infers_acceleration_from_positions_and_time() {
    let acceleration =
        infer_acceleration::<UnitDomain>(Position(2.0), Position(12.0), Time(2.0), Time(2.0));

    assert_eq!(acceleration, Acceleration(2.5));
}

#[test]
fn permits_shared_scalar_representation() {
    assert_eq!(advance::<ScalarDomain>(2.0, 3.0, 8.0, 2.0, 0.5), 4.5);
    assert_eq!(infer_acceleration::<ScalarDomain>(2.0, 12.0, 2.0, 2.0), 2.5);
}

#[test]
fn integrates_vec3_position_and_vector_quantities() {
    use glam::Vec3;

    let position = advance::<Glam3D>(
        Vec3::new(2.0, -4.0, 10.0),
        Vec3::new(3.0, 2.0, -1.0),
        Vec3::new(8.0, -4.0, 16.0),
        2.0,
        0.5,
    );

    assert_eq!(position, Vec3::new(4.5, -3.5, 11.5));
}
