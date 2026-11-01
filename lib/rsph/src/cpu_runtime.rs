use std::marker::PhantomData;

use crate::types::math::Math;

pub struct CPURuntimeBuilder<M: Math> {
    pub(crate) math: PhantomData<M>,
}

pub struct CPURuntime<M: Math> {
    math: PhantomData<M>,
}
