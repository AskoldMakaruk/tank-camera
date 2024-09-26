use bytes::Bytes;
use protocol::{SignalEnum, TankCommand, UserId};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder,
    },
    ice_transport::{ice_connection_state::RTCIceConnectionState, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration,
        offer_answer_options::{RTCAnswerOptions, RTCOfferOptions},
        peer_connection_state::RTCPeerConnectionState,
        sdp::{sdp_type::RTCSdpType, session_description::RTCSessionDescription},
        RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{track_local_static_sample::TrackLocalStaticSample, TrackLocal},
};

use crate::{camera::VideoPacket, prelude::*, signaling::WebSocketCommand};

#[derive(PartialEq, Eq)]
pub enum ConnState {
    NotConnected,
    Connected,
    Failed,
}
/// initializes webrtc
pub async fn init_connection(
    counter: ConnectionState,
    frame_receiver: Receiver<VideoPacket>,
    webrtc_cmd_receiver: Receiver<WebRtcEnumCommand>,
    ws_sender: Sender<WebSocketCommand>,
) -> anyhow::Result<()> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let ccounter = counter.clone();
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            println!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Connected {
                let mut a = ccounter.lock().unwrap();
                *a = ConnState::Connected;
            }
            Box::pin(async {})
        },
    ));

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
            // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
            // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
            println!("Peer Connection has gone to failed exiting");
            let mut a = counter.lock().unwrap();
            *a = ConnState::Failed;
        }

        Box::pin(async {})
    }));

    let video_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.to_owned(),
            ..Default::default()
        },
        "video".to_owned(),
        "webrtc-rs".to_owned(),
    ));

    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(33));
        loop {
            if let Ok(frame) = frame_receiver.recv() {
                let _ = video_track
                    .write_sample(&Sample {
                        data: Bytes::from(frame.data),
                        duration: Duration::from_secs(1),
                        ..Default::default()
                    })
                    .await;
            }

            let _ = ticker.tick().await;
        }
    });
    tokio::spawn(async move {
        loop {
            if let Ok(cmd) = webrtc_cmd_receiver.recv() {
                match cmd {
                    WebRtcEnumCommand::ReceiveIceHandshake(id, data) => {
                        let result = handle_ice(&data, peer_connection.clone()).await;
                        dbg!(&result);
                        if let Ok(ice) = result {
                            info!("sending ice answer");
                            let _ = ws_sender.send(WebSocketCommand::SendSignal(
                                SignalEnum::TankCommand(TankCommand::IceAnswer(id, ice)),
                            ));
                        } else if let Err(er) = result {
                            error!("{0}", er)
                        }
                    }
                    WebRtcEnumCommand::CloseConn => {
                        let _ = peer_connection.close().await;
                    }
                    WebRtcEnumCommand::ReceiveSdpOffer(id, data) => {
                        let result =
                            receive_sdp_offer_send_answer(peer_connection.clone(), data).await;
                        if let Ok(answer) = result {
                            info!("sending sdp answer");
                            let _ = ws_sender.send(WebSocketCommand::SendSignal(
                                SignalEnum::TankCommand(TankCommand::SdpAnswer(id, answer)),
                            ));
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

type Rtc = Arc<RTCPeerConnection>;
async fn handle_ice(data: &str, conn: Rtc) -> Result<String> {
    info!("ender handle ice");
    let offer = serde_json::from_str::<RTCSessionDescription>(data)?;

    info!("parsed offer ");
    // Set the remote SessionDescription
    conn.set_remote_description(offer).await?;

    info!("set remote desc ");
    // Create an answer
    let answer = conn.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = conn.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    conn.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    if let Some(local_desc) = conn.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        Ok(json_str)
        // let b64 = signal::encode(&json_str);
        // println!("{b64}");
    } else {
        Err(anyhow::Error::msg("generate local_description failed!"))
    }
}

pub async fn receive_sdp_answer(peer_a: Rtc, answer_sdp: String) -> Result<()> {
    warn!("SDP: Receive Answer {:?}", answer_sdp);

    // Setting Remote Description
    let ans = RTCSessionDescription::answer(answer_sdp)?;
    peer_a.set_remote_description(ans).await?;
    Ok(())
}

pub async fn receive_sdp_offer_send_answer(peer_b: Rtc, offer_sdp: String) -> Result<String> {
    warn!("SDP: Video Offer Receive! {}", offer_sdp);

    // Set Remote Description
    let offer_obj = RTCSessionDescription::offer(offer_sdp)?;
    peer_b.set_remote_description(offer_obj).await?;

    // Create SDP Answer
    let answer = peer_b
        .create_answer(Some(RTCAnswerOptions::default()))
        .await?;

    peer_b.set_local_description(answer.clone()).await?;

    info!("SDP: Sending Video Answer {:?}", answer);
    Ok(answer.sdp)
}

pub async fn create_sdp_offer(peer_a: Rtc) -> Result<String> {
    let offer = peer_a
        .create_offer(Some(RTCOfferOptions::default()))
        .await?;
    peer_a.set_local_description(offer.clone()).await?;

    info!("SDP: Sending Offer {:?}", offer.sdp);
    Ok(offer.sdp)
}
pub enum WebRtcEnumCommand {
    ReceiveSdpOffer(UserId, String),
    ReceiveIceHandshake(UserId, String),
    CloseConn,
}
