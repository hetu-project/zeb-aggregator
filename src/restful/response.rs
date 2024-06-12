use std::collections::HashMap;
use serde::{Serialize, Deserialize};


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_id: String,
    pub neighbor_nodes: Vec<String>,
    pub is_alive: bool,
    pub rpc_domain: String,
    pub rpc_port: u32,
    pub ws_domain: String,
    pub ws_port: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NodesOverviewResponse {
    pub nodes: Vec<Node>,
    pub total_node_count: u32,
    pub total_message_count: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeId {
    pub node_id: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageInfo {
    pub message_id: String,
    pub from_addr: String,
    pub to_addr: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDetailResponse {
    pub node_id: String,
    pub is_alive: bool,
    pub clock: HashMap<String, i32>,
    pub message_list: Vec<MessageInfo>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageId {
    pub message_id: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageDetailResponse {
    pub message_id: String,
    pub from_addr: String,
    pub to_addr: String,
    pub clock_list: Vec<MessageClock>,
    pub message_type: i32,
    pub message_data: Vec<u8>,
    pub signature: Vec<u8>,
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageClock {
    pub node_id: String,
    pub clock: HashMap<String, i32>,
    pub clock_hash: String,
}


#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ::prost::Enumeration,
    Serialize,
    Deserialize
)]
#[repr(i32)]
pub enum ZType {
    Rng = 0,
    Event = 1,
    Clock = 2,
    Gateway = 3,
    Zchat = 4,
}

impl ZType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ZType::Rng => "Z_TYPE_RNG",
            ZType::Event => "Z_TYPE_EVENT",
            ZType::Clock => "Z_TYPE_CLOCK",
            ZType::Gateway => "Z_TYPE_GATEWAY",
            ZType::Zchat => "Z_TYPE_ZCHAT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Z_TYPE_RNG" => Some(Self::Rng),
            "Z_TYPE_EVENT" => Some(Self::Event),
            "Z_TYPE_CLOCK" => Some(Self::Clock),
            "Z_TYPE_GATEWAY" => Some(Self::Gateway),
            "Z_TYPE_ZCHAT" => Some(Self::Zchat),
            _ => None,
        }
    }
}
