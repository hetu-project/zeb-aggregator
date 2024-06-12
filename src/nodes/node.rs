use std::collections::{HashSet, VecDeque};
use reqwest::Client;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
use prost::Message;
use crate::business;
use crate::vlc;
use crate::zmessage;
use sea_orm::{DatabaseConnection, QueryFilter};
use sea_orm::entity::*;
use crate::db::entities::{z_messages, merge_logs, clock_infos, node_info};
use chrono::{DateTime};
use async_trait::async_trait;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct P2PNode {
    pub id: String,
    pub rpc_domain: String,
    pub ws_domain: String,
    pub rpc_port: u32,
    pub ws_port: u32,
    pub public_key: Option<String>,
}


#[async_trait]
pub trait NodeOps {
    async fn neighbors(&self) -> Vec<Arc<Mutex<P2PNode>>>;
    async fn update_node_info(&self);
    async fn store_db(&self);
}


impl P2PNode {
    pub async fn query_data(&self, client: Arc<Client>, gateway_type: i32, index: i32) -> Vec<u8> {
        // let client = Client::new();
        let url = format!("http://{}:{}/rpc{}", self.rpc_domain, self.rpc_port, self.rpc_port);
        println!("Request queryByKeyId {}", url);
        let request_id = "fake123";
        let request_data = json!({"id":request_id ,"method": "queryByKeyId","gatewayType":gateway_type,"index":index});
        let response = client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&request_data)
            .send()
            .await;

