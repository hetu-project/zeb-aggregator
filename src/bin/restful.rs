use std::collections::HashMap;
use gateway::db::connection::get_conn;
use gateway::restful::response::{MessageDetailResponse, MessageInfo, Node, NodeDetailResponse, NodesOverviewResponse,MessageClock};
use axum::{
    routing::{get},
    http::StatusCode,
    Json, Router,
};
use axum::extract::Path;
use tower_http::cors::{Any, CorsLayer};
use sea_orm::entity::*;
use sea_orm::query::*;
use gateway::nodes::node::P2PNode;
use gateway::db::entities::{z_messages, merge_logs, clock_infos, node_info};
use tokio::time::{self, Duration};
use std::sync::{Arc};
use reqwest::ClientBuilder;
async fn get_nodes_info() -> Result<Json<NodesOverviewResponse>, StatusCode> {
    let conn = get_conn().await;
    let nodes: Vec<node_info::Model> = node_info::Entity::find().all(conn).await.expect("REASON");
    let message_count = z_messages::Entity::find().count(conn).await.expect("REASON");
    let mut nodes_response: Vec<Node> = vec![];

    for n in &nodes {
        let neighbor_nodes: Vec<String> = serde_json::from_str(&*n.neighbor_nodes.clone()).expect("Failed to parse JSON");
        nodes_response.push(Node {
            node_id: n.node_id.clone(),
            neighbor_nodes,
            is_alive: n.is_alive.clone(),
            rpc_domain: n.rpc_domain.clone(),
            rpc_port: n.rpc_port as u32,
            ws_domain: n.ws_domain.clone(),
            ws_port: n.ws_port as u32,
        })
    }
    let res = NodesOverviewResponse {
        nodes: nodes_response,
        total_node_count: nodes.len() as u32,
        total_message_count: message_count as u32,
    };
    Ok(Json(res))
}

async fn get_node_by_id(Path(id): Path<String>) -> Result<Json<NodeDetailResponse>, StatusCode> {
    let mut message_list: Vec<MessageInfo> = vec![];
    // let id: i32 = id.parse().expect("Failed to convert string to i32");
    let conn = get_conn().await;

    let node_info_query = node_info::Entity::find().filter(node_info::Column::NodeId.eq(id.clone())).one(conn).await.expect("Fail to query").unwrap();
    let messages_query: Vec<z_messages::Model> = z_messages::Entity::find().filter(z_messages::Column::NodeId.eq(id.clone())).all(conn).await.expect("Fail to query");
    let node_clock_infos_query: Vec<clock_infos::Model> = clock_infos::Entity::find().filter(clock_infos::Column::NodeId.eq(id.clone())).all(conn).await.expect("Fail to query");

    let mut max_clock = 0;
    for clock_info in node_clock_infos_query {
        let clock: HashMap<String, u32> = serde_json::from_str(&*clock_info.clock).expect("JSON was not well-formatted");
        let str = clock.values().next().unwrap().to_string();
        let num: i32 = str.parse().unwrap();
        if num > max_clock {
            max_clock = num
        }
    }

    for m in &messages_query {
        message_list.push(MessageInfo {
            message_id: m.message_id.clone(),
            from_addr: m.from.clone(),
            to_addr: m.to.clone(),
        })
    }
    let mut clock: HashMap<String, i32> = HashMap::new();
    clock.insert(node_info_query.node_id.clone(), max_clock);

    let res = NodeDetailResponse {
        node_id: id.clone(),
        is_alive: node_info_query.is_alive.clone(),
        clock,
        message_list,
    };
    Ok(Json(res))
}

async fn get_message_by_id(Path(id): Path<String>) -> Result<Json<MessageDetailResponse>, StatusCode> {
    let conn = get_conn().await;

    let message: z_messages::Model;
    let query_res = z_messages::Entity::find().filter(z_messages::Column::MessageId.eq(id.clone())).one(conn).await.expect("REASON");
    match query_res {
        Some(query_res) => {
            message = query_res;
        }
        None => return Err(StatusCode::NOT_FOUND)
    }

    let node_clock_infos_query: Vec<clock_infos::Model> = clock_infos::Entity::find().filter(clock_infos::Column::MessageId.eq(id.clone())).all(conn).await.expect("Fail to query");


    let mut clock_list = Vec::new();
    for clock_info in node_clock_infos_query {
        let clock_content: HashMap<String, i32> = serde_json::from_str(&*clock_info.clock).unwrap();
        clock_list.push(MessageClock{
            node_id: clock_info.node_id,
            clock: clock_content,
            clock_hash: clock_info.clock_hash,
        })
    }

    let res = MessageDetailResponse {
        message_id: message.message_id.clone(),
        from_addr: message.from.clone(),
        to_addr: message.to.clone(),
        clock_list,
        message_type: message.r#type,
        signature: message.signature.unwrap(),
        message_data: message.data,
    };
    Ok(Json(res))
}


