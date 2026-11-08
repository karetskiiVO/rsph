use super::Device;

/// Builds a runtime for the selected device.
pub trait BuildRuntime<D: Device> {
    type Error;
    type Runtime; // TODO: Runtime trait

    fn build_runtime() -> Result<Self::Runtime, Self::Error>;
}
