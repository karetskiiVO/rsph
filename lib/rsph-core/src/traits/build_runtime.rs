pub trait BuildRuntime {
    type ErrorKind;
    type Runtime;

    fn build_runtime() -> Result<Self::Runtime, Self::ErrorKind>;
}
