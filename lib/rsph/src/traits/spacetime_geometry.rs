use crate::traits::RelativisticDomain;

/// Spacetime geometry in the model's coordinate chart.
/// Metric representation, signature and units must match the recovery component.
pub trait SpacetimeGeometry<D: RelativisticDomain> {
    /// Metric data at one event, with enough information for the selected recovery.
    type Metric: Copy;

    /// Invalid coordinates or unavailable/invalid metric data.
    type Error;

    /// Samples geometry at the given coordinate time and spatial position.
    fn metric_at(&self, time: D::Time, position: D::Position) -> Result<Self::Metric, Self::Error>;
}
