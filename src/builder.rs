use std::sync::Arc;

use crate::{
    permit::{always::AlwaysPermitter, threshold::ThresholdConfig, PermitConfig, Permitter},
    refill::{rate::RateConfig, RefillConfig},
    Bucket, BuildError,
};

/// Builder for a [`Bucket`]
#[derive(Default)]
pub struct Builder {
    initial: Option<u64>,
    refill_config: Option<RefillConfig>,
    permit_config: Option<PermitConfig>,
}

impl Builder {
    /// Set the initial number of tokens in the [`Bucket`]
    pub fn initial(self, initial: u64) -> Self {
        Self {
            initial: Some(initial),
            ..self
        }
    }

    /// Use constant refill rate
    pub fn rate(self, config: RateConfig) -> Self {
        Self {
            refill_config: Some(RefillConfig::Rate(config)),
            ..self
        }
    }

    /// Use threshold-based permit allocation
    pub fn threshold(self, config: ThresholdConfig) -> Self {
        Self {
            permit_config: Some(PermitConfig::Threshold(config)),
            ..self
        }
    }

    /// Build the [`Bucket`]
    pub fn build(self) -> Result<Bucket, BuildError> {
        let refill = self.refill_config.ok_or(BuildError::MissingRefill)?;
        let inner = Arc::new(refill.into_inner_bucket(self.initial.unwrap_or(0)));
        let permitter: Arc<dyn Permitter> = match self.permit_config {
            Some(PermitConfig::Threshold(threshold_config)) => {
                Arc::new(threshold_config.into_permitter(inner.clone()))
            }
            None => Arc::new(AlwaysPermitter::new(inner.clone())),
        };

        Ok(Bucket { permitter, inner })
    }
}
