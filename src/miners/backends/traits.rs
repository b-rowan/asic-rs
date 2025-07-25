use crate::data::miner::MinerData;
use crate::miners::data::{DataField, DataLocation};
use async_trait::async_trait;

/// Trait that every miner backend must implement to provide miner data.
#[async_trait]
pub trait GetMinerData: Send + Sync {
    /// Asynchronously retrieves standardized information about a miner,
    /// returning it as a `MinerData` struct.
    async fn get_data(&self) -> MinerData;

    /// Returns the locations of the specified data field on the miner.
    ///
    /// This associates API commands (routes) with `DataExtractor` structs,
    /// describing how to extract the data for a given `DataField`.
    fn get_locations(&self, data_field: DataField) -> &'static [DataLocation];
}
