use asic_rs::MinerFactory;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let _miner_ip = IpAddr::from([192, 168, 1, 199]);

    let miners_subnet = MinerFactory::new()
        .with_subnet("192.168.1.199/32")
        .unwrap()
        .scan()
        .await
        .unwrap();

    let miners_ocets = MinerFactory::new()
        .with_octets("192", "168", "1", "199")
        .unwrap()
        .scan()
        .await
        .unwrap();

    let miners_range = MinerFactory::new()
        .with_range("192.168.1.199")
        .unwrap()
        .scan()
        .await
        .unwrap();

    assert!(miners_subnet.len() == miners_ocets.len() && miners_subnet.len() == miners_range.len());

    for (i, miner) in miners_subnet.iter().enumerate() {
        let data = miner.get_data().await;
        println!(
            "Miner {}: {}",
            i + 1,
            serde_json::to_string_pretty(&data).unwrap()
        );
    }
}
