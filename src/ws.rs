use std::vec;

use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use lcd_core::error::MinerError;
use lcd_core::miner::entry::PoolConfig;
use log::error;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

#[derive(Serialize, Deserialize, Debug)]
struct BatchConfig {
    ips: Vec<String>,
    pools: Vec<PoolConfig>,
    mode: String,
}

pub async fn connect_to_websocket(
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, MinerError> {
    //let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    match connect_async(url).await {
        Ok((ws_stream, _)) => Ok(ws_stream),
        Err(e) => {
            error!("Failed to connect: {}", e);
            Err(MinerError::WebSocketError(e.to_string()))
        }
    }
}

pub async fn send_message(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    message: &str,
) -> Result<(), MinerError> {
    // info!("send: {}", message);
    ws_stream
        .send(Message::Text(message.to_string()))
        .await
        .map_err(|e| MinerError::WebSocketError(e.to_string()))
}

pub async fn receive_message(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    runtime_handle: &tokio::runtime::Handle,
) {
    loop {
        // if failed to receive message, return to reconnect
        let msg = match ws_stream.next().await {
            Some(Ok(msg)) => msg,
            Some(Err(e)) => {
                error!("Failed to receive message: {}", e);
                return;
            }
            None => {
                error!("Failed to receive message");
                return;
            }
        };

        //.expect("Failed to receive message")
        //.unwrap();
        match msg {
            Message::Text(text) => {
                info!("Received message: {}", text);
                // parser text into json, ignore error
                let json: Value = serde_json::from_str(&text).unwrap_or(json!({}));
                match json["name"].as_str() {
                    Some("scan") => {
                        info!("Received scan command");
                        let ip = json["data"].as_str().unwrap_or("");
                        if ip.is_empty() {
                            error!("IP is empty");
                        } else {
                            match process_scan(ws_stream, ip, runtime_handle).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to process scan: {}", e);
                                    return;
                                }
                            }
                        }
                    }
                    Some("config") => {
                        info!("Received config command");
                        let config = json["data"].as_str().unwrap_or("");
                        if config.is_empty() {
                            error!("Config is empty");
                        } else {
                            // convert config to json
                            let batch_config: BatchConfig = serde_json::from_str(config).unwrap();
                            info!("batch_config: {:?}", &batch_config);
                            match process_config(ws_stream, &batch_config, runtime_handle).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to process config: {}", e);
                                    return;
                                }
                            }
                        }
                    }
                    Some("query") => {
                        info!("Received query command");
                        let ip = json["data"].as_str().unwrap_or("");
                        if ip.is_empty() {
                            error!("IP is empty");
                        } else {
                            // query machine
                            // use watching interface
                            let ips = vec![ip.to_string()];
                            let result = lcd_core::watching(runtime_handle.clone(), ips, 3)
                                .await
                                // ignore error
                                .unwrap_or(vec![]);

                            let message = serde_json::json!({
                                "name": "query_result",
                                "data": if result.len() > 0 {
                                    serde_json::to_string(&result[0]).unwrap()
                                } else {
                                    "{}".to_string()
                                }
                            });

                            match send_message(ws_stream, &message.to_string()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to send query result message: {}", e);
                                    return;
                                }
                            }
                        }
                    }
                    _ => {
                        error!("Received unexpected message type");
                    }
                }
            }
            // handle disconnect
            Message::Close(_) => {
                info!("Received close message");
                return;
            }
            _ => error!("Received unexpected message type"),
        }
    }
}

async fn process_scan(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ip: &str,
    runtime_handle: &tokio::runtime::Handle,
) -> Result<(), MinerError> {
    // go through ip range from 1 to 255, every time 10 machines
    let count = 10;
    for i in 1..=26 {
        let start = (i - 1) * count;
        let result = lcd_core::scan(runtime_handle.clone(), ip, start, 10, 1)
            .await
            .unwrap();
        // info!("scan result: {:?}", &result);
        // construct json message

        // convert result to json string
        let converted = serde_json::to_string(&result).unwrap();

        let message = serde_json::json!({
            "name": "scan_result",
            "data": converted,
            "progress": (((i as f32) / 26.0) * 100.0) as i32
        });

        send_message(ws_stream, &message.to_string()).await?;
    }

    Ok(())
}

async fn process_config(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    batch_config: &BatchConfig,
    runtime_handle: &tokio::runtime::Handle,
) -> Result<(), MinerError> {
    let result = lcd_core::config(
        runtime_handle.clone(),
        batch_config.ips.clone(),
        batch_config.pools.clone(),
        batch_config.mode.clone(),
    )
    .await
    .unwrap();
    info!("config result: {:?}", &result);
    // construct json message

    // convert result to json string
    let converted = serde_json::to_string(&result).unwrap();

    let message = serde_json::json!({
        "name": "config_result",
        "data": converted,
    });

    send_message(ws_stream, &message.to_string()).await
}
