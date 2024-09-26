use futures::{SinkExt, StreamExt, TryStreamExt};
use handler::{handle_operator_message, handle_tank_message};
use std::any;
use std::fs::File;
use std::sync::Arc;
use std::{io::Error as IoError, net::SocketAddr, sync::Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Result;

use async_std::task;
use futures::{channel::mpsc::unbounded, future, pin_mut};
use log::{error, info, warn, SetLoggerError};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use simplelog::{CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};

pub mod handler;
pub mod state;

const LOG_FILE: &str = "signalling_server_prototype.log";

//////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Setup Logging
fn setup_logging() -> Result<(), SetLoggerError> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            TerminalMode::Mixed,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(LOG_FILE).unwrap(),
        ),
    ])
}

use protocol::{ProtoId, SignalEnum, TankId, UserId, UserMessage};
use std::net::UdpSocket;

pub fn get_local_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };
    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };
    match socket.local_addr() {
        Ok(addr) => Some(addr.ip().to_string()),
        Err(_) => None,
    }
}

fn generate_id(length: u8) -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect();
    println!("{}", rand_string);
    rand_string
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    let (outgoing, incoming) = ws_stream.split();
    // peer map
    let (tx, rx) = unbounded();
    state::insert_peer(addr, tx.clone());
    let _ = state::send(&addr, SignalEnum::Start);

    let id_mutex = Arc::new(Mutex::new(Option::<ProtoId>::None));

    let broadcast_incoming = incoming
        .try_filter(|msg| {
            // Broadcasting a Close message from one client
            // will close the other clients.
            future::ready(!msg.is_close())
        })
        .try_for_each(|msg| {
            warn!(
                "Received a message from {}: {}",
                addr,
                msg.to_text().unwrap()
            );
            let message = msg.to_text().unwrap().to_string();
            if let Ok(signal) = serde_json::from_str::<SignalEnum>(&message) {
                if signal.is_login() && id_mutex.lock().map(|x| x.is_none()).unwrap_or(false) {
                    if signal.is_tank() {
                        let tank_id = TankId::new("123".to_string());
                        state::insert_tank(addr, tank_id.clone());

                        let msg = SignalEnum::TankMessage(protocol::TankMessage::LoginResponse(
                            tank_id.clone(),
                        ));
                        state::send_message_to_tank(&tank_id, msg);
                        if let Ok(mut x) = id_mutex.lock() {
                            *x = Some(ProtoId::Tank(tank_id));
                        }
                    } else if signal.is_operator() {
                        let user_id = UserId::new(generate_id(10));
                        state::insert_user(addr, user_id.clone());

                        let msg =
                            SignalEnum::UserResponse(UserMessage::LoginResponse(user_id.clone()));
                        state::send_message_to_operator(&user_id, msg);
                        if let Ok(mut x) = id_mutex.lock() {
                            *x = Some(ProtoId::User(user_id));
                        }
                    }
                } else {
                    let result: anyhow::Result<()> = match signal {
                        SignalEnum::TankCommand(cmd) => {
                            if let Ok(Some(ProtoId::Tank(tank_id))) = id_mutex.lock().as_deref() {
                                handle_tank_message(tank_id.clone(), cmd)
                            } else {
                                Ok(())
                            }
                        }
                        SignalEnum::UserCommand(cmd) => {
                            if let Ok(Some(ProtoId::User(user_id))) = id_mutex.lock().as_deref() {
                                handle_operator_message(user_id.clone(), cmd)
                            } else {
                                Ok(())
                            }
                        }
                        _ => Ok(()),
                    };

                    if result.is_err() {
                        error!("Handle Message Error {:?}", result);
                    } else {
                        info!("Handle Message Ok : result {:?}", result);
                    }
                }
                return future::ok(());
            } else {
                error!(" cant' parse");
            }

            future::ok(())
        });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);

    state::remove_peer(&addr);
    match id_mutex.lock().map(|x| x.clone()).ok().flatten() {
        Some(id) => match id {
            ProtoId::Tank(tank_id) => state::remove_tank(&tank_id),
            ProtoId::User(user_id) => state::remove_user(&user_id),
        },
        None => todo!(),
    }
}

async fn run() -> Result<(), IoError> {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(handle_connection(stream, addr));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    match setup_logging() {
        Ok(_) => (),
        Err(e) => {
            println!("Could not start logger,{}\n...exiting", e);
            std::process::exit(1);
        }
    }

    task::block_on(run());
}
