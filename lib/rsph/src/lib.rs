//! Statically configured particle simulation with a CPU execution backend.
//!
//! [`Setup`] selects the backend, numeric representation, physical model and
//! neighbor strategy through types. The CPU backend is sequential and evaluates
//! every particle from the same old state, with reusable output storage.
//!
//! Implement [`Physics`] once in terms of [`Math`] to share the physical
//! equations with another backend. The model also supplies the numerical update
//! rule; this crate does not select an SPH formulation or time integrator.
//!
//! CPU neighbor implementations [`cpu::AllPairs`] and [`cpu::FixedRadius`] are
//! reference quadratic searches. GPU compilation and optimized spatial indexes
//! are not implemented here.

mod config;
pub mod cpu;
pub mod math;
mod physics;
mod solver;

pub use config::{Config, Neighbors, Params, Setup};
pub use math::{Math, Real, Vector3};
pub use physics::Physics;
pub use solver::{Execute, ScalarOf, Solver};

#[cfg(test)]
mod tests;
