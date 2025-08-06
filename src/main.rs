use asic_rs::MinerFactory;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let miner_ip = IpAddr::from([10, 154, 12, 52]);

    let miner = MinerFactory::new()
        .get_miner(miner_ip)
        .await
        .unwrap()
        .unwrap();

    dbg!(&miner);
    let data = miner.get_data().await;
    dbg!(data);

    // let factory = MinerFactory::new().with_subnet("10.154.12.0/20").unwrap();
    // factory
    //     .scan_stream()
    //     .unwrap()
    //     .for_each_concurrent(1024, async |miner| {
    //         let data = miner.get_data().await;
    //         dbg!(data);
    //     })
    //     .await;
    // println!("Scan completed");
}
