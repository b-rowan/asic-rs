use asic_rs::MinerFactory;
use futures::StreamExt;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let _miner_ip = IpAddr::from([192, 168, 1, 199]);

    let factory = MinerFactory::new().with_subnet("10.0.0.0/20").unwrap();
    factory
        .scan_stream()
        .unwrap()
        .for_each_concurrent(1024, async |miner| {
            let data = miner.get_data().await;
            dbg!(data);
        })
        .await;
    println!("Scan completed");
}
