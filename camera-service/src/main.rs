#[macro_use]
extern crate log;

use camera::{fps_thread, VideoPacket};
use connection::{ConnState, WebRtcEnumCommand};
use encoding::encoder_config;
use encoding::{encoder_thread, Encoder};
use log::SetLoggerError;
use nokhwa::utils::ApiBackend;
use prelude::*;
use signaling::WebSocketCommand;
use simplelog::*;
use std::env;
use std::str::FromStr;

pub mod camera;
pub mod connection;
pub mod encoding;
pub mod signaling;

pub use camera::camera_thread;

static THRESHOLD_MILLIS: u128 = 1000;

fn setup_logging() -> Result<(), SetLoggerError> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        simplelog::Config::default(),
        TerminalMode::Mixed,
    )])
}
#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;
    let width = 720;
    let height = 480;
    let video_device_index: usize = env::var("VIDEO_DEVICE_INDEX")
        .ok()
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    let framerate: u32 = env::var("FRAMERATE")
        .ok()
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(10u32);
    let encoder = env::var("ENCODER")
        .ok()
        .and_then(|o| Encoder::from_str(o.as_ref()).ok())
        .unwrap_or(Encoder::AV1);

    warn!("Framerate {framerate}");

    let config = encoder_config(width, height);
    let client_counter = Arc::new(Mutex::new(ConnState::NotConnected));

    let (soc_cmd_tx, soc_cmd_rx) = mpsc::channel::<WebSocketCommand>();
    let (rtc_cmd_tx, rtc_cmd_rx) = mpsc::channel::<WebRtcEnumCommand>();
    let (fps_tx, fps_rx) = mpsc::channel::<u128>();
    let (cam_tx, cam_rx) = mpsc::channel::<CameraPacket>();
    let (vid_tx, vid_rx) = mpsc::channel::<VideoPacket>();

    let devices = nokhwa::query(ApiBackend::Video4Linux)?;
    info!("available cameras: {:?}", devices);

    let fps_thread = fps_thread(fps_rx);

    let camera_thread = camera_thread(client_counter.clone(), video_device_index as u32, cam_tx);

    let encoder_thread = encoder_thread(fps_tx, cam_rx, vid_tx, encoder, config, width);

    let _ =
        connection::init_connection(client_counter, vid_rx, rtc_cmd_rx, soc_cmd_tx.clone()).await;
    let signaling_result = signaling::socket_cmd_thread(soc_cmd_rx, rtc_cmd_tx).await;

    const CONNECTION: &str = "ws://127.0.0.1:9002";
    let _ = soc_cmd_tx.send(WebSocketCommand::ConnectToSignalServer(
        CONNECTION.to_owned(),
    ));

    encoder_thread.join().unwrap();
    fps_thread.join().unwrap();
    camera_thread.join().unwrap();

    if let Ok(task_set) = signaling_result {
        task_set.join_all().await;
    }
    Ok(())
}

pub mod prelude {
    pub use anyhow::Result;
    use image::{ImageBuffer, Rgb};

    pub use std::sync::mpsc;
    pub use std::sync::mpsc::{channel, Receiver, Sender};
    pub use std::thread::JoinHandle;
    pub use std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use crate::connection::ConnState;

    pub type ConnectionState = Arc<Mutex<ConnState>>;
    pub type CameraPacket = (ImageBuffer<Rgb<u8>, Vec<u8>>, u128);
}
