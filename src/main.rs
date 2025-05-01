use asic_rs::get_miner;
use asic_rs::miners::backends::traits::GetMinerData;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let miner_ip = IpAddr::from([10, 0, 3, 131]);

    let miner = get_miner(miner_ip).await.unwrap();
    if miner.is_some() {
        dbg!(miner.unwrap().get_data().await);
    }

    // let miner = BTMinerV3Backend::new(miner_ip);
    // dbg!(miner.get_device_info().await.unwrap());
    // dbg!(miner.get_miner_status_summary().await.unwrap());
    // dbg!(miner.get_miner_status_pools().await.unwrap());
    // dbg!(miner.get_miner_status_edevs().await.unwrap());
}
