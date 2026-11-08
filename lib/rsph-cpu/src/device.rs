use rsph_lib::traits::Device;

pub struct CPUBuilder {

}

impl CPUBuilder {
    pub fn new() -> Self { Self { } }

    pub fn build(self) -> Result<CPU, ()> { todo!("implement") }
}

pub struct CPU {}

impl CPU {

}

impl Device for CPU {}
