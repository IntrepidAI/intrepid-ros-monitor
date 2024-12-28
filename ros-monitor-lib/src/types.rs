use std::collections::HashMap;
use std::time::Duration;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct DiscoveryEventWrapper {
    pub ts: u64,
    #[serde(flatten)]
    pub event: DiscoveryEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryEvent {
    Ping,
    NodeAdded {
        name: String,
        namespace: String,
        properties: NodeProperties,
    },
    NodeRemoved {
        name: String,
        namespace: String,
    },
    TopicAdded {
        name: String,
        properties: TopicProperties,
    },
    TopicRemoved {
        name: String,
    },
    ServiceAdded {
        name: String,
        properties: ServiceProperties,
    },
    ServiceRemoved {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct NodeProperties {
    pub enclave: String,
    pub publishers: HashMap<String, String>,
    pub subscribers: HashMap<String, String>,
    pub clients: HashMap<String, String>,
    pub services: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct TopicProperties {
    pub types: Vec<String>,
    pub publishers: Vec<PubSubProperties>,
    pub subscribers: Vec<PubSubProperties>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct ServiceProperties {
    pub types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct PubSubProperties {
    pub node_name: String,
    pub node_namespace: String,
    pub topic_type: String,
    pub qos_profile: QosProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct QosProfile {
    pub history: HistoryPolicy,
    pub depth: usize,
    pub reliability: ReliabilityPolicy,
    pub durability: DurabilityPolicy,
    pub deadline: Duration,
    pub lifespan: Duration,
    pub liveliness: LivelinessPolicy,
    pub liveliness_lease_duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum HistoryPolicy {
    KeepAll,
    KeepLast,
    SystemDefault,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum ReliabilityPolicy {
    BestEffort,
    Reliable,
    SystemDefault,
    BestAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum DurabilityPolicy {
    TransientLocal,
    Volatile,
    SystemDefault,
    BestAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum LivelinessPolicy {
    Automatic,
    ManualByNode,
    ManualByTopic,
    SystemDefault,
    BestAvailable,
    Unknown,
}
