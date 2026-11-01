use core::{convert::Infallible, fmt};

use crate::{Math, Neighbors, Real, Vector3};

/// CPU index construction and traversal, separate from the shared formulas.
///
/// Implementations must emit valid particle indices, excluding the queried
/// particle, and must not emit the same neighbor more than once. `positions`
/// is unchanged between rebuilding the index and all queries for that stage.
pub trait CpuNeighbors<M: Math>: Neighbors<M> {
    type Index;
    type Error;

    fn build(positions: &[M::Vec3], params: &Self::Params) -> Result<Self::Index, Self::Error>;

    fn rebuild(
        index: &mut Self::Index,
        positions: &[M::Vec3],
        params: &Self::Params,
    ) -> Result<(), Self::Error>;

    fn for_each<F: FnMut(usize)>(
        index: &Self::Index,
        positions: &[M::Vec3],
        particle: usize,
        visit: F,
    );
}

/// Exhaustive directed pairs, without allocating a neighbor list.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllPairs;

impl<M: Math> Neighbors<M> for AllPairs {
    type Params = ();
}

impl<M: Math> CpuNeighbors<M> for AllPairs {
    type Index = ();
    type Error = Infallible;

    fn build(_: &[M::Vec3], _: &()) -> Result<(), Infallible> {
        Ok(())
    }

    fn rebuild(_: &mut (), _: &[M::Vec3], _: &()) -> Result<(), Infallible> {
        Ok(())
    }

    fn for_each<F: FnMut(usize)>(_: &(), positions: &[M::Vec3], particle: usize, mut visit: F) {
        for neighbor in 0..positions.len() {
            if neighbor != particle {
                visit(neighbor);
            }
        }
    }
}

/// Exhaustive search within one inclusive global radius, without list storage.
///
/// Positions must be finite. The radius and its square must both be positive
/// and finite in the selected precision. This is a reference O(n²) search.
#[derive(Clone, Copy, Debug, Default)]
pub struct FixedRadius;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadiusError {
    InvalidRadius,
}

impl fmt::Display for RadiusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("radius and its square must be positive and finite")
    }
}

impl core::error::Error for RadiusError {}

/// Validated squared cutoff stored by [`FixedRadius`].
pub struct RadiusIndex<S> {
    radius_squared: S,
}

fn radius_squared<S: Real>(radius: S) -> Result<S, RadiusError> {
    let squared = radius * radius;
    if !radius.is_finite() || radius <= S::ZERO || !squared.is_finite() || squared <= S::ZERO {
        return Err(RadiusError::InvalidRadius);
    }
    Ok(squared)
}

impl<M: Math> Neighbors<M> for FixedRadius {
    type Params = M::Real;
}

impl<M: Math> CpuNeighbors<M> for FixedRadius {
    type Index = RadiusIndex<M::Real>;
    type Error = RadiusError;

    fn build(_: &[M::Vec3], radius: &M::Real) -> Result<Self::Index, Self::Error> {
        Ok(RadiusIndex {
            radius_squared: radius_squared(*radius)?,
        })
    }

    fn rebuild(
        index: &mut Self::Index,
        _: &[M::Vec3],
        radius: &M::Real,
    ) -> Result<(), Self::Error> {
        index.radius_squared = radius_squared(*radius)?;
        Ok(())
    }

    fn for_each<F: FnMut(usize)>(
        index: &Self::Index,
        positions: &[M::Vec3],
        particle: usize,
        mut visit: F,
    ) {
        let origin = positions[particle];
        for (neighbor, position) in positions.iter().enumerate() {
            if neighbor != particle {
                let displacement = position.sub(origin);
                if displacement.dot(displacement) <= index.radius_squared {
                    visit(neighbor);
                }
            }
        }
    }
}
