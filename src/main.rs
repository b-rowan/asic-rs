use asic_rs::get_miner;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let miner_ip = IpAddr::from([192, 168, 86, 21]);

    let miner = get_miner(miner_ip).await.unwrap();
    if miner.is_some() {
        let data = miner.unwrap().get_data().await;
        println!("{:?}", serde_json::to_string(&data).unwrap());
    } else {
        println!("No miner found");
    }
}
