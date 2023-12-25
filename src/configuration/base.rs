#[derive(Clone, Debug)]
pub struct IndexerConfiguration {
    pub mq: ZMQConfiguration,
}

#[derive(Clone, Debug)]
pub struct ZMQConfiguration {
    pub zmq_url: String,
    pub zmq_topic: Vec<String>,
}