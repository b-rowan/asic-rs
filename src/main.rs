use asic_rs::{MinerFactory, miners::backends::traits::GetMinerData};
use std::net::IpAddr;

#[tokio::main]
async fn main() {
    let miner_ip = IpAddr::from([10, 154, 11, 55]);

    let miner = MinerFactory::new()
        .get_miner(miner_ip)
        .await
        .unwrap()
        .unwrap();

    dbg!(&miner);
    let data = miner.get_data().await;
    dbg!(data);

    // let now = Instant::now();
    // let factory = MinerFactory::new()
    //     .with_range("10.154.11-19.0-255")
    //     .unwrap()
    //     .with_concurrent_limit(10000)
    //     .with_connectivity_timeout_secs(3)
    //     .with_identification_timeout_secs(3);

    // let hosts = &factory.hosts();
    // println!("Scanning {} Hosts", hosts.len());

    // let result: Arc<Mutex<Vec<Box<dyn GetMinerData>>>> = Arc::new(Mutex::new(Vec::new()));
    // let remainder: Arc<Mutex<Vec<IpAddr>>> = Arc::new(Mutex::new(hosts.clone()));

    // factory
    //     .scan_stream_with_ip()
    //     .unwrap()
    //     .for_each_concurrent(None, async |(ip, miner)| {
    //         let mut _remainder = remainder.lock().await;
    //         let index = _remainder.iter().position(|x| *x == ip).unwrap();
    //         _remainder.remove(index);
    //         println!(
    //             "res: {}, rem: {}",
    //             result.lock().await.len(),
    //             &_remainder.len()
    //         );
    //         match miner {
    //             Some(m) => {
    //                 result.lock().await.push(m);
    //             }
    //             None => {}
    //         }
    //     })
    //     .await;

    // println!("Scan completed. Found {}", result.lock().await.len());
    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
}
