use core::marker::PhantomData;

use crate::{Math, Physics};

/// Backend-independent description of a neighbor strategy.
pub trait Neighbors<M: Math> {
    type Params;
}

/// Compile-time selection of compatible numeric and physical types.
///
/// Execution support is checked separately by [`crate::Execute`]. Merely naming
/// a backend here does not promise that it can execute the configuration.
pub trait Config: Sized {
    type Math: Math;
    type Physics: Physics<Self::Math>;
    type Neighbors: Neighbors<Self::Math>;
    type Backend;
}

/// Convenient configuration without a custom `impl Config`.
pub struct Setup<B, M, P, N> {
    _backend: PhantomData<fn() -> B>,
    _math: PhantomData<fn() -> M>,
    _physics: PhantomData<fn() -> P>,
    _neighbors: PhantomData<fn() -> N>,
}

impl<B, M, P, N> Config for Setup<B, M, P, N>
where
    M: Math,
    P: Physics<M>,
    N: Neighbors<M>,
{
    type Math = M;
    type Physics = P;
    type Neighbors = N;
    type Backend = B;
}

/// Runtime values whose types are determined by the configuration.
pub struct Params<C: Config> {
    pub physics: <C::Physics as Physics<C::Math>>::Params,
    pub neighbors: <C::Neighbors as Neighbors<C::Math>>::Params,
}
