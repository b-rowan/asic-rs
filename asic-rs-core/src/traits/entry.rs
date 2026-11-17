use std::fmt::{Debug, Display};
use std::net::IpAddr;

use async_trait::async_trait;

use crate::{
    errors::ModelSelectionError,
    traits::{
        auth::MinerAuth,
        discovery::DiscoveryCommands,
        identification::{FirmwareIdentification, WebResponse},
        miner::Miner,
    },
};

/// Combined trait for firmware registry entries.
///
/// Provides identification logic, discovery commands, and the ability to
/// construct a fully-typed miner instance after identification succeeds.
#[async_trait]
pub trait FirmwareEntry:
    FirmwareIdentification + DiscoveryCommands + Send + Sync + Debug + Display
{
    /// Optional web endpoint checked only after initial discovery identifies this
    /// firmware. Entries sharing this endpoint can resolve overlapping root pages.
    fn secondary_web_path(&self) -> Option<&'static str> {
        None
    }

    /// Construct using the secondary response, avoiding another model/version fetch.
    async fn build_miner_from_web_response(
        &self,
        ip: IpAddr,
        auth: Option<&MinerAuth>,
        _response: &WebResponse<'_>,
    ) -> Result<Box<dyn Miner>, ModelSelectionError> {
        self.build_miner(ip, auth).await
    }

    /// Construct a fully-typed miner instance for the given IP.
    ///
    /// When `auth` is provided, it is used for both discovery (e.g.
    /// AntMiner digest auth) and applied to the miner for runtime
    /// operations. When `None`, backends use their default credentials.
    async fn build_miner(
        &self,
        ip: IpAddr,
        auth: Option<&MinerAuth>,
    ) -> Result<Box<dyn Miner>, ModelSelectionError>;
}
