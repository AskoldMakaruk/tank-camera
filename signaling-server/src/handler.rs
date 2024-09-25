use std::net::SocketAddr;

use log::*;
use protocol::{
    OperatorCommand, SignalEnum, TankCommand, TankId, TankResponse, UserId, UserResponse,
};

use crate::state;
pub fn handle_operator_message(
    addr: SocketAddr,
    user_id: UserId,
    cmd: OperatorCommand,
) -> anyhow::Result<()> {
    match cmd {
        OperatorCommand::ConnectTo(tank_id, data) => {
            let msg =
                SignalEnum::OperatorCommand(OperatorCommand::ConnectTo(tank_id.clone(), data));
            state::send_message_to_tank(&tank_id, msg)?;
        }
        OperatorCommand::Login => {
            let tanks = state::get_tank_list();
            let msg = SignalEnum::UserResponse(UserResponse::CameraListGetSuccess(tanks));
            state::send_message_to_operator(&user_id, msg)?;
        }
    };
    Ok(())
}
pub fn handle_tank_message(
    addr: SocketAddr,
    tank_id: TankId,
    cmd: TankCommand,
) -> anyhow::Result<()> {
    match cmd {
        TankCommand::Login => {
            let msg = SignalEnum::TankResponse(TankResponse::LoginResponse(tank_id.clone()));
            state::send_message_to_tank(&tank_id, msg)?;
        }
        TankCommand::NewCamera(_) => todo!(),
    };
    Ok(())
}

