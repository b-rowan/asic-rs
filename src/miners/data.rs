use crate::miners::api::ApiClient;
use crate::miners::backends::traits::GetMinerData;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use strum::{EnumIter, IntoEnumIterator};

/// Represents the individual pieces of data that can be queried from a miner device.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy, EnumIter)]
pub enum DataField {
    /// Schema version of the miner data.
    SchemaVersion,
    /// Timestamp of when the data was collected.
    Timestamp,
    /// IP address of the miner.
    Ip,
    /// MAC address of the miner.
    Mac,
    /// Information about the miner's device.
    DeviceInfo,
    /// Serial number of the miner.
    SerialNumber,
    /// Hostname assigned to the miner.
    Hostname,
    /// Version of the miner's API.
    ApiVersion,
    /// Firmware version of the miner.
    FirmwareVersion,
    /// Control board version of the miner.
    ControlBoardVersion,
    /// Expected number of hashboards.
    ExpectedHashboards,
    /// Details about the hashboards (e.g., temperatures, chips, etc.).
    Hashboards,
    /// Current hashrate reported by the miner.
    Hashrate,
    /// Expected number of chips across all hashboards.
    ExpectedChips,
    /// Total number of chips detected.
    TotalChips,
    /// Expected number of fans.
    ExpectedFans,
    /// Fan speed or fan configuration.
    Fans,
    /// PSU fan speed or configuration.
    PsuFans,
    /// Average temperature reported by the miner.
    AverageTemperature,
    /// Fluid temperature reported by the miner.
    FluidTemperature,
    /// Current power consumption in watts.
    Wattage,
    /// Configured power limit in watts.
    WattageLimit,
    /// Efficiency of the miner (e.g., J/TH).
    Efficiency,
    /// Whether the fault or alert light is flashing.
    LightFlashing,
    /// Messages reported by the miner (e.g., errors or warnings).
    Messages,
    /// Uptime in seconds.
    Uptime,
    /// Whether the miner is currently hashing.
    IsMining,
    /// Pool configuration (addresses, statuses, etc.).
    Pools,
}

/// A function pointer type that takes a JSON `Value` and an optional key,
/// returning the extracted value if found.
type ExtractorFn = for<'a> fn(&'a Value, Option<&'static str>) -> Option<&'a Value>;

/// Describes how to extract a specific value from a command's response.
///
/// Created by a backend and used to locate a field within a JSON structure.
#[derive(Clone, Copy)]
pub struct DataExtractor {
    /// Function used to extract data from a JSON response.
    pub func: ExtractorFn,
    /// Optional key or pointer within the response to extract.
    pub key: Option<&'static str>,
}

/// Alias for a tuple describing the API command and the extractor used to parse its result.
pub type DataLocation = (&'static str, DataExtractor);

/// Extracts a value from a JSON object using a key (flat lookup).
///
/// Returns `None` if the key is `None` or not found in the object.
pub fn get_by_key<'a>(data: &'a Value, key: Option<&str>) -> Option<&'a Value> {
    data.get(key?.to_string())
}

/// Extracts a value from a JSON object using a JSON pointer path.
///
/// Returns `None` if the pointer is `None` or the path doesn't exist.
pub fn get_by_pointer<'a>(data: &'a Value, pointer: Option<&str>) -> Option<&'a Value> {
    data.pointer(pointer?)
}

/// A utility for collecting structured miner data from an API backend.
pub struct DataCollector<'a> {
    /// Backend-specific data mapping logic.
    miner: &'a dyn GetMinerData,
    /// API client used to send commands to the miner.
    api_client: &'a dyn ApiClient,
    /// Cache of command responses keyed by command string.
    cache: HashMap<String, Value>,
}

impl<'a> DataCollector<'a> {
    /// Constructs a new `DataCollector` with the given backend and API client.
    pub fn new(miner: &'a dyn GetMinerData, api_client: &'a dyn ApiClient) -> Self {
        Self {
            miner,
            api_client,
            cache: HashMap::new(),
        }
    }

    /// Collects **all** available fields from the miner and returns a map of results.
    pub async fn collect_all(&mut self) -> HashMap<DataField, &Value> {
        self.collect(DataField::iter().collect::<Vec<_>>().as_slice())
            .await
    }

    /// Collects only the specified fields from the miner and returns a map of results.
    ///
    /// This method sends only the minimum required set of API commands.
    pub async fn collect(&mut self, fields: &[DataField]) -> HashMap<DataField, &Value> {
        let mut results = HashMap::new();
        let required_commands = self.get_required_commands(fields);

        for command in required_commands {
            if let Ok(response) = self.api_client.send_command(command).await {
                self.cache.insert(command.to_string(), response);
            }
        }

        // Extract the data for each field using the cached responses.
        for &field in fields {
            if let Some(value) = self.extract_field(field) {
                results.insert(field, value);
            }
        }

        results
    }

    /// Determines the unique set of API commands needed for the requested fields.
    ///
    /// Uses the backend's location mappings to identify required commands.
    fn get_required_commands(&self, fields: &[DataField]) -> HashSet<&'static str> {
        fields
            .iter()
            .flat_map(|&field| self.miner.get_locations(field))
            .map(|(cmd, _)| *cmd)
            .collect()
    }

    /// Attempts to extract the value for a specific field from the cached command responses.
    ///
    /// Uses the extractor function and key associated with the field for parsing.
    fn extract_field(&self, field: DataField) -> Option<&Value> {
        for (command, extractor) in self.miner.get_locations(field) {
            if let Some(response_data) = self.cache.get(*command) {
                if let Some(value) = (extractor.func)(response_data, extractor.key) {
                    return Some(value); // Return the first successful extraction.
                }
            }
        }
        None
    }
}
