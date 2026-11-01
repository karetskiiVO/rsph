use std::marker::PhantomData;

use crate::{cpu_runtime::CPURuntimeBuilder, types::{backend::CPU, math::Math, physics_model::Model}};

pub struct RuntimeBuilder<M: Math> {
    math: PhantomData<M>,
}

pub struct Runtime {}

impl Runtime {
    pub fn builder<M: Math>() -> RuntimeBuilder<M> {
        RuntimeBuilder {
            math: PhantomData,
        } 
    }
}

impl<M: Math> RuntimeBuilder<M> {
    pub fn model(self, _model: Model) -> Self { todo!() }

    pub fn cpu_backend(self, _cpu: CPU) -> CPURuntimeBuilder<M> {
        CPURuntimeBuilder { math: PhantomData }
    }
    pub fn gpu_backend(self) -> ! { todo!() }
}
