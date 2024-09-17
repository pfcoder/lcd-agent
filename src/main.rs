use dotenv::dotenv;
use log::{error, info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::env;
use std::{fs, time::Duration};
use tokio::select;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
// use tokio_tungstenite::connect_async;

use crate::tasks::watch_machines;
use crate::ws::{connect_to_websocket, receive_message};

mod collector;
mod error;
mod sh;
mod tasks;
mod ws;

const HOME_DIR: &str = ".lcd-agent/";

fn create_home_dir() -> std::io::Result<String> {
    if let Some(mut path) = dirs::home_dir() {
        path.push(HOME_DIR);
        fs::create_dir_all(path.clone())?;
        return Ok(path.to_str().unwrap().to_owned());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Home directory not found",
    ))
}

fn init_log(app_path: &str) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[Console] {d} - {l} -{t} - {m}{n}",
        )))
        .build();

    // Create a file appender with dynamic log path
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[File] {d} - {l} - {t} - {m}{n}",
        )))
        .build(app_path.to_owned() + "/log/info.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    // Use this config
    log4rs::init_config(config).unwrap();
}

fn init_lcd(_app_path: &str) {
    // lcd_core::init(&MinersLibConfig {
    //     app_path: app_path.to_owned(),
    //     is_need_db: false,
    //     // todo: move to env
    //     feishu_app_id: "".to_owned(),
    //     feishu_app_secret: "".to_owned(),
    //     feishu_bot: "".to_owned(),
    //     db_keep_days: 7,
    // });
}

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    dotenv().ok();
    let agent_token = env::var("AGENT_TOEK").expect("AGENT_TOEK must be set");
    // create home dir if not exist
    let app_path = create_home_dir().unwrap();
    init_log(&app_path);
    init_lcd(&app_path);
    let mut sched = JobScheduler::new().await?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(100)
        .enable_all()
        .build()
        .unwrap();

    let runtime_handle = runtime.handle().clone();
    sched
        .add(Job::new_async("0 */2 * * * *", move |_uuid, mut _l| {
            let runtime_handle = runtime_handle.clone();
            Box::pin(async move {
                //info!("I run async every 2 minutes");
                watch_machines(runtime_handle).await;
            })
        })?)
        .await?;

    // Add code to be run during/after shutdown
    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    // Start the scheduler
    sched.start().await?;

    // process websocket
    //let runtime_handle_clone = runtime.handle().clone();
    let rt_handle = runtime.handle().clone();
    runtime.spawn(async move {
        //let url = "ws://45.144.136.65:8080/websocket/a5b913409b4154b869869ea6d5d73e88";
        let url = format!("wss://omni.earthledger.com/websocket/{}", agent_token);
        //let url = "ws://localhost:8080/websocket/a5b913409b4154b869869ea6d5d73e88";
        loop {
            info!("try to connect to websocket server");
            let mut stream;
            match connect_to_websocket(&url).await {
                Ok(ws_stream) => {
                    info!("WebSocket handshake has been successfully completed");
                    stream = ws_stream;
                }
                Err(e) => {
                    error!("Failed to connect: {}", e);
                    // sleep 5 seconds before reconnect
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            }

            receive_message(&mut stream, &rt_handle).await;
            // if return, means error happened, need to reconnect
            tokio::time::sleep(Duration::from_secs(10)).await; // Wait before attempting to reconnect
        }
    });

    // Wait for Ctrl+C or the jobs to finish
    select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl+C received, shutting down");
            // shutdown the scheduler
            sched.shutdown().await?;
        },
    }

    Ok(())
}
