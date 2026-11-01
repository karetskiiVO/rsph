use core::ops::AddAssign;

use crate::cpu::{
    AlignedVec3, AllPairs, Cpu, CpuError, CpuMath, CpuNeighbors, FixedRadius, RadiusError, Vec3,
};
use crate::math::{Math, Vector3};
use crate::{Params, Physics, Setup, Solver};

#[derive(Clone, Copy, Debug)]
struct Particle<V> {
    position: V,
    velocity: V,
}

struct Rate<V>(V);

impl<V: Vector3> AddAssign for Rate<V> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.add(rhs.0);
    }
}

struct Springs;

impl<M: Math> Physics<M> for Springs {
    type Params = M::Vec3;
    type Particle = Particle<M::Vec3>;
    type Rate = Rate<M::Vec3>;

    fn position(particle: &Self::Particle) -> M::Vec3 {
        particle.position
    }

    fn local_rate(gravity: &Self::Params, _: &Self::Particle) -> Self::Rate {
        Rate(*gravity)
    }

    fn pair_rate(
        _: &Self::Params,
        particle: &Self::Particle,
        neighbor: &Self::Particle,
    ) -> Self::Rate {
        Rate(neighbor.position.sub(particle.position))
    }

    fn advance(
        _: &Self::Params,
        particle: &Self::Particle,
        rate: Self::Rate,
        dt: M::Real,
    ) -> Self::Particle {
        Particle {
            position: particle.position.add(particle.velocity.scale(dt)),
            velocity: particle.velocity.add(rate.0.scale(dt)),
        }
    }
}

type F64 = CpuMath<f64, Vec3<f64>>;
type SpringConfig = Setup<Cpu, F64, Springs, AllPairs>;
type RadiusConfig = Setup<Cpu, F64, Springs, FixedRadius>;

fn particle(x: f64, velocity: f64) -> Particle<Vec3<f64>> {
    Particle {
        position: Vec3::new(x, 0.0, 0.0),
        velocity: Vec3::new(velocity, 0.0, 0.0),
    }
}

#[test]
fn cpu_steps_read_the_same_old_state_and_reuse_the_new_state() {
    let mut solver = Solver::<SpringConfig>::new(
        Cpu,
        vec![particle(0.0, 1.0), particle(2.0, -1.0)],
        Params {
            physics: Vec3::zero(),
            neighbors: (),
        },
    )
    .unwrap();

    solver.step(0.5).unwrap();
    let p = solver.state().particles();
    assert_eq!((p[0].position.x, p[1].position.x), (0.5, 1.5));
    assert_eq!((p[0].velocity.x, p[1].velocity.x), (2.0, -2.0));

    solver.step(0.5).unwrap();
    let p = solver.state().particles();
    assert_eq!((p[0].position.x, p[1].position.x), (1.5, 0.5));
    assert_eq!((p[0].velocity.x, p[1].velocity.x), (2.5, -2.5));
}

#[test]
fn same_physics_runs_with_f32_aligned_vectors() {
    type C = Setup<Cpu, CpuMath<f32, AlignedVec3<f32>>, Springs, AllPairs>;
    let mut solver = Solver::<C>::new(
        Cpu,
        vec![Particle {
            position: AlignedVec3::zero(),
            velocity: AlignedVec3::new(2.0, 0.0, 0.0),
        }],
        Params {
            physics: AlignedVec3::new(0.0, -2.0, 0.0),
            neighbors: (),
        },
    )
    .unwrap();

    solver.step(0.5_f32).unwrap();
    let p = &solver.state().particles()[0];
    assert_eq!(p.position, AlignedVec3::new(1.0, 0.0, 0.0));
    assert_eq!(p.velocity, AlignedVec3::new(2.0, -1.0, 0.0));
}

#[test]
fn fixed_radius_includes_boundary_and_coincident_particles_but_not_self() {
    let positions = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
    ];
    let index = <FixedRadius as CpuNeighbors<F64>>::build(&positions, &1.0).unwrap();
    let mut found = Vec::new();
    <FixedRadius as CpuNeighbors<F64>>::for_each(&index, &positions, 0, |j| found.push(j));
    assert_eq!(found, [1, 2]);
}

#[test]
fn neighbor_queries_follow_positions_after_each_step() {
    let mut solver = Solver::<RadiusConfig>::new(
        Cpu,
        vec![particle(0.0, 0.0), particle(2.0, -1.0)],
        Params {
            physics: Vec3::zero(),
            neighbors: 1.0,
        },
    )
    .unwrap();
    solver.step(1.0).unwrap();
    assert_eq!(solver.state().particles()[0].velocity.x, 0.0);
    solver.step(1.0).unwrap();
    let p = solver.state().particles();
    assert_eq!((p[0].velocity.x, p[1].velocity.x), (1.0, -2.0));
}

#[test]
fn empty_simulation_can_step() {
    let mut solver = Solver::<SpringConfig>::new(
        Cpu,
        Vec::new(),
        Params {
            physics: Vec3::zero(),
            neighbors: (),
        },
    )
    .unwrap();
    solver.step(0.5).unwrap();
    assert!(solver.state().particles().is_empty());
}

#[test]
fn invalid_time_steps_do_not_advance_particles() {
    let mut solver = Solver::<SpringConfig>::new(
        Cpu,
        vec![particle(1.0, 2.0)],
        Params {
            physics: Vec3::zero(),
            neighbors: (),
        },
    )
    .unwrap();
    for dt in [0.0, -0.1, f64::NAN, f64::INFINITY] {
        assert!(matches!(solver.step(dt), Err(CpuError::InvalidTimeStep)));
        assert_eq!(solver.state().particles()[0].position.x, 1.0);
    }
}

#[test]
fn invalid_radius_is_rejected() {
    for radius in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let result = Solver::<RadiusConfig>::new(
            Cpu,
            vec![particle(0.0, 0.0)],
            Params {
                physics: Vec3::zero(),
                neighbors: radius,
            },
        );
        assert!(matches!(
            result,
            Err(CpuError::Neighbors(RadiusError::InvalidRadius))
        ));
    }
}

#[test]
fn changed_invalid_radius_leaves_particles_unchanged() {
    let mut solver = Solver::<RadiusConfig>::new(
        Cpu,
        vec![particle(1.0, 2.0)],
        Params {
            physics: Vec3::zero(),
            neighbors: 1.0,
        },
    )
    .unwrap();
    solver.state_mut().params_mut().neighbors = f64::NAN;
    assert!(matches!(
        solver.step(0.5),
        Err(CpuError::Neighbors(RadiusError::InvalidRadius))
    ));
    assert_eq!(solver.state().particles()[0].position.x, 1.0);
    assert_eq!(solver.state().particles()[0].velocity.x, 2.0);
}

#[test]
fn changed_radius_takes_effect_on_the_next_step() {
    let mut solver = Solver::<RadiusConfig>::new(
        Cpu,
        vec![particle(0.0, 0.0), particle(1.0, 0.0)],
        Params {
            physics: Vec3::zero(),
            neighbors: 0.5,
        },
    )
    .unwrap();
    solver.step(0.5).unwrap();
    assert_eq!(solver.state().particles()[0].velocity.x, 0.0);
    solver.state_mut().params_mut().neighbors = 2.0;
    solver.step(0.5).unwrap();
    let p = solver.state().particles();
    assert_eq!((p[0].velocity.x, p[1].velocity.x), (0.5, -0.5));
}
