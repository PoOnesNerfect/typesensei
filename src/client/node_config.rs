#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub host: String,     // For Typesense Cloud use xxx.a1.typesense.net
    pub port: u16,        // For Typesense Cloud use 443
    pub protocol: String, // For Typesense Cloud use https
}

impl From<(String, u16, String)> for NodeConfig {
    fn from((host, port, protocol): (String, u16, String)) -> Self {
        Self {
            host,
            port,
            protocol,
        }
    }
}
