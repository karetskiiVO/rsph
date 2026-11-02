use std::marker::PhantomData;

use crate::traits::{Device, Model, PhysicsDomain};

pub struct Config<
    PhD: PhysicsDomain,
    D: Device,
    M: Model<PhD>,
> {
    pub device: D,

    _phd: PhantomData<PhD>,
    _m:   PhantomData<M>,
}