async fn get_merge_log_by_message_id(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let conn = get_conn().await;
    let query_res = z_messages::Entity::find().filter(z_messages::Column::MessageId.eq(id.clone())).one(conn).await.expect("Fail to query message");
    match query_res {
        Some(..) => {}
        None => return Err(StatusCode::NOT_FOUND)
    }

    let node_clock_info_query = clock_infos::Entity::find().filter(clock_infos::Column::MessageId.eq(id.clone())).one(conn).await.expect("Fail to query").unwrap();
    let mut start_merge_logs_query: Vec<merge_logs::Model> = merge_logs::Entity::find().filter(merge_logs::Column::SClockHash.eq(node_clock_info_query.clock_hash.clone())).all(conn).await.expect("Fail to query");
    let mut end_merge_logs_query: Vec<merge_logs::Model> = merge_logs::Entity::find().filter(merge_logs::Column::EClockHash.eq(node_clock_info_query.clock_hash.clone())).all(conn).await.expect("Fail to query");


    start_merge_logs_query.append(&mut end_merge_logs_query);
    let result = serde_json::json!(
      {
          "merge_logs": &start_merge_logs_query,
      }
    );

    Ok(Json(result))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let seed_node_id = std::env::var("SEED_NODE_ID").expect("SEED_NODE_ID not set");
    let seed_node_rpc_domain = std::env::var("SEED_NODE_RPC_DOMAIN").expect("SEED_NODE_RPC_DOMAIN not set");
    let seed_node_ws_domain = std::env::var("SEED_NODE_WS_DOMAIN").expect("SEED_NODE_WS_DOMAIN not set");
    let seed_node_rpc_port = std::env::var("SEED_NODE_RPC_PORT").expect("SEED_NODE_RPC_PORT not set");
    let seed_node_ws_port = std::env::var("SEED_NODE_WS_PORT").expect("SEED_NODE_WS_PORT not set");
    let seed_node_public_key = std::env::var("SEED_NODE_PUBLIC_KEY").expect("SEED_NODE_PUBLIC_KEY not set");

    let node = Arc::new(
        P2PNode {
            id: seed_node_id,
            rpc_domain: seed_node_rpc_domain,
            ws_domain: seed_node_ws_domain,
            rpc_port: seed_node_rpc_port.parse().unwrap(),
            ws_port: seed_node_ws_port.parse().unwrap(),
            public_key: Option::from(seed_node_public_key),
        }
    );

    // for node in node.bfs_traverse().await {
    //     node.update_node_info().await;
    //     node.store_db().await;
    // }


    let client = Arc::new(
        ClientBuilder::new()
            .timeout(Duration::from_secs(5)) // 设置超时时间为10秒
            .build()
            .expect("Failed to build client")
    );
    let conn = get_conn().await;
    let conn = Arc::new(conn);


    let client_clone = Arc::clone(&client);
    let conn_clone = Arc::clone(&conn);
    let node_clone = Arc::clone(&node);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let nodes = node_clone.bfs_traverse(client_clone.clone()).await;
            for node in nodes {
                println!("Handle node: {}", node.id);
                node.update_node_info(client_clone.clone(),&conn_clone).await;
                node.store_db(client_clone.clone(),&conn_clone).await;
            }
        }
    });

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers([http::header::AUTHORIZATION]);

    let app = Router::new()
        .nest(
            "/gateway",
            Router::new()
                .route("/overview", get(get_nodes_info))
                .route("/node/:id", get(get_node_by_id))
                .route("/message/:id", get(get_message_by_id))
                .route("/merge_log_by_message_id/:id", get(get_merge_log_by_message_id))
                .layer(cors),
        );

    let port = std::env::var("RESTFUL_PORT").expect("RESTFUL_PORT not set");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_owned() + &*port).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
