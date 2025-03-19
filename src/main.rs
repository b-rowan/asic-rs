use std::net::IpAddr;

use asic_rs::data::device::*;
use asic_rs::miners::factory::get_miner;

#[tokio::main]
async fn main() {
    let miner_ip = IpAddr::from([10, 5, 0, 172]);

    let miner_info = get_miner(miner_ip, None, None).await.unwrap();
    dbg!(miner_info);

    let antminer = DeviceInfo::new(MinerMake::AntMiner, "ANTMINER HS3", MinerFirmware::Stock);

    // Create a Whatsminer device
    let whatsminer = DeviceInfo::new(
        MinerMake::WhatsMiner,
        "WHATSMINER M30S",
        MinerFirmware::Stock,
    );

    dbg!(&antminer);
    dbg!(antminer.clone().hashrate());
    dbg!(&whatsminer);
    dbg!(whatsminer.clone().hashrate());
}
