use crate::{connection::WebRtcEnumCommand, prelude::*};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use protocol::{SignalEnum, TankCommand, TankMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type SocketWriteChannel = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SocketReadChannel = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub enum WebSocketCommand {
    ConnectToSignalServer(String),
    SendSignal(SignalEnum),
}
pub async fn socket_cmd_thread(
    cmd_receiver: Receiver<WebSocketCommand>,
    rtc_sender: Sender<WebRtcEnumCommand>,
) -> Result<tokio::task::JoinSet<()>> {
    let (mut socket_tx, socket_rx) = futures_channel::mpsc::unbounded::<Message>();
    let (ch_soc_tx, ch_soc_rx) = mpsc::channel::<SocketWriteChannel>();
    let (ch_socr_tx, ch_socr_rx) = mpsc::channel::<SocketReadChannel>();
    let (sig_tx, sig_rx) = mpsc::channel::<SignalEnum>();

    let mut socket_tx2 = socket_tx.clone();

    let mut set = tokio::task::JoinSet::new();

    set.spawn(async move {
        if let Ok(write) = ch_soc_rx.recv() {
            debug!("received socket write channel");
            let _ = socket_rx.map(Ok).forward(write).await;
        }
    });

    set.spawn(async move {
        if let Ok(read) = ch_socr_rx.recv() {
            debug!("received socket read channel");

            read.for_each(|message| async {
                if let Ok(data) = message.unwrap().into_text() {
                    if let Ok(signal) = serde_json::from_str::<SignalEnum>(&data) {
                        let _ = sig_tx.send(signal);
                    }

                    info!("message from signal server: {0}", data);
                }
            })
            .await;
        }
    });

    set.spawn(async move {
        loop {
            if let Ok(cmd) = cmd_receiver.recv() {
                match cmd {
                    WebSocketCommand::ConnectToSignalServer(ip) => {
                        info!("connecting to signal server at {0}", &ip);
                        let (ws_stream, _) = connect_async(ip).await.expect("Failed to connect");
                        let (write, read) = ws_stream.split();

                        let _ = ch_soc_tx.send(write);
                        let _ = ch_socr_tx.send(read);

                        let ser_text =
                            serde_json::to_string(&SignalEnum::TankCommand(TankCommand::Login));

                        if let Ok(text) = ser_text {
                            let _ = socket_tx.send(Message::text(text)).await;
                        }
                    }
                    WebSocketCommand::SendSignal(signal) => {
                        let ser_text = serde_json::to_string(&signal);
                        if let Ok(text) = ser_text {
                            let _ = socket_tx.send(Message::text(text)).await;
                        }
                    }
                }
            }
        }
    });

    set.spawn(async move {
        loop {
            if let Ok(cmd) = sig_rx.recv() {
                match cmd {
                    SignalEnum::TankMessage(response) => match response {
                        TankMessage::LoginResponse(tank_id) => {
                            info!("My tank id is: {0}", tank_id.inner());
                        }
                        TankMessage::IceConnectionOffer(id, data) => {
                            info!("receiving ICE handshake");
                            let _ =
                                rtc_sender.send(WebRtcEnumCommand::ReceiveIceHandshake(id, data));
                        }
                        TankMessage::SdpConnectionOffer(id, data) => {
                            info!("receiving SDP offer");
                            let _ = rtc_sender.send(WebRtcEnumCommand::ReceiveSdpOffer(id, data));
                        }
                    },

                    _ => trace!("ignore"),
                }
            }
        }
    });

    Ok(set)
}
