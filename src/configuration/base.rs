#[derive(Clone, Debug)]
pub struct IndexerConfiguration {
    pub mq: ZMQConfiguration,
    pub net: NetConfiguration,
    pub db_path: String,
    pub save_block_cache_count: u32,
}

#[derive(Clone, Debug)]
pub struct NetConfiguration {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl Default for NetConfiguration {
    fn default() -> Self {
        Self {
            url: "http://localhost:18443".to_string(),
            username: "bitcoinrpc".to_string(),
            password: "bitcoinrpc".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ZMQConfiguration {
    pub zmq_url: String,
    pub zmq_topic: Vec<String>,
}
