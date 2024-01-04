use log::Level;

#[derive(Clone, Debug)]
pub struct IndexerConfiguration {
    pub mq: ZMQConfiguration,
    pub net: NetConfiguration,
    pub db_path: String,
    pub save_block_cache_count: u32,
    pub log_configuration: LogConfiguration,
}

#[derive(Clone, Debug)]
pub struct LogConfiguration {
    pub log_level: log::LevelFilter,
}
impl Default for LogConfiguration {
    fn default() -> Self {
        Self {
            log_level: log::LevelFilter::Debug,
        }
    }
}
impl Default for IndexerConfiguration {
    fn default() -> Self {
        Self {
            mq: ZMQConfiguration {
                zmq_url: "tcp://0.0.0.0:28332".to_string(),
                zmq_topic: vec!["sequence".to_string(), "rawtx".to_string()],
            },
            net: Default::default(),
            db_path: "./indexerdb".to_string(),
            save_block_cache_count: 10,
            log_configuration: LogConfiguration {
                log_level: Level::Debug,
            },
        }
    }
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