pub fn handle_message(
    addr: SocketAddr,
    user_id: UserId,
    message_from_client: String,
) -> Result<(), String> {
    let result: SignalEnum = match serde_json::from_str(&message_from_client) {
        Ok(x) => x,
        Err(_) => {
            println!("Could not deserialize Message {} ", message_from_client);
            return Err("Could not deserialize Message".to_string());
        }
    };
    warn!("Handle {:?} from {:?} , {:?}", result, addr, user_id);

    // let result = match result {
    //     SignalEnum::OperatorCommand(cmd) => ,
    //     SignalEnum::TankCommand(cmd) => ,
    // SignalEnum::VideoAnswer(answer, session_id) => {
    //     let mut session_list_lock = session_list.lock().unwrap();
    //     let possible_session = session_list_lock.get_mut(&session_id);
    //
    //     match possible_session {
    //         None => {
    //             let e_msg = format!(
    //                 "VideoAnswer Session {} Doesn NOT Exist, Groot kak",
    //                 session_id.inner()
    //             );
    //             error!("VideoAnswer Session Doesn NOT Exist, Groot kak");
    //             return Err(e_msg);
    //         }
    //         Some(session_members) => {
    //             let opt_guest = session_members.guest.clone();
    //             let guest = match opt_guest {
    //                 Some(guest) => guest,
    //                 None => {
    //                     let emsg= String::from("IceCandidate Error: No guest in Session, where are you sending Ice Candidates mate? ");
    //                     return Err(emsg);
    //                 }
    //             };
    //             let sig_msg = SignalEnum::VideoAnswer(answer, session_id.clone());
    //             let message = match serde_json::to_string(&sig_msg) {
    //                 Ok(msg) => msg,
    //                 Err(e) => {
    //                     let e_msg = format!(
    //                         "Could not Serialize {:?} as VideoAnswer, {:?}",
    //                         session_id, e
    //                     );
    //                     return Err(e_msg);
    //                 }
    //             };
    //             (message, Destination::OtherPeer(guest))
    //         }
    //     }
    // }
    // SignalEnum::IceCandidate(candidate, session_id) => {
    //     let mut session_list_lock = session_list.lock().unwrap();
    //     let possible_session = session_list_lock.get_mut(&session_id);
    //
    //     match possible_session {
    //         None => {
    //             let e_msg = format!(
    //                 "IceCandidate Session {} Does NOT Exist, Groot kak",
    //                 session_id.inner()
    //             );
    //             error!("IceCandidate Session Does NOT Exist, Groot kak");
    //             return Err(e_msg);
    //         }
    //         Some(session_members) => {
    //             let opt_guest = session_members.guest.clone();
    //             let guest = match opt_guest {
    //                 Some(guest) => guest,
    //                 None => {
    //                     let emsg= String::from("IceCandidate Error: No guest in Session, where are you sending Ice Candidates mate? ");
    //                     return Err(emsg);
    //                 }
    //             };
    //
    //             let host = session_members.host.clone();
    //             let destination_peer;
    //             if user_id == guest {
    //                 destination_peer = host;
    //             } else if user_id == host {
    //                 destination_peer = guest;
    //             } else {
    //                 let user_list_lock = user_list.lock().unwrap();
    //                 let socket_of_misalligned_user = user_list_lock.get(&user_id);
    //                 error!("UserID connection with {} attempted to send ICE peers to session {} when not assigned to the session", user_id.clone().inner(), session_id.clone().inner());
    //                 error!(
    //                     "Socket Address of Illegal user {:?}",
    //                     socket_of_misalligned_user
    //                 );
    //                 error!("Not Forwarding Ice candidate");
    //                 let e_msg = format!("User {:?}, attempted to send Ice Candidate on session {:?}, which User is not a part of", user_id.inner(), session_id.clone());
    //                 return Err(e_msg);
    //             }
    //
    //             let sig_msg = SignalEnum::IceCandidate(candidate, session_id.clone());
    //             let message = match serde_json::to_string(&sig_msg) {
    //                 Ok(msg) => msg,
    //                 Err(e) => {
    //                     let e_msg = format!(
    //                         "Could not Serialize {:?} as VideoAnswer, {:?}",
    //                         session_id.clone(),
    //                         e
    //                     );
    //                     return Err(e_msg);
    //                 }
    //             };
    //             (message, Destination::OtherPeer(destination_peer))
    //         }
    //     }
    // }
    // SignalEnum::ICEError(_, _) => {
    //     unimplemented!("IceError Handling")
    // }
    // SignalEnum::SessionNew => {
    //     let session_id = SessionID::new(generate_id(5));
    //     let sig_msg = SignalEnum::SessionReady(session_id.clone());
    //     let message = match serde_json::to_string(&sig_msg) {
    //         Ok(msg) => msg,
    //         Err(e) => {
    //             let e_msg = format!(
    //                 "Could not Serialize {:?} as SessionReady, {:?}",
    //                 session_id, e
    //             );
    //             return Err(e_msg);
    //         }
    //     };
    //     let session = SessionMembers {
    //         host: user_id,
    //         guest: None,
    //     };
    //     let insert_result = session_list
    //         .lock()
    //         .unwrap()
    //         .insert(session_id.clone(), session.clone());
    //     if insert_result.is_some() {
    //         warn!("Session_id {:?} Replaced \n    old Session value: {:?} \n    New Session value: {:?} \n ",session_id,insert_result, session);
    //     }
    //     (message, Destination::SourcePeer)
    // }
    // ///////////////////////////////////
    // SignalEnum::SessionJoin(session_id) => {
    //     debug!("inside Session Join ");
    //     // Either Send back SessionJoinError Or SessionJoinSuccess
    //     let mut session_list_lock = session_list.lock().unwrap();
    //     let possible_session = session_list_lock.get_mut(&session_id);
    //
    //     match possible_session {
    //         None => {
    //             debug!("Session Doesn NOT Exist");
    //             //  Session Does not Exists Send back error !
    //             let sig_msg = SignalEnum::SessionJoinError("Session Does Not Exist".into());
    //             let message = match serde_json::to_string(&sig_msg) {
    //                 Ok(msg) => msg,
    //                 Err(e) => {
    //                     let e_msg = format!(
    //                         "Could not Serialize {:?} as SessionJoinError, {:?}",
    //                         session_id, e
    //                     );
    //                     return Err(e_msg);
    //                 }
    //             };
    //             (message, Destination::SourcePeer)
    //         }
    //         Some(session_members) => {
    //             debug!("Session Exists ! Begin Signalling Flow ... ");
    //
    //             //  Session Exists Send back ready to start signalling !
    //             session_members.guest = Some(user_id);
    //
    //             let sig_msg = SignalEnum::SessionJoinSuccess(session_id.clone());
    //             let message = match serde_json::to_string(&sig_msg) {
    //                 Ok(msg) => msg,
    //                 Err(e) => {
    //                     let e_msg = format!(
    //                         "Could not Serialize {:?} as SessionJoinSuccess, {:?}",
    //                         session_id, e
    //                     );
    //                     return Err(e_msg);
    //                 }
    //             };
    //             (message, Destination::SourcePeer)
    //         }
    //     }
    // }
    // SignalEnum::Debug => {
    //     debug!("=====================================");
    //     debug!("====== Signalling Server State ======");
    //     debug!("    User List {:?}", user_list);
    //     debug!("    Session List {:?}", session_list);
    //     debug!("====================================");
    //     return Ok(());
    // }
    //     _ => {
    //         let var_name = error!("Should not recieve state, {:?}", result);
    //         return Err(format!("Should not recieve state, {:?}", result));
    //     }
    // };

    // info!(
    //     "Message Handled, Replying to Client {:?} {:?}",
    //     message_to_client, destination
    // );

    Ok(())
}
