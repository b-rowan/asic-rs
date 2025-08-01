use crate::miners::backends::traits::GetMinerData;
use crate::miners::factory::MinerFactory;
use std::error::Error;
use std::net::IpAddr;

pub mod data;
pub mod miners;
pub(crate) mod test;

pub async fn get_miner(ip: IpAddr) -> Result<Option<Box<dyn GetMinerData>>, Box<dyn Error>> {
    let factory = MinerFactory::new();
    factory.get_miner(ip).await
}
