use crate::{connection::ConnState, encoding::Encoder, prelude::*};

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VideoPacket {
    pub data: Vec<u8>,
    pub frameType: Option<String>,
    pub epochTime: Duration,
    pub encoding: Encoder,
}

pub fn camera_thread(
    client_counter: ConnectionState,
    video_device_index: u32,
    cam_tx: Sender<CameraPacket>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            {
                info!("waiting for connection...");
                thread::sleep(Duration::from_millis(1200));
                let counter = client_counter.lock().unwrap();
                if *counter != ConnState::Connected {
                    continue;
                }
            }
            let requested =
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
            let mut camera = Camera::new(
                nokhwa::utils::CameraIndex::Index(video_device_index), // index
                requested,
            )
            .unwrap();
            camera.open_stream().unwrap();
            loop {
                {
                    let counter = client_counter.lock().unwrap();
                    if *counter != ConnState::Connected {
                        break;
                    }
                }
                let frame = camera.frame().unwrap();
                let decoded = frame.decode_image::<RgbFormat>().unwrap();
                cam_tx.send((decoded, since_the_epoch().as_millis()));
            }
        }
    })
}

pub fn fps_thread(fps_rx: mpsc::Receiver<u128>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut num_frames = 0;
        let mut now_plus_1 = since_the_epoch().as_millis() + 1000;
        warn!("Starting fps loop");
        loop {
            match fps_rx.recv() {
                Ok(dur) => {
                    if now_plus_1 < dur {
                        warn!("FPS: {:?}", num_frames);
                        num_frames = 0;
                        now_plus_1 = since_the_epoch().as_millis() + 1000;
                    } else {
                        num_frames += 1;
                    }
                }
                Err(e) => {
                    error!("Receive error: {:?}", e);
                    panic!("I'm done yo");
                }
            }
        }
    })
}

pub fn since_the_epoch() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}
