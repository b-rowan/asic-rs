use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use std::net::IpAddr;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) async fn send_rpc_command(
    ip: &IpAddr,
    command: &'static str,
) -> Option<serde_json::Value> {
    let mut stream = tokio::net::TcpStream::connect(format!("{}:4028", ip))
        .await
        .unwrap();

    let command = format!("{{\"command\":\"{command}\"}}");

    stream.write_all(command.as_bytes()).await.unwrap();

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await.unwrap();

    let response = String::from_utf8_lossy(&buffer)
        .into_owned()
        .replace('\0', "");
    
    parse_rpc_result(&response)
}

pub(crate) async fn send_web_command(
    ip: &IpAddr,
    command: &'static str,
    https: bool,
) -> Option<(String, HeaderMap, StatusCode)> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to initalize client");
    let scheme = if https { "https" } else { "http" };
    let resp = client
        .execute(
            client
                .get(format!("{}://{}{}", scheme, ip, command))
                .build()
                .expect("Failed to construct request."),
        )
        .await;
    match resp {
        Ok(data) => {
            let resp_headers = &data.headers().to_owned();
            let resp_status = &data.status().to_owned();
            let resp_text = &data.text().await;
            match resp_text {
                Ok(text) => Some((text.clone(), resp_headers.clone(), *resp_status)),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn parse_rpc_result(response: &str) -> Option<serde_json::Value> {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(response);
    let success_codes = ["S", "I"];

    match parsed.ok() {
        Some(data) => {
            let command_status = data["STATUS"][0]["STATUS"].as_str();

            match command_status {
                Some(status) => {
                    if success_codes.contains(&status) {
                        Some(data)
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
        None => None,
    }
}
