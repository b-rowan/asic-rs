use crate::data::miner::MinerData;

pub trait GetMinerData {
    async fn get_data(&self) -> MinerData;
}
