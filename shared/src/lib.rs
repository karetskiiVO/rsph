#![cfg_attr(not(test), no_std)]

use glam::Vec3;

pub const MAX_PARTICLES: usize = 8192;
pub const H: f32 = 0.1;
pub const H2: f32 = H * H;
pub const REST_DENSITY: f32 = 1000.0;
pub const STIFFNESS: f32 = 2000.0;
pub const VISCOSITY: f32 = 0.1;
pub const GRAVITY: f32 = -9.81;
pub const DT: f32 = 0.001;
pub const PARTICLE_MASS: f32 = 0.02;
pub const BOUNDARY: f32 = 1.0;

/// Параметры симуляции, передаваемые в шейдер через push constant.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SimParams {
    pub h: f32,
    pub h2: f32,
    pub rest_density: f32,
    pub stiffness: f32,
    pub viscosity: f32,
    pub gravity: f32,
    pub dt: f32,
    pub particle_mass: f32,
    pub boundary: f32,
    pub num_particles: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            h: H,
            h2: H2,
            rest_density: REST_DENSITY,
            stiffness: STIFFNESS,
            viscosity: VISCOSITY,
            gravity: GRAVITY,
            dt: DT,
            particle_mass: PARTICLE_MASS,
            boundary: BOUNDARY,
            num_particles: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }
}

/// Частица SPH. Выровнена под vec4 для эффективной передачи в GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Particle {
    pub position: Vec3,
    pub mass: f32,
}

impl Particle {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            mass: PARTICLE_MASS,
        }
    }
}

/// Кубическое ядро сглаживания (3D).
#[inline]
pub fn kernel(r: f32, h: f32) -> f32 {
    if r > h {
        return 0.0;
    }
    let q = r / h;
    let sigma = 1.0 / (core::f32::consts::PI * h * h * h);
    if q <= 1.0 {
        sigma * (1.0 - 1.5 * q * q + 0.75 * q * q * q)
    } else {
        let tail = 2.0 - q;
        sigma * 0.25 * tail * tail * tail
    }
}

/// Градиент кубического ядра.
#[inline]
pub fn kernel_grad(r: Vec3, r_len: f32, h: f32) -> Vec3 {
    if r_len > h || r_len < 1e-6 {
        return Vec3::ZERO;
    }
    let q = r_len / h;
    let sigma = 1.0 / (core::f32::consts::PI * h * h * h);
    let dw_dr = if q <= 1.0 {
        sigma * (-3.0 * q + 2.25 * q * q) / h
    } else {
        let tail = 2.0 - q;
        sigma * (-0.75 * tail * tail) / h
    };
    (dw_dr / r_len) * r
}

/// Уравнение состояния (слабосжимаемая жидкость).
#[inline]
pub fn equation_of_state(density: f32, rest_density: f32, stiffness: f32) -> f32 {
    stiffness * (density - rest_density)
}
