use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use lcd_core::error::MinerError;
use log::error;
use log::info;
use serde_json::json;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

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
) {
    ws_stream
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");
}

pub async fn receive_message(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    runtime_handle: &tokio::runtime::Handle,
) {
    loop {
        let msg = ws_stream
            .next()
            .await
            .expect("Failed to receive message")
            .unwrap();
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
                            process_scan(ws_stream, ip, runtime_handle).await;
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
) {
    // go through ip range from 1 to 255, every time 10 machines
    let count = 10;
    for i in 1..=26 {
        let start = (i - 1) * count;
        let result = lcd_core::scan(runtime_handle.clone(), ip, start, 10, 3)
            .await
            .unwrap();
        info!("scan result: {:?}", &result);
        // construct json message

        // convert result to json string
        let converted = serde_json::to_string(&result).unwrap();

        let message = serde_json::json!({
            "name": "scan_result",
            "data": converted,
            "progress": (i as f32) / 26.0
        });

        send_message(ws_stream, &message.to_string()).await;
    }
}
