use std::collections::HashMap;
use crate::miners::api::rpc::errors::RPCError;
use crate::miners::api::rpc::status::RPCCommandStatus;
use crate::miners::api::{APIClient, RPCAPIClient};
use crate::miners::commands::MinerCommand;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::net::IpAddr;
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct CGMinerRPC {
    ip: IpAddr,
    port: u16,
}

impl CGMinerRPC {
    pub fn new(ip: IpAddr) -> Self {
        Self { ip, port: 4028 }
    }

    fn parse_rpc_result(&self, response: &str) -> Result<Value> {
        let mut val: Value = serde_json::from_str(response)?;

        let status_array = val
            .get("STATUS")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing or invalid STATUS array"))?;

        if status_array.is_empty() {
            bail!("Empty STATUS array");
        }

        let status_str = status_array[0]
            .get("STATUS")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing STATUS field"))?;

        let message = status_array[0].get("Msg").and_then(|v| v.as_str());
        let status   = RPCCommandStatus::from_str(status_str, message);

        status.into_result().map_err(|e| {
            dbg!("{}: API Command Error: {}", self.ip, &e);
            anyhow!(e)
        })?;

        if let Some(stats_arr) = val["STATS"].as_array_mut() {
            for item in stats_arr {
                // MM ID0:Summary
                if let Some(s) = item["MM ID0:Summary"].as_str() {
                    let parsed = self.parse_nested_stats(s);
                    item["MM ID0:Summary"] = json!(parsed);
                }

                // HBinfo
                if let Some(s) = item["HBinfo"].as_str() {
                    let parsed = self.parse_nested_stats(s);
                    item["HBinfo"] = json!(parsed);
                }
            }
        }

        Ok(val)
    }

    fn parse_stats(&self, stats: &str) -> HashMap<String, Vec<String>> {
        let mut stats_dict = HashMap::new();

        let re = Regex::new(r"(\w+)\[([^\]]+)\]").unwrap();

        for cap in re.captures_iter(stats) {
            let key = cap[1].to_string();
            let value_str = &cap[2];

            let values: Vec<String> = if value_str.contains(' ') {
                value_str
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                vec![value_str.to_string()]
            };

            stats_dict.insert(key, values);
        }

        stats_dict
    }

    fn parse_nested_stats(&self, stats: &str) -> HashMap<String, HashMap<String, Vec<String>>> {
        let mut outer = HashMap::new();
        let re = Regex::new(r"'([^']+)':\{([^}]*)\}").unwrap();

        for cap in re.captures_iter(stats) {
            let section = cap[1].to_string();
            let inner_str = &cap[2];
            let inner_map = self.parse_stats(inner_str);
            outer.insert(section, inner_map);
        }
        outer
    }
}

#[async_trait]
impl RPCAPIClient for CGMinerRPC {
    async fn send_command(
        &self,
        command: &str,
        _privileged: bool,
        param: Option<Value>,
    ) -> Result<Value> {
        let cmd = match param {
            Some(params) => json!({
                "command": command,
                "parameter": params
            }),
            None => json!({
                "command": command
            }),
        };

        let stream = tokio::net::TcpStream::connect(format!("{}:{}", self.ip, self.port))
            .await
            .map_err(|_| RPCError::ConnectionFailed)?;
        let mut stream = stream;

        let json_str = cmd.to_string();
        stream.write_all(json_str.as_bytes()).await?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        if buffer.is_empty() {
            bail!("No data received from miner");
        }

        let response = String::from_utf8_lossy(&buffer)
            .into_owned()
            .replace('\0', "");

        if response == "Socket connect failed: Connection refused\n" {
            bail!("Miner connection refused");
        }

        self.parse_rpc_result(&response)
    }
}

#[async_trait]
impl APIClient for CGMinerRPC {
    async fn get_api_result(&self, command: &MinerCommand) -> Result<Value> {
        match command {
            MinerCommand::RPC {
                command,
                parameters,
            } => self
                .send_command(command, false, parameters.clone())
                .await
                .map_err(|e| anyhow!(e.to_string())),
            _ => Err(anyhow!("Cannot send non RPC command to RPC API")),
        }
    }
}
#[cfg(test)]
mod parse_rpc_result_nested_tests {
    use super::*;
    use std::net::{Ipv4Addr, IpAddr};
    use crate::test::json::cgminer::avalon::STATS_COMMAND;

    fn test_rpc() -> CGMinerRPC {
        CGMinerRPC::new(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }

    #[test]
    fn parses_full_cgminer_response() {
        let val = test_rpc().parse_rpc_result(STATS_COMMAND).unwrap();

        assert_eq!(val.pointer("/STATUS/0/STATUS"), Some(&json!("S")));

        assert_eq!(
            val.pointer("/STATS/0/MM ID0:Summary/STATS/GHSmm/0"),
            Some(&json!("55032.79"))
        );
        assert_eq!(
            val.pointer("/STATS/0/MM ID0:Summary/STATS/Freq/0"),
            Some(&json!("282.86"))
        );

        assert_eq!(
            val.pointer("/STATS/0/HBinfo/HB0/PVT_T0/0"),
            Some(&json!("58"))
        );
        assert_eq!(
            val.pointer("/STATS/0/HBinfo/HB0/MW0/1"),
            Some(&json!("664"))
        );

        assert_eq!(val.pointer("/STATS/1/ID"), Some(&json!("POOL0")));

        assert_eq!(
            val.pointer("/STATS/0/MM ID0:Summary/STATS/BVer/0"),
            Some(&json!("25052801_14a19a2"))
        );
    }

    #[test]
    fn hbinfo_multiple_blocks_hb0_and_hb1() {
        let resp = r#"
        {
          "STATUS":[{"STATUS":"S","Msg":"ok"}],
          "STATS":[{
            "ID":"AVALON0",
            "HBinfo":"'HB0':{TEMP[55 56]} 'HB1':{TEMP[77 78] VOLT[12.3]}"
          }]
        }"#;
        let val = test_rpc().parse_rpc_result(resp).unwrap();
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB0/TEMP/0"), Some(&json!("55")));
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB1/TEMP/1"), Some(&json!("78")));
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB1/VOLT/0"), Some(&json!("12.3")));
    }

    #[test]
    fn hbinfo_realistic_long_string() {
        let long = "'HB0':{PVT_T0[58 59 60] MW0[100 200]} 'HB1':{PVT_T0[99 98] MW0[300 400]}";
        let resp = format!(
            r#"
            {{
              "STATUS":[{{"STATUS":"S","Msg":"ok"}}],
              "STATS":[{{"HBinfo":"{long}"}}]
            }}"#
        );
        let val = test_rpc().parse_rpc_result(&resp).unwrap();
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB0/PVT_T0/2"), Some(&json!("60")));
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB0/MW0/1"), Some(&json!("200")));
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB1/PVT_T0/0"), Some(&json!("99")));
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB1/MW0/1"), Some(&json!("400")));
    }

    #[test]
    fn hbinfo_empty_block_is_ok() {
        let resp = r#"
        {
          "STATUS":[{"STATUS":"S","Msg":"ok"}],
          "STATS":[{
            "HBinfo":"'HB0':{}"
          }]
        }"#;
        let val = test_rpc().parse_rpc_result(resp).unwrap();
        assert_eq!(val.pointer("/STATS/0/HBinfo/HB0"), Some(&json!({})));
    }
}
