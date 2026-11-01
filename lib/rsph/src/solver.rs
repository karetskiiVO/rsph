use core::marker::PhantomData;

use crate::{Config, Math, Params};

pub type ScalarOf<C> = <<C as Config>::Math as Math>::Real;
type InputOf<C> = <<C as Config>::Backend as Execute<C>>::Input;
type StateOf<C> = <<C as Config>::Backend as Execute<C>>::State;
type ErrorOf<C> = <<C as Config>::Backend as Execute<C>>::Error;

/// Backend capability for one complete, statically selected configuration.
pub trait Execute<C: Config> {
    type State;
    type Input;
    type Error;

    fn init(&mut self, input: Self::Input, params: Params<C>) -> Result<Self::State, Self::Error>;

    fn step(&mut self, state: &mut Self::State, dt: ScalarOf<C>) -> Result<(), Self::Error>;
}

/// Owns a backend and its state without trait objects or runtime mode switches.
pub struct Solver<C: Config>
where
    C::Backend: Execute<C>,
{
    backend: C::Backend,
    state: StateOf<C>,
    _config: PhantomData<fn() -> C>,
}

impl<C: Config> Solver<C>
where
    C::Backend: Execute<C>,
{
    pub fn new(
        mut backend: C::Backend,
        input: InputOf<C>,
        params: Params<C>,
    ) -> Result<Self, ErrorOf<C>> {
        let state = backend.init(input, params)?;
        Ok(Self {
            backend,
            state,
            _config: PhantomData,
        })
    }

    pub fn step(&mut self, dt: ScalarOf<C>) -> Result<(), ErrorOf<C>> {
        self.backend.step(&mut self.state, dt)
    }

    pub fn state(&self) -> &StateOf<C> {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut StateOf<C> {
        &mut self.state
    }
}
