use protocol::{SignalEnum, TankCommand, TankId, TankMessage, UserCommand, UserId, UserMessage};

use crate::state;
pub fn handle_operator_message(user_id: UserId, cmd: UserCommand) -> anyhow::Result<()> {
    match cmd {
        UserCommand::IceOffer(tank_id, data) => {
            let msg = SignalEnum::TankMessage(TankMessage::IceConnectionOffer(
                user_id.clone(),
                data.clone(),
            ));
            state::send_message_to_tank(&tank_id, msg)?;
        }
        UserCommand::Login => {
            let tanks = state::get_tank_list();
            let msg = SignalEnum::UserResponse(UserMessage::CameraListGetSuccess(tanks));
            state::send_message_to_operator(&user_id, msg)?;
        }
        UserCommand::SdpOffer(tank_id, data) => {
            let msg =
                SignalEnum::TankMessage(TankMessage::SdpConnectionOffer(user_id.clone(), data));
            state::send_message_to_tank(&tank_id, msg)?;
        }
    };
    Ok(())
}
pub fn handle_tank_message(tank_id: TankId, cmd: TankCommand) -> anyhow::Result<()> {
    match cmd {
        TankCommand::Login => {
            let msg = SignalEnum::TankMessage(TankMessage::LoginResponse(tank_id.clone()));
            state::send_message_to_tank(&tank_id, msg)?;
        }
        TankCommand::NewCamera(_) => todo!(),
        TankCommand::IceAnswer(user_id, data) => {
            let msg = SignalEnum::UserResponse(UserMessage::IceOfferAnswer(tank_id, data));
            state::send_message_to_operator(&user_id, msg)?;
        }
        TankCommand::SdpAnswer(user_id, data) => {
            let msg = SignalEnum::UserResponse(UserMessage::SdpAnswer(tank_id, data));
            state::send_message_to_operator(&user_id, msg)?;
        }
    };
    Ok(())
}
