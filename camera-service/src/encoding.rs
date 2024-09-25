use std::{str::FromStr, time::Instant};

use image::{codecs, ImageBuffer, Rgb};
use rav1e::{
    color::ChromaSampling, config::SpeedSettings, data::FrameType, Config, Context, EncoderConfig,
};
use serde::{Deserialize, Serialize};

use crate::{
    camera::{since_the_epoch, VideoPacket},
    prelude::*,
    THRESHOLD_MILLIS,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Encoder {
    MJPEG,
    AV1,
}

impl FromStr for Encoder {
    type Err = ();

    fn from_str(input: &str) -> Result<Encoder, Self::Err> {
        match input {
            "MJPEG" => Ok(Encoder::MJPEG),
            "AV1" => Ok(Encoder::AV1),
            _ => Err(()),
        }
    }
}

pub fn encoder_config(width: usize, height: usize) -> Config {
    let mut speed_settings = SpeedSettings::from_preset(1);
    speed_settings.rdo_lookahead_frames = 1;

    let enc = EncoderConfig {
        width,
        height,
        bit_depth: 8,
        error_resilient: true,
        min_key_frame_interval: 20,
        max_key_frame_interval: 50,
        low_latency: true,
        min_quantizer: 50,
        quantizer: 100,
        still_picture: false,
        tiles: 4,
        chroma_sampling: ChromaSampling::Cs444,
        speed_settings,
        ..Default::default()
    };
    Config::new().with_encoder_config(enc).with_threads(4)
}

pub fn encoder_thread(
    fps_tx: Sender<u128>,
    cam_rx: Receiver<CameraPacket>,
    video_sender: Sender<VideoPacket>,
    encoder: Encoder,
    cfg: Config,
    width: usize,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let fps_tx_copy = fps_tx.clone();
        let mut ctx: Context<u8> = cfg.new_context().unwrap();
        loop {
            let (frame, age) = cam_rx.recv().unwrap();
            // If age older than threshold, throw it away.
            let frame_age = since_the_epoch().as_millis() - age;
            debug!("frame age {}", frame_age);
            if frame_age > THRESHOLD_MILLIS {
                debug!("throwing away old frame with age {} ms", frame_age);
                continue;
            }

            let video_frame = if encoder == Encoder::MJPEG {
                encode_mjpeg(&frame, encoder.clone())
            } else {
                match encode_idk(&frame, encoder.clone(), &mut ctx, width) {
                    Ok(x) => x,
                    Err(_) => continue,
                }
            };
            let _ = video_sender.send(video_frame);
            fps_tx_copy.send(since_the_epoch().as_millis()).unwrap();
        }
    })
}

fn encode_mjpeg(frame: &ImageBuffer<Rgb<u8>, Vec<u8>>, encoder: Encoder) -> VideoPacket {
    let mut buf: Vec<u8> = Vec::new();
    let mut jpeg_encoder = codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 80);
    let _res = jpeg_encoder
        .encode_image(frame)
        .map_err(|e| error!("{:?}", e));
    VideoPacket {
        data: buf.clone(),
        frameType: None,
        epochTime: since_the_epoch(),
        encoding: encoder.clone(),
    }
}

fn encode_idk(
    frame: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    encoder: Encoder,
    context: &mut Context<u8>,
    width: usize,
) -> Result<VideoPacket> {
    let mut r_slice: Vec<u8> = vec![];
    let mut g_slice: Vec<u8> = vec![];
    let mut b_slice: Vec<u8> = vec![];
    for pixel in frame.pixels() {
        let (r, g, b) = to_ycbcr(pixel);
        r_slice.push(r);
        g_slice.push(g);
        b_slice.push(b);
    }
    let planes = vec![r_slice, g_slice, b_slice];
    debug!("Creating new frame");
    let mut frame = context.new_frame();
    let encoding_time = Instant::now();
    for (dst, src) in frame.planes.iter_mut().zip(planes) {
        dst.copy_from_raw_u8(&src, width, 1);
    }

    context.send_frame(frame)?;
    debug!("receiving encoded frame");
    let pkt = context.receive_packet()?;
    debug!("time encoding {:?}", encoding_time.elapsed());
    debug!("read thread: base64 Encoding packet {}", pkt.input_frameno);
    let frame_type = if pkt.frame_type == FrameType::KEY {
        "key"
    } else {
        "delta"
    };
    let data = pkt.data;
    debug!("read thread: base64 Encoded packet {}", pkt.input_frameno);
    let frame = VideoPacket {
        data,
        frameType: Some(frame_type.to_string()),
        epochTime: since_the_epoch(),
        encoding: encoder.clone(),
    };
    Ok(frame)
}

fn clamp(val: f32) -> u8 {
    (val.round() as u8).clamp(0_u8, 255_u8)
}

fn to_ycbcr(pixel: &Rgb<u8>) -> (u8, u8, u8) {
    let [r, g, b] = pixel.0;

    let y = 16_f32 + (65.481 * r as f32 + 128.553 * g as f32 + 24.966 * b as f32) / 255_f32;
    let cb = 128_f32 + (-37.797 * r as f32 - 74.203 * g as f32 + 112.000 * b as f32) / 255_f32;
    let cr = 128_f32 + (112.000 * r as f32 - 93.786 * g as f32 - 18.214 * b as f32) / 255_f32;

    (clamp(y), clamp(cb), clamp(cr))
}
