use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolScheme {
    StratumV1,
    StratumV1SSL,
    StratumV2,
}

impl From<String> for PoolScheme {
    fn from(scheme: String) -> Self {
        match scheme.as_str() {
            "stratum+tcp" => PoolScheme::StratumV1,
            "stratum+ssl" => PoolScheme::StratumV1SSL,
            "stratum2+tcp" => PoolScheme::StratumV2,
            _ => panic!("Invalid pool scheme"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolURL {
    pub scheme: PoolScheme,
    pub host: String,
    pub port: u16,
    pub pubkey: Option<String>,
}

impl From<String> for PoolURL {
    fn from(url: String) -> Self {
        let parsed = Url::parse(&url).expect("Invalid pool URL");
        let scheme = PoolScheme::from(parsed.scheme().to_string());
        let host = parsed.host_str().unwrap().to_string();
        let port = parsed.port().unwrap_or(80);
        let path = parsed.path();
        let pubkey = match path {
            "" => None,
            _ => Some(path[1..].to_string()),
        };
        PoolURL {
            scheme,
            host,
            port,
            pubkey,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolData {
    pub position: Option<u16>,
    pub url: Option<PoolURL>,
    pub accepted_shares: Option<u64>,
    pub rejected_shares: Option<u64>,
    pub active: Option<bool>,
    pub alive: Option<bool>,
    pub user: Option<String>,
}