        match response {
            Ok(resp) => {
                match resp.json::<HashMap<String, Value>>().await {
                    Ok(json_data) => {
                        let key = "result";
                        if json_data.contains_key("result") {
                            let hex_string = json_data["result"].as_str().expect("Expected a string");
                            let mut bytes = vec![0u8; hex_string.len() / 2];
                            hex::decode_to_slice(hex_string, &mut bytes).expect("Failed to decode hex string");
                            let query_res = business::QueryResponse::decode(&*bytes);
                            query_res.unwrap().data
                        } else {
                            println!("The key '{}' does not exist in the JSON data.", key);
                            Vec::new()
                        }

                        // let hex_string = json_data["result"].as_str().expect("Expected a string");
                        // let mut bytes = vec![0u8; hex_string.len() / 2];
                        // hex::decode_to_slice(hex_string, &mut bytes).expect("Failed to decode hex string");
                        // let query_res = business::QueryResponse::decode(&*bytes);
                        // query_res.unwrap().data
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {:?}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to send request: {:?}", e);
                Vec::new()
            }
        }
    }

    pub async fn store_db(&self, client: Arc<Client>, conn: &DatabaseConnection) {
        // let conn = get_conn().await;

        let (clock_nodes_max_id, merge_logs_max_id, z_messagers_max_id) = self.get_indexes(conn).await;

        println!("query record index{} {} {}", clock_nodes_max_id, merge_logs_max_id, z_messagers_max_id);

        let data = self.query_data(client.clone(), business::GatewayType::ClockNode as i32, clock_nodes_max_id).await;
        let clock_nodes = business::ClockInfos::decode(&*data).unwrap().clock_infos;
        for x in &clock_nodes {
            let clock_json = serde_json::to_string(&x.clock.clone().unwrap().values).expect("Failed to serialize HashMap");
            let timestamp_secs = x.create_at / 1000;
            let timestamp_nanos = (x.create_at % 1000) * 1_000_000;
            let create_at = DateTime::from_timestamp(timestamp_secs as i64, timestamp_nanos as u32).unwrap().naive_utc();
            let new_clock_info = clock_infos::ActiveModel {
                id: NotSet,
                clock: ActiveValue::Set(clock_json),
                node_id: ActiveValue::Set(self.id.to_string()),
                message_id: ActiveValue::Set(hex::encode(x.message_id.clone())),
                event_count: ActiveValue::Set(x.count.try_into().unwrap()),
                create_at: ActiveValue::Set(Some(create_at)),
                clock_hash: ActiveValue::Set(hex::encode(x.clock_hash.clone())),
            };
            new_clock_info.insert(conn).await.expect("Fail to Insert Clock Node");
        }


        let data = self.query_data(client.clone(), business::GatewayType::MergeLog as i32, merge_logs_max_id).await;
        let merge_logs = vlc::MergeLogs::decode(&*data).unwrap().merge_logs;
        for x in &merge_logs {
            let timestamp_secs = x.merge_at / 1000;
            let timestamp_nanos = (x.merge_at % 1000) * 1_000_000;
            let merge_at = DateTime::from_timestamp(timestamp_secs as i64, timestamp_nanos as u32).unwrap().naive_utc();

            let new_merge_log = merge_logs::ActiveModel {
                id: NotSet,
                from_id: ActiveValue::Set(hex::encode(x.from_id.clone()).to_string()),
                to_id: ActiveValue::Set(hex::encode(x.to_id.clone()).to_string()),
                start_count: ActiveValue::Set(x.start_count.try_into().unwrap()),
                end_count: ActiveValue::Set(x.end_count.try_into().unwrap()),
                s_clock_hash: ActiveValue::Set(hex::encode(x.s_clock_hash.clone()).to_string()),
                e_clock_hash: ActiveValue::Set(hex::encode(x.e_clock_hash.clone()).to_string()),
                merge_at: ActiveValue::Set(merge_at),
                node_id: ActiveValue::Set(self.id.to_string()),
            };
            new_merge_log.insert(conn).await.expect("Fail to Insert Merge Log");
        }


        let data = self.query_data(client.clone(), business::GatewayType::ZMessage as i32, z_messagers_max_id).await;
        let zmessages = zmessage::ZMessages::decode(&*data).unwrap().messages;
        for x in &zmessages {
            let version: Option<i32> = Some(x.version as i32);
            let new_message = z_messages::ActiveModel {
                id: NotSet,
                message_id: ActiveValue::Set(hex::encode(x.id.clone())),
                version: ActiveValue::Set(version),
                r#type: ActiveValue::Set(x.r#type.try_into().unwrap()),
                public_key: ActiveValue::Set(Option::from(hex::encode(x.public_key.clone()))),
                data: ActiveValue::Set(x.data.clone()),
                signature: ActiveValue::Set(Option::from(x.signature.clone())),
                from: ActiveValue::Set(hex::encode(&*x.from)),
                to: ActiveValue::Set(hex::encode(&*x.to)),
                node_id: ActiveValue::Set(self.id.to_string()),
            };
            new_message.insert(conn).await.expect("Fail to Insert ZMessage");
        }

        let clock_update_index = clock_nodes.len() as i32 + clock_nodes_max_id;
        let merge_logs_update_index = merge_logs.len() as i32 + merge_logs_max_id;
        let zmessages_update_index = zmessages.len() as i32 + z_messagers_max_id;

        println!("update indexes: {} {} {}", clock_update_index, merge_logs_update_index, zmessages_update_index);
        self.update_indexes(conn, clock_update_index, merge_logs_update_index, zmessages_update_index).await;
    }


    pub async fn get_indexes(&self, db: &DatabaseConnection) -> (i32, i32, i32) {
        let result = node_info::Entity::find()
            .filter(node_info::Column::NodeId.eq(self.id.to_string()))
            .one(db)
            .await.expect("Fail to query index").unwrap();
        (result.clock_info_index, result.merge_log_index, result.z_message_index)
    }

    pub async fn update_indexes(&self, db: &DatabaseConnection, clock_info_index: i32, merge_log_index: i32, zmessage_index: i32) {
        if let Some(query_node_info) = node_info::Entity::find()
            .filter(node_info::Column::NodeId.eq(self.id.to_string()))
            .one(db)
            .await.expect("Fail to query node info")
        {
            let mut query_node_info: node_info::ActiveModel = query_node_info.into();

            query_node_info.clock_info_index = Set(clock_info_index);
            query_node_info.merge_log_index = Set(merge_log_index);
            query_node_info.z_message_index = Set(zmessage_index);

            query_node_info.update(db).await.expect("Fail to update index");
        }
    }

    pub async fn neighbors(&self, client: Arc<Client>) -> Vec<P2PNode> {
        // let client = Client::new();

        let url = format!("http://{}:{}/rpc{}", self.rpc_domain, self.rpc_port, self.rpc_port);

        let request_data = json!({"method": "getNeighbors"});

        let response = client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&request_data)
            .send()
            .await
            .expect("Failed to send request");

        if response.status().is_success() {
            let json_data: HashMap<String, Value> = response.json().await.expect("Failed to parse JSON");
            let nodes: Vec<P2PNode> = json_data
                .into_iter()
                .map(|(k, v)| {
                    let rpc_port = v["rpcPort"].as_u64().unwrap() as u32;
                    let ws_port = v["wsPort"].as_u64().unwrap() as u32;
                    let node = P2PNode {
                        id: k,
                        rpc_domain: v["rpcDomain"].as_str().unwrap().to_string(),
                        ws_domain: v["wsDomain"].as_str().unwrap().to_string(),
                        rpc_port,
                        ws_port,
                        public_key: v["publicKey"].as_str().map(|s| s.to_string()),
                    };
                    node
                })
                .collect();
            nodes
        } else {
            Vec::new()
        }
    }

    pub async fn bfs_traverse(&self, client: Arc<Client>) -> Vec<P2PNode> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(self.clone());
        visited.insert(self.id.clone());

        while let Some(current_node) = queue.pop_front() {
            result.push(current_node.clone());
            let neighbors = current_node.neighbors(client.clone()).await;
            for neighbor in neighbors {
                let neighbor_id = neighbor.id.clone();
                if !visited.contains(&neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        result
    }

    pub async fn update_node_info(&self, client: Arc<Client>, conn: &DatabaseConnection) {
        let neighbors = self.neighbors(client).await;
        // let conn = get_conn().await;
        let mut neighbor_nodes: Vec<String> = Vec::new();
        for node in neighbors {
            neighbor_nodes.push(node.id.parse().unwrap());
        }
        let mut is_alive = true;
        if neighbor_nodes.len() == 0 {
            is_alive = false;
        }

        if let Ok(Some(existing_record)) = node_info::Entity::find()
            .filter(node_info::Column::NodeId.eq(self.id.clone()))
            .one(conn)
            .await
        {
            let mut active_model: node_info::ActiveModel = existing_record.into();
            active_model.neighbor_nodes = Set(serde_json::to_string(&neighbor_nodes).unwrap().to_string());
            active_model.is_alive = Set(is_alive);
            active_model.update(conn).await.expect("Fail to Update Node Info");
        } else {
            let new_node_info = node_info::ActiveModel {
                id: NotSet,
                node_id: ActiveValue::Set(self.id.to_string()),
                neighbor_nodes: ActiveValue::Set(serde_json::to_string(&neighbor_nodes).unwrap().to_string()),
                is_alive: ActiveValue::Set(is_alive),
                rpc_domain: ActiveValue::Set(self.rpc_domain.to_string()),
                rpc_port: ActiveValue::Set(self.rpc_port as i32),
                ws_domain: ActiveValue::Set(self.ws_domain.to_string()),
                ws_port:ActiveValue::Set(self.ws_port as i32),
                clock_info_index: NotSet,
                merge_log_index: NotSet,
                z_message_index: NotSet,
            };
            new_node_info.insert(conn).await.expect("Fail to Insert Node Info");
        }
    }
}


// async fn get_merge_logs_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
//     let max_id = merge_logs::Entity::find()
//         .select_only()
//         .column_as(Expr::col(merge_logs::Column::Id).max(), "max_id")
//         .into_tuple::<Option<i32>>()
//         .one(db)
//         .await?
//         .flatten()
//         .unwrap_or(0);
//     Ok(max_id as u32)
// }
//
// async fn get_clock_infos_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
//     let max_id = clock_infos::Entity::find()
//         .select_only()
//         .column_as(Expr::col(clock_infos::Column::Id).max(), "max_id")
//         .into_tuple::<Option<i32>>()
//         .one(db)
//         .await?
//         .flatten()
//         .unwrap_or(0);
//     Ok(max_id as u32)
// }
//
// async fn get_z_messagers_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
//     let max_id = z_messages::Entity::find()
//         .select_only()
//         .column_as(Expr::col(z_messages::Column::Id).max(), "max_id")
//         .into_tuple::<Option<i32>>()
//         .one(db)
//         .await?
//         .flatten()
//         .unwrap_or(0);
//     Ok(max_id as u32)
// }